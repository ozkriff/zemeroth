use std::{fmt::Debug, time::Duration};

pub use crate::action::{
    change_color_to::ChangeColorTo, empty::Empty, fork::Fork, hide::Hide, move_by::MoveBy,
    sequence::Sequence, set_color::SetColor, show::Show, sleep::Sleep,
};

mod change_color_to;
mod empty;
mod fork;
mod hide;
mod move_by;
mod sequence;
mod set_color;
mod show;
mod sleep;

pub trait Action: Debug {
    fn begin(&mut self) {}
    fn update(&mut self, _dtime: Duration) {}
    fn end(&mut self) {}

    fn duration(&self) -> Duration {
        Duration::new(0, 0)
    }

    fn try_fork(&mut self) -> Option<Box<dyn Action>> {
        None
    }

    fn is_finished(&self) -> bool {
        true
    }
}

/// Just a helper trait to replace
/// `Box::new(action::Empty::new())`
/// with
/// `action::Empty::new().boxed()`.
pub trait Boxed {
    type Out;

    fn boxed(self) -> Self::Out;
}

impl<T: 'static + Action> Boxed for T {
    type Out = Box<dyn Action>;

    fn boxed(self) -> Self::Out {
        Box::new(self)
    }
}
