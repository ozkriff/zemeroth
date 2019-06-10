use std::{
    sync::mpsc::{channel, Receiver},
    time::Duration,
};

use ggez::{
    graphics::{self, Font, Text},
    nalgebra::Point2,
    Context,
};
use log::info;
use scene::{Layer, Scene, Sprite};
use ui::{self, Gui};

use crate::{
    core::battle::state,
    screen::{self, Screen, Transition},
    utils, ZResult,
};

#[derive(Copy, Clone, Debug)]
enum Message {
    Menu,
    StartBattle,
}

fn make_gui(context: &mut Context, font: Font) -> ZResult<ui::Gui<Message>> {
    let mut gui = ui::Gui::new(context);
    let h = utils::line_heights().large;
    let font_size = utils::font_size();
    let button_start_battle = {
        let text = Box::new(Text::new(("[start battle]", font, font_size)));
        ui::Button::new(context, text, h, gui.sender(), Message::StartBattle)?
    };
    let button_menu = {
        let text = Box::new(Text::new(("[menu]", font, font_size)));
        ui::Button::new(context, text, h, gui.sender(), Message::Menu)?
    };
    let mut layout = ui::VLayout::new();
    layout.add(Box::new(button_start_battle));
    layout.add(Box::new(button_menu));
    let anchor = ui::Anchor(ui::HAnchor::Left, ui::VAnchor::Top);
    gui.add(&ui::pack(layout), anchor);
    Ok(gui)
}

#[derive(Debug, Clone, Default)]
struct Layers {
    fg: Layer,
}

impl Layers {
    fn sorted(self) -> Vec<Layer> {
        vec![self.fg]
    }
}

#[derive(Debug)]
pub struct StrategyMap {
    font: graphics::Font,
    gui: Gui<Message>,

    sprite: Sprite,
    scene: Scene,
    layers: Layers,

    receiver: Option<Receiver<state::BattleResult>>,
}

impl StrategyMap {
    pub fn new(context: &mut Context) -> ZResult<Self> {
        let font = utils::default_font(context);
        let gui = make_gui(context, font)?;

        let mut sprite = Sprite::from_path(context, "/tile.png", 0.1)?;
        sprite.set_centered(true);
        sprite.set_pos(Point2::new(0.5, 0.5));

        let layers = Layers::default();
        let scene = Scene::new(layers.clone().sorted());

        Ok(Self {
            gui,
            font,
            sprite,
            scene,
            layers,
            receiver: None,
        })
    }
}

impl Screen for StrategyMap {
    fn update(&mut self, _context: &mut Context, dtime: Duration) -> ZResult<Transition> {
        self.scene.tick(dtime);
        Ok(Transition::None)
    }

    fn draw(&self, context: &mut Context) -> ZResult {
        self.sprite.draw(context)?;
        self.scene.draw(context)?;
        self.gui.draw(context)?;
        Ok(())
    }

    fn resize(&mut self, aspect_ratio: f32) {
        self.gui.resize(aspect_ratio);
    }

    fn click(&mut self, context: &mut Context, pos: Point2<f32>) -> ZResult<Transition> {
        let message = self.gui.click(pos);
        info!(
            "StrategyScreen: click: pos={:?}, message={:?}",
            pos, message
        );
        match message {
            Some(Message::StartBattle) => {
                let scenario = utils::deserialize_from_file(context, "/scenario_01.ron")?;
                let (sender, receiver) = channel();
                self.receiver = Some(receiver);
                let screen = screen::Battle::new(context, scenario, sender)?;
                Ok(Transition::Push(Box::new(screen)))
            }
            Some(Message::Menu) => Ok(Transition::Pop),
            None => Ok(Transition::None),
        }
    }

    fn move_mouse(&mut self, _context: &mut Context, pos: Point2<f32>) -> ZResult {
        self.gui.move_mouse(pos);
        Ok(())
    }
}
