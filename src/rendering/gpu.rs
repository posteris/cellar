use std::{sync::Arc, time::Instant};

use bytemuck::cast_slice;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    error::EventLoopError,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window as WinitWindow, WindowAttributes},
};

use crate::{
    common::{cell::Cell, size::Size},
    rendering::{canvas::Canvas, pace::Pace, painter::Painter},
};

const CELL_WIDTH_PX: u32 = 8;
const CELL_HEIGHT_PX: u32 = 16;
const FRAME_BUFFERS: usize = 3;
const STAGING_CHUNK_BYTES: u64 = 256 * 1024;

const SHADER: &str = r#"
struct VertexOut {
    @builtin(position) position: vec4f,
};

struct Cell {
    foreground: u32,
    background: u32,
};

struct Grid {
    size: vec2u,
    cell_size: vec2u,
};

@group(0) @binding(0) var<storage, read> cells: array<Cell>;
@group(0) @binding(1) var<uniform> grid: Grid;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOut {
    let corners = array<vec2f, 3>(
        vec2f(-1.0, -1.0),
        vec2f( 3.0, -1.0),
        vec2f(-1.0,  3.0),
    );

    let p = corners[index];

    var out: VertexOut;
    out.position = vec4f(p, 0.0, 1.0);
    return out;
}

fn unpack_rgb(packed: u32) -> vec3f {
    return vec3f(
        f32(packed & 0xffu) / 255.0,
        f32((packed >> 8u) & 0xffu) / 255.0,
        f32((packed >> 16u) & 0xffu) / 255.0,
    );
}

@fragment
fn fs_main(@builtin(position) position: vec4f) -> @location(0) vec4f {
    let coord = vec2u(floor(position.xy / vec2f(grid.cell_size)));

    if (coord.x >= grid.size.x || coord.y >= grid.size.y) {
        return vec4f(0.0, 0.0, 0.0, 1.0);
    }

    let index = coord.y * grid.size.x + coord.x;

    return vec4f(unpack_rgb(cells[index].background), 1.0);
}
"#;

pub struct Window {
    canvas: Canvas,
    window: Option<Arc<WinitWindow>>,
    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,
    pipeline: Option<wgpu::RenderPipeline>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    cells_buffers: Vec<wgpu::Buffer>,
    bind_groups: Vec<wgpu::BindGroup>,
    grid_buffer: Option<wgpu::Buffer>,
    staging_belt: Option<wgpu::util::StagingBelt>,
    frame: usize,
    next_paint: Option<Instant>,
}

impl Window {
    #[inline]
    pub fn new(size: Size) -> Self {
        let size = Size::new(size.width().max(1), size.height().max(1));

        Self {
            canvas: Canvas::new(size),
            window: None,
            surface: None,
            device: None,
            queue: None,
            config: None,
            pipeline: None,
            bind_group_layout: None,
            cells_buffers: Vec::new(),
            bind_groups: Vec::new(),
            grid_buffer: None,
            staging_belt: None,
            frame: 0,
            next_paint: None,
        }
    }

    pub fn run(&mut self, painter: &mut impl Painter) -> Result<(), EventLoopError> {
        let event_loop = EventLoop::new()?;
        let mut app = App {
            backend: self,
            painter,
        };
        event_loop.run_app(&mut app)
    }

