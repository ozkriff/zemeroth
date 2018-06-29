use std::fmt::Debug;
use std::time::Duration;

pub use action::change_color_to::ChangeColorTo;
pub use action::empty::Empty;
pub use action::fork::Fork;
pub use action::hide::Hide;
pub use action::move_by::MoveBy;
pub use action::sequence::Sequence;
pub use action::set_color::SetColor;
pub use action::show::Show;
pub use action::sleep::Sleep;

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
