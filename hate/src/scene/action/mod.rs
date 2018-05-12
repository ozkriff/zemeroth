use std::fmt::Debug;
use std::time::Duration;

pub use scene::action::change_color_to::ChangeColorTo;
pub use scene::action::fork::Fork;
pub use scene::action::hide::Hide;
pub use scene::action::move_by::MoveBy;
pub use scene::action::sequence::Sequence;
pub use scene::action::set_color::SetColor;
pub use scene::action::show::Show;
pub use scene::action::sleep::Sleep;

// TODO: mod change_size
// TODO: mod change_rotation
// TODO: mod easing

mod change_color_to;
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

    fn try_fork(&mut self) -> Option<Box<Action>> {
        None
    }

    fn is_finished(&self) -> bool {
        true
    }
}
