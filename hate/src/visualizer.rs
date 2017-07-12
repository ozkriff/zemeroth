use std::thread;
use std::sync::mpsc;
use screen;
use ::{Time, Screen, Context, Settings};
use screen_stack::Screens;

#[cfg(not(target_os = "android"))]
fn check_assets_dir() {
    use std::fs;
    use std::process;

    // TODO: check assets version
    if let Err(e) = fs::metadata("assets") {
        println!("Can`t find 'assets' dir: {}", e);
        println!("Note: see 'Assets' section of README.rst");
        process::exit(1);
    }
}

#[cfg(target_os = "android")]
fn check_assets_dir() {}

fn max_frame_time(context: &Context) -> Time {
    Time(1.0 / context.settings().max_fps)
}

pub struct Visualizer {
    screens: Screens,
    prev_frame_start: Time,
    context: Context,
}

impl Visualizer {
    pub fn new(settings: Settings) -> Self {
        check_assets_dir();
        let (tx, rx) = mpsc::channel();
        let context = Context::new(tx, settings);
        let screens = Screens::new(rx);
        let prev_frame_start = context.now();
        Self {
            screens,
            prev_frame_start,
            context,
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
        let frame_start = self.context.now();
        let dtime = frame_start.delta(self.prev_frame_start);
        self.context.clear();
        self.screens.tick(&mut self.context, dtime);
        self.context.flush();
        self.handle_events();
        let frame_time = self.context.now().delta(frame_start);
        let remainder = max_frame_time(&self.context).delta(frame_time);
        if remainder > Time(0.0) {
            thread::sleep(remainder.to_duration());
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
