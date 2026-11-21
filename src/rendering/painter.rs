use crate::rendering::frame::Frame;

pub trait Painter {
    fn paint(&mut self, frame: &mut Frame);
}
