use std::thread;
use std::sync::mpsc;
use std::time::{Instant, Duration};
use screen;
use screen::Screen;
use context::Context;
use settings::Settings;
use screen_stack::Screens;

fn max_frame_time(context: &Context) -> Duration {
    Duration::from_millis((1_000.0 / context.settings().max_fps) as _)
}

pub struct Visualizer {
    screens: Screens,
    prev_frame_start: Instant,
    context: Context,
}

impl Visualizer {
    pub fn new(settings: Settings) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            screens: Screens::new(rx),
            prev_frame_start: Instant::now(),
            context: Context::new(tx, settings),
        }
    }

    pub fn run(&mut self, start_screen: Box<Screen>) {
        let command_push_start_screen = screen::Command::Push(start_screen);
        self.context.add_command(command_push_start_screen);
        self.screens.handle_commands();
        while self.is_running() {
            self.tick();
        }
    }

    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    fn tick(&mut self) {
        let frame_start = Instant::now();
        let dtime = frame_start.duration_since(self.prev_frame_start);
        self.context.clear();
        self.screens.tick(&mut self.context, dtime);
        self.context.flush();
        self.handle_events();
        let frame_time = Instant::now().duration_since(frame_start);
        let max_frame_time = max_frame_time(&self.context);
        if frame_time < max_frame_time {
            thread::sleep(max_frame_time - frame_time);
        }
        self.prev_frame_start = frame_start;
    }

    fn handle_events(&mut self) {
        for event in self.context.pull_events() {
            self.screens.handle_event(&mut self.context, event);
        }
    }

    fn is_running(&self) -> bool {
        !self.screens.should_close() && !self.context.should_close()
    }
}
