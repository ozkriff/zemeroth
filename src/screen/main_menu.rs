use std::{
    sync::mpsc::{channel, Receiver},
    time::Duration,
};

use ggez::{
    graphics::{Font, Text},
    nalgebra::Point2,
    Context,
};
use log::debug;
use scene::Sprite;
use ui::{self, Gui};

use crate::{
    core::tactical_map::state,
    screen::{self, Screen, Transition},
    utils, ZResult,
};

#[derive(Copy, Clone, Debug)]
enum Message {
    Exit,
    StartInstant,
    StartCampaign,
    StartStrategyMap,
}

fn make_gui(context: &mut Context, font: Font) -> ZResult<ui::Gui<Message>> {
    let mut gui = ui::Gui::new(context);
    let h = 0.2;
    let font_size = utils::font_size();
    let button_battle = {
        let text = Box::new(Text::new(("[demo battle]", font, font_size)));
        ui::Button::new(context, text, h, gui.sender(), Message::StartInstant)
    };
    let button_campaign = {
        let text = Box::new(Text::new(("[campaign]", font, font_size)));
        ui::Button::new(context, text, h, gui.sender(), Message::StartCampaign)
    };
    let button_strategy_map = {
        let text = Box::new(Text::new(("[strategy mode]", font, font_size)));
        ui::Button::new(context, text, h, gui.sender(), Message::StartStrategyMap)
    };
    let button_exit = {
        let text = Box::new(Text::new(("[exit]", font, font_size)));
        ui::Button::new(context, text, h, gui.sender(), Message::Exit)
    };
    let mut layout = ui::VLayout::new();
    layout.add(Box::new(button_battle));
    layout.add(Box::new(button_campaign));
    layout.add(Box::new(button_strategy_map));
    layout.add(Box::new(button_exit));
    let anchor = ui::Anchor(ui::HAnchor::Middle, ui::VAnchor::Middle);
    gui.add(&ui::pack(layout), anchor);
    Ok(gui)
}

#[derive(Debug)]
pub struct MainMenu {
    gui: Gui<Message>,

    receiver: Option<Receiver<state::BattleResult>>,
}

impl MainMenu {
    pub fn new(context: &mut Context) -> ZResult<Self> {
        let font = utils::default_font(context);
        let gui = make_gui(context, font)?;

        let mut sprite = Sprite::from_path(context, "/tile.png", 0.1)?;
        sprite.set_centered(true);
        sprite.set_pos(Point2::new(0.5, 0.5));

        // TODO: create some random agent arc-moving animation
        Ok(Self {
            gui,
            receiver: None,
        })
    }
}

impl Screen for MainMenu {
    fn update(&mut self, _context: &mut Context, _: Duration) -> ZResult<Transition> {
        Ok(Transition::None)
    }

    fn draw(&self, context: &mut Context) -> ZResult {
        self.gui.draw(context)
    }

    fn resize(&mut self, aspect_ratio: f32) {
        self.gui.resize(aspect_ratio);
    }

    fn click(&mut self, context: &mut Context, pos: Point2<f32>) -> ZResult<Transition> {
        let message = self.gui.click(pos);
        debug!("MainMenu: click: pos={:?}, message={:?}", pos, message);
        match message {
            Some(Message::StartInstant) => {
                let scenario = utils::deserialize_from_file(context, "/scenario_01.ron")?;
                let (sender, receiver) = channel();
                self.receiver = Some(receiver);
                let screen = screen::Battle::new(context, scenario, sender)?;
                Ok(Transition::Push(Box::new(screen)))
            }
            Some(Message::StartCampaign) => {
                let screen = screen::Campaign::new(context)?;
                Ok(Transition::Push(Box::new(screen)))
            }
            Some(Message::StartStrategyMap) => {
                let screen = screen::StrategyMap::new(context)?;
                Ok(Transition::Push(Box::new(screen)))
            }
            Some(Message::Exit) => Ok(Transition::Pop),
            None => Ok(Transition::None),
        }
    }
}
