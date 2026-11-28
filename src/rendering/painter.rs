use crate::rendering::{canvas::Canvas, pace::Pace};

pub trait Painter {
    fn paint(&mut self, canvas: &mut Canvas) -> Pace;
}
