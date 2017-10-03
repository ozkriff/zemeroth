extern crate hate;

pub struct EmptyScreen;

use hate::{Context, Event, Screen, Time};

impl Screen for EmptyScreen {
    fn tick(&mut self, _: &mut Context, _: Time) {}

    fn handle_event(&mut self, _: &mut Context, _: Event) {}
}

pub fn main() {
    let settings = hate::Settings::default();
    let mut visualizer = hate::Visualizer::new(settings);
    let start_screen = Box::new(EmptyScreen);
    visualizer.run(start_screen);
}
