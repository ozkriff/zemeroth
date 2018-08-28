use std::time::Duration;

use ggez::graphics::{self, Font, Point2, Text};
use ggez::Context;
// use scene::{Layer, Scene, Sprite};
use scene::action::{self, Boxed};
use ui::{self, Gui};

use self::view::{make_action_create_map, View};
use core::strategy_map::State;
use geom;
use screen::{self, Screen, Transition};
use ZResult;

mod view;

#[derive(Copy, Clone, Debug)]
enum Message {
    Menu,
    StartBattle,
}

fn make_gui(context: &mut Context, font: &Font) -> ZResult<ui::Gui<Message>> {
    let mut gui = ui::Gui::new(context);
    let image_start_battle = Text::new(context, "[start battle]", font)?.into_inner();
    let image_menu = Text::new(context, "[menu]", font)?.into_inner();
    let button_start_battle =
        ui::Button::new(image_start_battle, 0.2, gui.sender(), Message::StartBattle);
    let button_menu = ui::Button::new(image_menu, 0.2, gui.sender(), Message::Menu);
    let mut layout = ui::VLayout::new();
    layout.add(Box::new(button_start_battle));
    layout.add(Box::new(button_menu));
    let anchor = ui::Anchor(ui::HAnchor::Left, ui::VAnchor::Top);
    gui.add(&ui::pack(layout), anchor);
    Ok(gui)
}

fn prepare_map_and_state(
    /*context*/ _: &mut Context,
    state: &mut State,
    view: &mut View,
) -> ZResult {
    let mut actions = Vec::new();
    // execute::create_terrain(state);
    actions.push(make_action_create_map(state, view)?);
    // execute::create_objects(state, &mut |state, event, phase| {
    //     let action = visualize::visualize(state, view, context, event, phase)
    //         .expect("Can't visualize the event");
    //     let action = action::Fork::new(action).boxed();
    //     actions.push(action);
    // });
    view.add_action(action::Sequence::new(actions).boxed());
    Ok(())
}

#[derive(Debug)]
pub struct StrategyMap {
    font: graphics::Font,
    gui: Gui<Message>,

    state: State,
    view: View,
    // sprite: Sprite,
    // scene: Scene,
    // layers: Layers,
}

impl StrategyMap {
    pub fn new(context: &mut Context) -> ZResult<Self> {
        let font = Font::new(context, "/OpenSans-Regular.ttf", 32)?;
        let gui = make_gui(context, &font)?;

        let mut state = State::new();

        // let mut sprite = Sprite::from_path(context, "/tile.png", 0.1)?;
        // sprite.set_centered(true);
        // sprite.set_pos(Point2::new(0.5, 0.5));

        // let layers = Layers::default();
        // let scene = Scene::new(layers.clone().sorted());

        let mut view = View::new(&state, context)?;

        prepare_map_and_state(context, &mut state, &mut view)?;

        Ok(Self {
            gui,
            font,
            state,
            view,
            // sprite,
            // scene,
            // layers,
        })
    }

    fn handle_event_click(&mut self, _: &mut Context, point: Point2) -> ZResult {
        let pos = geom::point_to_hex(self.view.tile_size(), point);
        println!("pos = {:?}", pos);
        // if self.block_timer.is_some() {
        //     return Ok(());
        // }
        if self.state.map().is_inboard(pos) {
            println!("inboard");
            // if let Some(id) = state::agent_id_at_opt(&self.state, pos) {
            //     self.handle_agent_click(context, id)?;
            // } else {
            //     self.try_move_selected_agent(context, pos);
            // }
        }
        Ok(())
    }
}

impl Screen for StrategyMap {
    fn update(&mut self, _context: &mut Context, dtime: Duration) -> ZResult<Transition> {
        self.view.tick(dtime);
        Ok(Transition::None)
    }

    fn draw(&self, context: &mut Context) -> ZResult {
        self.view.draw(context)?;
        self.gui.draw(context)
    }

    fn resize(&mut self, aspect_ratio: f32) {
        self.gui.resize(aspect_ratio);
    }

    fn click(&mut self, context: &mut Context, pos: Point2) -> ZResult<Transition> {
        let message = self.gui.click(pos);
        info!(
            "StrategyScreen: click: pos={:?}, message={:?}",
            pos, message
        );
        match message {
            Some(Message::StartBattle) => {
                let screen = screen::Battle::new(context)?;
                return Ok(Transition::Push(Box::new(screen)));
            }
            Some(Message::Menu) => return Ok(Transition::Pop),
            None => self.handle_event_click(context, pos)?,
        }
        Ok(Transition::None)
    }
}
