use std::{
    sync::mpsc::{channel, Receiver},
    time::Duration,
};

use gwg::{
    graphics::{Font, Point2, Text},
    Context,
};
use log::trace;
use ui::{self, Gui, Widget};
use zscene::Sprite;

use crate::{
    core::battle::{component::Prototypes, scenario, state},
    screen::{self, Screen, StackCommand},
    utils, ZResult,
};

#[derive(Copy, Clone, Debug)]
enum Message {
    Exit,
    StartInstant,
    StartCampaign,
}

fn make_gui(context: &mut Context, font: Font) -> ZResult<ui::Gui<Message>> {
    let mut gui = ui::Gui::new(context);
    let h = utils::line_heights().large;
    let font_size = utils::font_size();
    let space = || Box::new(ui::Spacer::new_vertical(h / 8.0));
    let button = &mut |context: &mut Context, text, message| -> ZResult<_> {
        let text = Box::new(Text::new((text, font, font_size)));
        let b = ui::Button::new(context, text, h, gui.sender(), message)?.stretchable(true);
        Ok(Box::new(b))
    };
    let mut layout = Box::new(ui::VLayout::new().stretchable(true));
    layout.add(button(context, "demo battle", Message::StartInstant)?);
    layout.add(space());
    layout.add(button(context, "campaign", Message::StartCampaign)?);
    #[cfg(not(target_arch = "wasm32"))] // can't quit WASM
    {
        layout.add(space());
        layout.add(button(context, "exit", Message::Exit)?);
    }
    layout.stretch_to_self(context)?;
    let layout = utils::add_offsets_and_bg_big(context, layout)?;
    let anchor = ui::Anchor(ui::HAnchor::Middle, ui::VAnchor::Middle);
    gui.add(&ui::pack(layout), anchor);
    Ok(gui)
}

#[derive(Debug)]
pub struct MainMenu {
    gui: Gui<Message>,
    receiver_battle_result: Option<Receiver<Option<state::BattleResult>>>,
}

// TODO: add the game's version to one of the corners
impl MainMenu {
    pub fn new(context: &mut Context) -> ZResult<Self> {
        let font = utils::default_font(context);
        let gui = make_gui(context, font)?;
        let mut sprite = Sprite::from_path(context, "/tile.png", 0.1)?;
        sprite.set_centered(true);
        sprite.set_pos(Point2::new(0.5, 0.5));
        Ok(Self {
            gui,
            receiver_battle_result: None,
        })
    }
}

impl Screen for MainMenu {
    fn update(&mut self, _context: &mut Context, _: Duration) -> ZResult<StackCommand> {
        Ok(StackCommand::None)
    }

    fn draw(&self, context: &mut Context) -> ZResult {
        self.gui.draw(context)?;
        Ok(())
    }

    fn click(&mut self, context: &mut Context, pos: Point2) -> ZResult<StackCommand> {
        let message = self.gui.click(pos);
        trace!("MainMenu: click: pos={:?}, message={:?}", pos, message);
        match message {
            Some(Message::StartInstant) => {
                let scenario = utils::deserialize_from_file(context, "/scenario_01.ron")?;
                let (sender, receiver) = channel();
                self.receiver_battle_result = Some(receiver);
                let proto = Prototypes::from_str(&utils::read_file(context, "/objects.ron")?);
                let battle_type = scenario::BattleType::Skirmish;
                let screen = screen::Battle::new(context, scenario, battle_type, proto, sender)?;
                Ok(StackCommand::PushScreen(Box::new(screen)))
            }
            Some(Message::StartCampaign) => {
                let screen = screen::Campaign::new(context)?;
                Ok(StackCommand::PushScreen(Box::new(screen)))
            }
            Some(Message::Exit) => Ok(StackCommand::Pop),
            None => Ok(StackCommand::None),
        }
    }

    fn resize(&mut self, aspect_ratio: f32) {
        self.gui.resize(aspect_ratio);
    }

    fn move_mouse(&mut self, _context: &mut Context, pos: Point2) -> ZResult {
        self.gui.move_mouse(pos);
        Ok(())
    }
}