    #[inline]
    fn request_redraw(&self) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn resume(&mut self, event_loop: &ActiveEventLoop) {
        let size = self.canvas.size();
        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title("Reticuli")
                        .with_inner_size(PhysicalSize::new(
                            size.width() as u32 * CELL_WIDTH_PX,
                            size.height() as u32 * CELL_HEIGHT_PX,
                        )),
                )
                .expect("failed to create window"),
        );

        let instance = wgpu::Instance::default();
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .expect("no suitable GPU adapter");
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("failed to request device");

        let surface = instance
            .create_surface(window.clone())
            .expect("failed to create surface");
        let window_size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);

        // Cell colors from `Canvas` are already sRGB bytes, so use a non-sRGB
        // surface format. An sRGB format would encode them a second time and
        // make every color noticeably lighter.
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|format| !format.is_srgb())
            .expect("no non-sRGB surface format available");

        let mut config = surface
            .get_default_config(&adapter, window_size.width, window_size.height)
            .expect("surface is not supported by this adapter");
        config.format = format;
        config.color_space = wgpu::SurfaceColorSpace::Srgb;
        config.present_mode = wgpu::PresentMode::Fifo;
        surface.configure(&device, &config);

        let cells_buffers = Self::create_cells_buffers(&device, size);
        let grid_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self::write_grid(&queue, &grid_buffer, size);
        let staging_belt = wgpu::util::StagingBelt::new(device.clone(), STAGING_CHUNK_BYTES);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_groups =
            Self::create_bind_groups(&device, &bind_group_layout, &cells_buffers, &grid_buffer);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        self.window = Some(window);
        self.surface = Some(surface);
        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(config);
        self.pipeline = Some(pipeline);
        self.bind_group_layout = Some(bind_group_layout);
        self.cells_buffers = cells_buffers;
        self.bind_groups = bind_groups;
        self.grid_buffer = Some(grid_buffer);
        self.staging_belt = Some(staging_belt);
        self.frame = 0;
        self.next_paint = None;

        self.request_redraw();
    }

    fn handle_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
        painter: &mut impl Painter,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(pixels) => {
                let size = grid_size(pixels);

                self.canvas.resize(size);

                {
                    if let (Some(surface), Some(device), Some(config)) =
                        (&self.surface, &self.device, &mut self.config)
                        && pixels.width > 0
                        && pixels.height > 0
                    {
                        config.width = pixels.width;
                        config.height = pixels.height;
                        surface.configure(device, config);
                    }
                }

                {
                    if let (Some(device), Some(queue), Some(layout), Some(grid_buffer)) = (
                        &self.device,
                        &self.queue,
                        &self.bind_group_layout,
                        &self.grid_buffer,
                    ) {
                        Self::write_grid(queue, grid_buffer, size);

                        let cells_buffers = Self::create_cells_buffers(device, size);
                        let bind_groups =
                            Self::create_bind_groups(device, layout, &cells_buffers, grid_buffer);

                        self.cells_buffers = cells_buffers;
                        self.bind_groups = bind_groups;
                    }
                }

                self.request_redraw();
            }

            WindowEvent::RedrawRequested => self.render(painter),

            _ => {}
        }
    }

    fn render(&mut self, painter: &mut impl Painter) {
        let surface = self.surface.as_ref().unwrap();
        let device = self.device.as_ref().unwrap();
        let queue = self.queue.as_ref().unwrap();
        let pipeline = self.pipeline.as_ref().unwrap();
        let config = self.config.as_ref().unwrap();

        // Acquire first. With Fifo this blocks until the presentation queue has
        // room, which paces CPU work before the painter runs.
        let (texture, reconfigure) = match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => (texture, false),
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => (texture, true),
            wgpu::CurrentSurfaceTexture::Outdated => {
                surface.configure(device, config);
                self.request_redraw();
                return;
            }
            wgpu::CurrentSurfaceTexture::Timeout => {
                self.request_redraw();
                return;
            }
            wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Lost
            | wgpu::CurrentSurfaceTexture::Validation => return,
        };

        let pace = painter.paint(&mut self.canvas);
        let redraw_now = match pace {
            Pace::Next => {
                self.next_paint = None;
                true
            }
            Pace::At(deadline) => {
                self.next_paint = Some(deadline);
                false
            }
            Pace::Wait => {
                self.next_paint = None;
                false
            }
        };

        let index = self.frame % FRAME_BUFFERS;
        let bytes = wgpu::BufferSize::new(
            self.canvas.size().area() as u64 * std::mem::size_of::<Cell>() as u64,
        )
        .unwrap();

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let belt = self.staging_belt.as_mut().unwrap();
            let mut view = belt.write_buffer(&mut encoder, &self.cells_buffers[index], 0, bytes);
            view.copy_from_slice(cast_slice::<Cell, u8>(self.canvas.cells()));
        }
        self.staging_belt
            .as_mut()
            .unwrap()
            .finish_and_recall_on_submit(&encoder);

        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &self.bind_groups[index], &[]);
            pass.draw(0..3, 0..1);
        }

        queue.submit(Some(encoder.finish()));
        queue.present(texture);

        if reconfigure {
            surface.configure(device, config);
        }

        self.canvas.clear_delta();
        self.frame += 1;

        if redraw_now {
            self.request_redraw();
        }
    }

    fn create_cells_buffers(device: &wgpu::Device, size: Size) -> Vec<wgpu::Buffer> {
        let bytes = size.area() as u64 * std::mem::size_of::<Cell>() as u64;

        (0..FRAME_BUFFERS)
            .map(|_| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    size: bytes,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .collect()
    }

    fn write_grid(queue: &wgpu::Queue, buffer: &wgpu::Buffer, size: Size) {
        queue.write_buffer(
            buffer,
            0,
            cast_slice::<u32, u8>(&[
                size.width() as u32,
                size.height() as u32,
                CELL_WIDTH_PX,
                CELL_HEIGHT_PX,
            ]),
        );
    }

    fn create_bind_groups(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        cells_buffers: &[wgpu::Buffer],
        grid_buffer: &wgpu::Buffer,
    ) -> Vec<wgpu::BindGroup> {
        cells_buffers
            .iter()
            .map(|cells_buffer| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(
                                cells_buffer.as_entire_buffer_binding(),
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Buffer(
                                grid_buffer.as_entire_buffer_binding(),
                            ),
                        },
                    ],
                })
            })
            .collect()
    }
}

struct App<'a, P: Painter> {
    backend: &'a mut Window,
    painter: &'a mut P,
}

impl<P: Painter> ApplicationHandler for App<'_, P> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.backend.resume(event_loop);
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if matches!(cause, StartCause::ResumeTimeReached { .. })
            && self.backend.next_paint.is_some()
        {
            self.backend.request_redraw();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        match self.backend.next_paint {
            Some(deadline) => event_loop.set_control_flow(ControlFlow::WaitUntil(deadline)),
            None => event_loop.set_control_flow(ControlFlow::Wait),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        self.backend
            .handle_event(event_loop, window_id, event, self.painter);
    }
}

fn grid_size(pixels: PhysicalSize<u32>) -> Size {
    Size::new(
        (pixels.width / CELL_WIDTH_PX).clamp(1, u16::MAX as u32) as u16,
        (pixels.height / CELL_HEIGHT_PX).clamp(1, u16::MAX as u32) as u16,
    )
}
