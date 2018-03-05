use std::time::Duration;
use std::fmt::Debug;

pub use scene::action::sequence::Sequence;
pub use scene::action::show::Show;
pub use scene::action::hide::Hide;
pub use scene::action::move_by::MoveBy;
pub use scene::action::fork::Fork;
pub use scene::action::sleep::Sleep;
pub use scene::action::change_color_to::ChangeColorTo;
pub use scene::action::set_color::SetColor;

// TODO: mod change_size
// TODO: mod change_rotation
// TODO: mod easing

mod sequence;
mod fork;
mod sleep;
mod hide;
mod show;
mod move_by;
mod set_color;
mod change_color_to;

pub trait Action: Debug {
    fn begin(&mut self) {}
    fn update(&mut self, _dtime: Duration) {}
    fn end(&mut self) {}

    fn duration(&self) -> Duration {
        Duration::new(0, 0)
    }

    fn try_fork(&mut self) -> Option<Box<Action>> {
        None
    }

    fn is_finished(&self) -> bool {
        true
    }
}
