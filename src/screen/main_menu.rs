use std::time::Duration;

use ggez::graphics::{Font, Point2, Text};
use ggez::Context;
use scene::Sprite;
use ui::{self, Gui};

use screen::{self, Screen, Transition};
use ZResult;

#[derive(Copy, Clone, Debug)]
enum Message {
    Exit,
    StartInstant,
    StartCampaign,
}

fn make_gui(context: &mut Context, font: &Font) -> ZResult<ui::Gui<Message>> {
    let mut gui = ui::Gui::new(context);
    let image_battle = Text::new(context, "[battle]", font)?.into_inner();
    let image_campaign = Text::new(context, "[campaign]", font)?.into_inner();
    let image_exit = Text::new(context, "[exit]", font)?.into_inner();
    let button_battle = ui::Button::new(image_battle, 0.2, gui.sender(), Message::StartInstant);
    let button_campaign =
        ui::Button::new(image_campaign, 0.2, gui.sender(), Message::StartCampaign);
    let button_exit = ui::Button::new(image_exit, 0.2, gui.sender(), Message::Exit);
    let mut layout = ui::VLayout::new();
    layout.add(Box::new(button_battle));
    layout.add(Box::new(button_campaign));
    layout.add(Box::new(button_exit));
    let anchor = ui::Anchor(ui::HAnchor::Middle, ui::VAnchor::Middle);
    gui.add(&ui::pack(layout), anchor);
    Ok(gui)
}

#[derive(Debug)]
pub struct MainMenu {
    gui: Gui<Message>,
}

impl MainMenu {
    pub fn new(context: &mut Context) -> ZResult<Self> {
        let font = Font::new(context, "/OpenSans-Regular.ttf", 32)?;
        let gui = make_gui(context, &font)?;

        let mut sprite = Sprite::from_path(context, "/tile.png", 0.1)?;
        sprite.set_centered(true);
        sprite.set_pos(Point2::new(0.5, 0.5));

        // TODO: create some random agent arc-moving animation
        Ok(Self { gui })
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

    fn click(&mut self, context: &mut Context, pos: Point2) -> ZResult<Transition> {
        let message = self.gui.click(pos);
        debug!("MainMenu: click: pos={:?}, message={:?}", pos, message);
        match message {
            Some(Message::StartInstant) => {
                let screen = screen::Battle::new(context)?;
                Ok(Transition::Push(Box::new(screen)))
            }
            Some(Message::StartCampaign) => {
                let screen = screen::StrategyMap::new(context)?;
                Ok(Transition::Push(Box::new(screen)))
            }
            Some(Message::Exit) => Ok(Transition::Pop),
            None => Ok(Transition::None),
        }
    }
}
