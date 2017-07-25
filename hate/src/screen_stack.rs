use std::sync::mpsc;
use context::Context;
use time::Time;
use event::Event;
use screen::{Command, Screen};

pub struct Screens {
    screens: Vec<Box<Screen>>,
    rx: mpsc::Receiver<Command>,
}

impl Screens {
    pub fn new(rx: mpsc::Receiver<Command>) -> Self {
        let screens = Vec::new();
        Self { screens, rx }
    }

    pub fn should_close(&self) -> bool {
        self.screens.is_empty()
    }

    pub fn tick(&mut self, context: &mut Context, dtime: Time) {
        self.screens.last_mut().unwrap().tick(context, dtime);
        self.handle_commands();
    }

    pub fn handle_commands(&mut self) {
        while let Ok(command) = self.rx.try_recv() {
            match command {
                Command::Push(screen) => {
                    self.screens.push(screen);
                }
                Command::Pop => {
                    self.screens.pop().unwrap();
                }
            }
        }
    }

    pub fn handle_event(&mut self, context: &mut Context, event: Event) {
        let screen = self.screens.last_mut().unwrap();
        screen.handle_event(context, event);
    }
}
