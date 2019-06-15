use log::info;
use std::{fmt::Debug, time::Duration};

use ggez::{self, graphics, nalgebra::Point2, Context};

use crate::ZResult;

mod agent_info;
mod battle;
mod campaign;
mod main_menu;
mod strategy_map;

pub use self::{
    agent_info::AgentInfo, battle::Battle, campaign::Campaign, main_menu::MainMenu,
    strategy_map::StrategyMap,
};

#[derive(Debug)]
pub enum Transition {
    None,
    Push(Box<dyn Screen>),
    Pop,
}

pub trait Screen: Debug {
    fn update(&mut self, context: &mut Context, dtime: Duration) -> ZResult<Transition>;
    fn draw(&self, context: &mut Context) -> ZResult;
    fn click(&mut self, context: &mut Context, pos: Point2<f32>) -> ZResult<Transition>;
    fn resize(&mut self, aspect_ratio: f32);

    fn move_mouse(&mut self, _context: &mut Context, _pos: Point2<f32>) -> ZResult {
        Ok(())
    }
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
        let dtime = ggez::timer::delta(context);
        let command = self.screen_mut().update(context, dtime)?;
        self.handle_command(context, command)
    }

    pub fn draw(&self, context: &mut Context) -> ZResult {
        let bg_color = [0.9, 0.9, 0.8, 1.0].into();
        graphics::clear(context, bg_color);
        self.screen().draw(context)?;
        graphics::present(context)?;
        Ok(())
    }

    pub fn click(&mut self, context: &mut Context, pos: Point2<f32>) -> ZResult {
        let command = self.screen_mut().click(context, pos)?;
        self.handle_command(context, command)
    }

    pub fn move_mouse(&mut self, context: &mut Context, pos: Point2<f32>) -> ZResult {
        self.screen_mut().move_mouse(context, pos)
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
                    #[cfg(not(target_arch = "wasm32"))] // we can't quit wasm anyway
                    ggez::quit(context);
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
