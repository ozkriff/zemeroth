use std::{fmt::Debug, time::Duration};

use log::info;
use macroquad::experimental as mq;
use mq::{graphics::Color, math::Vec2, Rect};

use crate::{utils, ZResult};

mod agent_info;
mod battle;
mod campaign;
mod confirm;
mod general_info;
mod main_menu;

pub use self::{
    agent_info::AgentInfo, battle::Battle, campaign::Campaign, confirm::Confirm,
    general_info::GeneralInfo, main_menu::MainMenu,
};

pub const COLOR_SCREEN_BG: Color = Color::new_const(229, 229, 204, 255);
pub const COLOR_POPUP_BG: Color = Color::new_const(229, 229, 204, 229);

#[derive(Debug)]
pub enum StackCommand {
    None,
    PushScreen(Box<dyn Screen>),
    PushPopup(Box<dyn Screen>),
    Pop,
}

pub trait Screen: Debug {
    fn update(&mut self, dtime: Duration) -> ZResult<StackCommand>;
    fn draw(&self) -> ZResult;
    fn click(&mut self, pos: Vec2) -> ZResult<StackCommand>;
    fn resize(&mut self, aspect_ratio: f32);

    fn move_mouse(&mut self, _pos: Vec2) -> ZResult {
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

pub struct ScreenStack {
    screens: Vec<ScreenWithPopups>,
}

impl ScreenStack {
    pub fn new(start_screen: Box<dyn Screen>) -> ZResult<Self> {
        Ok(Self {
            screens: vec![ScreenWithPopups::new(start_screen)],
        })
    }

    pub fn update(&mut self, dtime: Duration) -> ZResult {
        let command = self.screen_mut().top_mut().update(dtime)?;
        self.handle_command(command)
    }

    pub fn draw(&self) -> ZResult {
        let screen = self.screen();
        screen.screen.draw()?;
        for popup in &screen.popups {
            self.draw_popup_bg();
            popup.draw()?;
        }
        Ok(())
    }

    pub fn click(&mut self, pos: Vec2) -> ZResult {
        let command = self.screen_mut().top_mut().click(pos)?;
        self.handle_command(command)
    }

    pub fn move_mouse(&mut self, pos: Vec2) -> ZResult {
        self.screen_mut().top_mut().move_mouse(pos)
    }

    pub fn resize(&mut self, aspect_ratio: f32) -> ZResult {
        for screen in &mut self.screens {
            screen.screen.resize(aspect_ratio);
            for popup in &mut screen.popups {
                popup.resize(aspect_ratio);
            }
        }
        Ok(())
    }

    pub fn handle_command(&mut self, command: StackCommand) -> ZResult {
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
                    std::process::exit(0);
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

    fn draw_popup_bg(&self) {
        let aspect_ratio = utils::aspect_ratio();
        let r = Rect::new(-aspect_ratio, -1.0, aspect_ratio * 2.0, 2.0);
        mq::shapes::draw_rectangle(r.x, r.y, r.w, r.h, COLOR_POPUP_BG);
    }
}
