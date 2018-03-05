use std::time::Duration;
use context::Context;
use event::Event;

pub enum Command {
    Push(Box<Screen>),
    Pop,
}

pub trait Screen {
    fn tick(&mut self, context: &mut Context, dtime: Duration);

    fn handle_event(&mut self, context: &mut Context, event: Event);
}
