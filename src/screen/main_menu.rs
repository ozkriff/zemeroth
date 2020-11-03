use std::{
    sync::mpsc::{channel, Receiver},
    time::Duration,
};

use macroquad::prelude::{Color, Font, Vec2};

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

fn make_gui(font: Font) -> ZResult<ui::Gui<Message>> {
    let mut gui = ui::Gui::new();
    let h = utils::line_heights().large;
    let font_size = utils::font_size();
    let space = || Box::new(ui::Spacer::new_vertical(h / 8.0));
    let button = &mut |text, message| -> ZResult<_> {
        let text = ui::Drawable::text(text, font, font_size);
        let b = ui::Button::new(text, h, gui.sender(), message)?.stretchable(true);
        Ok(Box::new(b))
    };
    let mut layout = Box::new(ui::VLayout::new().stretchable(true));
    layout.add(button("demo battle", Message::StartInstant)?);
    layout.add(space());
    layout.add(button("campaign", Message::StartCampaign)?);
    #[cfg(not(target_arch = "wasm32"))] // can't quit WASM
    {
        layout.add(space());
        layout.add(button("exit", Message::Exit)?);
    }
    layout.stretch_to_self()?;
    let layout = utils::add_offsets_and_bg_big(layout)?;
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
    pub async fn new() -> ZResult<Self> {
        let font = utils::default_font();
        let gui = make_gui(font)?;
        let mut sprite = Sprite::from_path("assets/img/tile.png", 0.1).await?;
        sprite.set_centered(true);
        sprite.set_pos(Vec2::new(0.5, 0.5));
        Ok(Self {
            gui,
            receiver_battle_result: None,
        })
    }
}

impl Screen for MainMenu {
    fn update(&mut self, _: Duration) -> ZResult<StackCommand> {
        Ok(StackCommand::None)
    }

    fn draw(&self) -> ZResult {
        self.gui.draw();
        Ok(())
    }

    fn click(&mut self, pos: Vec2) -> ZResult<StackCommand> {
        let message = self.gui.click(pos);
        trace!("MainMenu: click: pos={:?}, message={:?}", pos, message);
        match message {
            Some(Message::StartInstant) => {
                let screen = async {
                    let scenario = utils::deserialize_from_file("assets/scenario_01.ron")
                        .await
                        .unwrap();
                    let (sender, receiver) = channel();
                    self.receiver_battle_result = Some(receiver);
                    let proto = Prototypes::from_str(
                        &utils::read_file("assets/objects.ron").await.unwrap(),
                    );
                    let battle_type = scenario::BattleType::Skirmish;

                    let screen = screen::Battle::new(scenario, battle_type, proto, sender)
                        .await
                        .unwrap();
                    unsafe { crate::screen::SCREEN_HACK = Some(Box::new(screen)) };
                };
                let coroutine = unsafe { macroquad::coroutines::start_coroutine(screen) };
                Ok(StackCommand::PushScreen(coroutine))
            }
            Some(Message::StartCampaign) => {
                // let screen = screen::Campaign::new()?;
                // Ok(StackCommand::PushScreen(Box::new(screen)))
                unimplemented!()
            }
            Some(Message::Exit) => Ok(StackCommand::Pop),
            None => Ok(StackCommand::None),
        }
    }

    fn resize(&mut self, aspect_ratio: f32) {
        self.gui.resize(aspect_ratio);
    }

    fn move_mouse(&mut self, pos: Vec2) -> ZResult {
        self.gui.move_mouse(pos);
        Ok(())
    }
}
