use context::Context;
use time::Time;
use event::Event;

pub enum Command {
    Push(Box<Screen>),
    Pop,
}

pub trait Screen {
    fn tick(&mut self, context: &mut Context, dtime: Time);

    fn handle_event(&mut self, context: &mut Context, event: Event);
}
