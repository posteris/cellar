use std::{
    thread,
    time::{Duration, Instant},
};

use reticuli::{
    common::{cell::Cell, color::Color, size::Size},
    rendering::{
        ansi::terminal::Terminal, canvas::Canvas, gpu::Window, pace::Pace, painter::Painter,
    },
};

const TARGET_FPS: u64 = 60;
const FRAME_INTERVAL: Duration = Pace::interval(TARGET_FPS);

struct Checkerboard {
    last_paint: Option<Instant>,
    elapsed: Duration,
    next_frame: Instant,
}

impl Checkerboard {
    #[inline]
    fn new() -> Self {
        Self {
            last_paint: None,
            elapsed: Duration::ZERO,
            next_frame: Instant::now(),
        }
    }

    #[inline]
    fn advance(&mut self) {
        let now = Instant::now();

        if let Some(last) = self.last_paint {
            self.elapsed += now.saturating_duration_since(last);
        }

        self.last_paint = Some(now);
    }

    #[inline]
    fn next_pace(&mut self) -> Pace {
        let now = Instant::now();

        self.next_frame = if self.next_frame <= now {
            now + FRAME_INTERVAL
        } else {
            self.next_frame + FRAME_INTERVAL
        };

        Pace::At(self.next_frame)
    }
}

impl Painter for Checkerboard {
    fn paint(&mut self, canvas: &mut Canvas) -> Pace {
        self.advance();

        let pace = self.next_pace();
        let shift = (self.elapsed.as_millis() / FRAME_INTERVAL.as_millis()) as u32;

        canvas.fill_with(|position| {
            let x = position.x() as u32 + shift;
            let y = position.y() as u32;

            if (x / 3) % 2 == (y / 3) % 2 {
                Cell::solid(Color::new(230, 120, 40))
            } else {
                Cell::solid(Color::new(20, 30, 50))
            }
        });

        pace
    }
}

fn main() {
    let handler = thread::spawn(|| {
        let mut painter = Checkerboard::new();
        Terminal::new().unwrap().run(&mut painter).unwrap();
    });

    {
        let mut painter = Checkerboard::new();
        Window::new(Size::new(80, 40)).run(&mut painter).unwrap();
    }

    handler.join().unwrap();
}
