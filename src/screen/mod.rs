use std::fmt::Debug;
use std::time::Duration;

use ggez::graphics::{self, Point2};
use ggez::{self, Context};

use ZResult;

mod battle;
mod camp;
mod campaign_menu;
mod main_menu;

pub use self::battle::Battle;
pub use self::camp::Camp;
pub use self::campaign_menu::CampaignMenu;
pub use self::main_menu::MainMenu;

#[derive(Debug)]
pub enum Transition {
    None,
    Push(Box<dyn Screen>),
    Pop,
}

pub trait Screen: Debug {
    fn update(&mut self, context: &mut Context, dtime: Duration) -> ZResult<Transition>;
    fn draw(&self, context: &mut Context) -> ZResult;
    fn click(&mut self, context: &mut Context, pos: Point2) -> ZResult<Transition>;
    fn resize(&mut self, aspect_ratio: f32);
}

const ERR_MSG: &str = "Screen stack is empty";

pub struct Screens {
    screens: Vec<Box<dyn Screen>>,
}

impl Screens {
    pub fn new(start_screen: Box<dyn Screen>) -> Self {
        Self {
            screens: vec![start_screen],
        }
    }

    pub fn update(&mut self, context: &mut Context) -> ZResult {
        let dtime = ggez::timer::get_delta(context);
        let command = self.screen_mut().update(context, dtime)?;
        self.handle_command(context, command)
    }

    pub fn draw(&self, context: &mut Context) -> ZResult {
        graphics::set_background_color(context, [0.9, 0.9, 0.8, 1.0].into());
        graphics::clear(context);
        self.screen().draw(context)?;
        graphics::present(context);
        Ok(())
    }

    pub fn click(&mut self, context: &mut Context, pos: Point2) -> ZResult {
        let command = self.screen_mut().click(context, pos)?;
        self.handle_command(context, command)
    }

    pub fn resize(&mut self, aspect_ratio: f32) {
        for screen in &mut self.screens {
            screen.resize(aspect_ratio);
        }
    }

    pub fn handle_command(&mut self, context: &mut Context, command: Transition) -> ZResult {
        match command {
            Transition::None => {}
            Transition::Push(screen) => {
                info!("Screens::handle_command: Push");
                self.screens.push(screen);
            }
            Transition::Pop => {
                info!("Screens::handle_command: Pop");
                if self.screens.len() > 1 {
                    self.screens.pop().expect(ERR_MSG);
                } else {
                    context.quit()?;
                }
            }
        }
        Ok(())
    }

    /// Returns a mutable reference to the top screen.
    fn screen_mut(&mut self) -> &mut dyn Screen {
        &mut **self.screens.last_mut().expect(ERR_MSG)
    }

    /// Returns a reference to the top screen.
    fn screen(&self) -> &dyn Screen {
        &**self.screens.last().expect(ERR_MSG)
    }
}
