use log::info;
use std::{fmt::Debug, time::Duration};

use ggez::{
    self,
    graphics::{self, Color},
    Context,
};
use nalgebra::Point2;

use crate::ZResult;

mod ability_info;
mod agent_info;
mod battle;
mod campaign;
mod confirm;
mod main_menu;

pub use self::{
    ability_info::AbilityInfo, agent_info::AgentInfo, battle::Battle, campaign::Campaign,
    confirm::Confirm, main_menu::MainMenu,
};

const COLOR_SCREEN_BG: Color = Color::new(0.9, 0.9, 0.8, 1.0);
const COLOR_POPUP_BG: Color = Color::new(0.9, 0.9, 0.8, 0.8);

#[derive(Debug)]
pub enum StackCommand {
    None,
    PushScreen(Box<dyn Screen>),
    PushPopup(Box<dyn Screen>),
    Pop,
}

pub trait Screen: Debug {
    fn update(&mut self, context: &mut Context, dtime: Duration) -> ZResult<StackCommand>;
    fn draw(&self, context: &mut Context) -> ZResult;
    fn click(&mut self, context: &mut Context, pos: Point2<f32>) -> ZResult<StackCommand>;
    fn resize(&mut self, aspect_ratio: f32);

    fn move_mouse(&mut self, _context: &mut Context, _pos: Point2<f32>) -> ZResult {
        Ok(())
    }
}

const ERR_MSG_STACK_EMPTY: &str = "Screen stack is empty";

struct ScreenWithPopups {
    screen: Box<dyn Screen>,
    popups: Vec<Box<dyn Screen>>,
}

impl ScreenWithPopups {
    fn new(screen: Box<dyn Screen>) -> Self {
        Self {
            screen,
            popups: Vec::new(),
        }
    }

    fn top_mut(&mut self) -> &mut dyn Screen {
        match self.popups.last_mut() {
            Some(popup) => popup.as_mut(),
            None => self.screen.as_mut(),
        }
    }
}

fn make_popup_bg_mesh(context: &mut Context) -> ZResult<graphics::Mesh> {
    let coords = graphics::screen_coordinates(context);
    let mode = graphics::DrawMode::fill();
    Ok(graphics::Mesh::new_rectangle(
        context,
        mode,
        coords,
        COLOR_POPUP_BG,
    )?)
}

pub struct Screens {
    screens: Vec<ScreenWithPopups>,
    popup_bg_mesh: graphics::Mesh,
}

impl Screens {
    pub fn new(context: &mut Context, start_screen: Box<dyn Screen>) -> ZResult<Self> {
        Ok(Self {
            screens: vec![ScreenWithPopups::new(start_screen)],
            popup_bg_mesh: make_popup_bg_mesh(context)?,
        })
    }

    pub fn update(&mut self, context: &mut Context) -> ZResult {
        let dtime = ggez::timer::delta(context);
        let command = self.screen_mut().top_mut().update(context, dtime)?;
        self.handle_command(context, command)
    }

    pub fn draw(&self, context: &mut Context) -> ZResult {
        graphics::clear(context, COLOR_SCREEN_BG);
        let screen = self.screen();
        screen.screen.draw(context)?;
        for popup in &screen.popups {
            graphics::draw(context, &self.popup_bg_mesh, graphics::DrawParam::default())?;
            popup.draw(context)?;
        }
        graphics::present(context)?;
        Ok(())
    }

    pub fn click(&mut self, context: &mut Context, pos: Point2<f32>) -> ZResult {
        let command = self.screen_mut().top_mut().click(context, pos)?;
        self.handle_command(context, command)
    }

    pub fn move_mouse(&mut self, context: &mut Context, pos: Point2<f32>) -> ZResult {
        self.screen_mut().top_mut().move_mouse(context, pos)
    }

    pub fn resize(&mut self, context: &mut Context, aspect_ratio: f32) -> ZResult {
        self.popup_bg_mesh = make_popup_bg_mesh(context)?;
        for screen in &mut self.screens {
            screen.screen.resize(aspect_ratio);
            for popup in &mut screen.popups {
                popup.resize(aspect_ratio);
            }
        }
        Ok(())
    }

    pub fn handle_command(&mut self, context: &mut Context, command: StackCommand) -> ZResult {
        match command {
            StackCommand::None => {}
            StackCommand::PushScreen(screen) => {
                info!("Screens::handle_command: PushScreen");
                self.screens.push(ScreenWithPopups::new(screen));
            }
            StackCommand::Pop => {
                info!("Screens::handle_command: Pop");
                let popups = &mut self.screen_mut().popups;
                if !popups.is_empty() {
                    popups.pop().expect(ERR_MSG_STACK_EMPTY);
                } else if self.screens.len() > 1 {
                    self.screens.pop().expect(ERR_MSG_STACK_EMPTY);
                } else {
                    #[cfg(not(target_arch = "wasm32"))] // we can't quit wasm anyway
                    ggez::event::quit(context);
                }
            }
            StackCommand::PushPopup(screen) => {
                info!("Screens::handle_command: PushPopup");
                self.screen_mut().popups.push(screen);
            }
        }
        Ok(())
    }

    /// Returns a mutable reference to the top screen.
    fn screen_mut(&mut self) -> &mut ScreenWithPopups {
        self.screens.last_mut().expect(ERR_MSG_STACK_EMPTY)
    }

    /// Returns a reference to the top screen.
    fn screen(&self) -> &ScreenWithPopups {
        self.screens.last().expect(ERR_MSG_STACK_EMPTY)
    }
}
