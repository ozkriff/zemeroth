extern crate hate;

use std::time::Duration;
use hate::{Context, Event, Screen};

pub struct EmptyScreen;

impl Screen for EmptyScreen {
    fn tick(&mut self, _: &mut Context, _: Duration) {}

    fn handle_event(&mut self, _: &mut Context, _: Event) {}
}

pub fn main() {
    let settings = hate::Settings::default();
    let mut visualizer = hate::Visualizer::new(settings);
    let start_screen = Box::new(EmptyScreen);
    visualizer.run(start_screen);
}
