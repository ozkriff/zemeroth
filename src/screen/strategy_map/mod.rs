use std::time::Duration;

use ggez::{
    graphics::{Font, Point2, Text},
    Context,
};
use scene::action::{self, Action, Boxed};
use ui::{self, Gui};

use self::view::{make_action_create_map, View};
use core::{
    map::PosHex,
    strategy_map::{command, execute, state, Id, State},
};
use geom;
use screen::{self, Screen, Transition};
use ZResult;

mod view;
mod visualize;

#[derive(Copy, Clone, Debug)]
enum Message {
    Menu,
    StartBattle, // TODO: remove this button
    EndTurn,
}

fn make_gui(context: &mut Context, font: &Font) -> ZResult<ui::Gui<Message>> {
    let mut gui = ui::Gui::new(context);
    let image_start_battle = Text::new(context, "[start battle]", font)?.into_inner();
    let image_menu = Text::new(context, "[menu]", font)?.into_inner();
    let image_end_turn = Text::new(context, "[end turn]", font)?.into_inner();
    let h = 0.1;
    let button_start_battle =
        ui::Button::new(image_start_battle, h, gui.sender(), Message::StartBattle);
    let button_menu = ui::Button::new(image_menu, h, gui.sender(), Message::Menu);
    let button_end_turn = ui::Button::new(image_end_turn, h, gui.sender(), Message::EndTurn);
    let mut layout = ui::VLayout::new();
    layout.add(Box::new(button_start_battle));
    layout.add(Box::new(button_menu));
    layout.add(Box::new(button_end_turn));
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

    // TODO: start from here:
    // execute::create_objects(state, &mut |state, event, phase| {})

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
    gui: Gui<Message>,
    selected_id: Option<Id>,
    state: State,
    view: View,
}

impl StrategyMap {
    pub fn new(context: &mut Context) -> ZResult<Self> {
        let mut state = State::new();
        let mut view = View::new(&state, context)?;
        prepare_map_and_state(context, &mut state, &mut view)?;
        let gui = make_gui(context, view.font())?;
        Ok(Self {
            gui,
            state,
            view,
            selected_id: None,
        })
    }

    fn handle_event_click(&mut self, context: &mut Context, point: Point2) -> ZResult {
        let pos = geom::point_to_hex(self.view.tile_size(), point);
        // TODO: replace with `info!`:
        println!("pos = {:?}", pos);
        // if self.block_timer.is_some() {
        //     return Ok(());
        // }
        if self.state.map().is_inboard(pos) {
            if let Some(id) = state::agent_id_at_opt(&self.state, pos) {
                self.set_mode(context, id)?;
            } else {
                self.try_move_selected_agent(context, pos);
            }
        }
        Ok(())
    }

    fn do_command_inner(
        &mut self,
        context: &mut Context,
        command: &command::Command,
    ) -> Box<dyn Action> {
        debug!("do_command_inner: {:?}", command);
        let mut actions = Vec::new();
        let state = &mut self.state;
        let view = &mut self.view;
        // TODO: Remove the stuttering:
        execute::execute(state, command, &mut |state, event, phase| {
            let action = visualize::visualize(state, view, context, event, phase)
                .expect("Can't visualize the event");
            actions.push(action);
        }).expect("Can't execute command");
        action::Sequence::new(actions).boxed()
    }

    fn do_command(&mut self, context: &mut Context, command: &command::Command) {
        let action = self.do_command_inner(context, command);
        self.add_action(action);
    }

    fn add_actions(&mut self, actions: Vec<Box<dyn Action>>) {
        self.add_action(action::Sequence::new(actions).boxed());
    }

    fn add_action(&mut self, action: Box<dyn Action>) {
        // self.block_timer = Some(action.duration());
        self.view.add_action(action);
    }

    fn deselect(&mut self) -> ZResult {
        // if let Some(panel) = self.panel_info.take() {
        //     self.gui.remove(&panel)?;
        // }
        // if let Some(panel) = self.panel_abilities.take() {
        //     self.gui.remove(&panel)?;
        // }
        if self.selected_id.is_some() {
            self.view.deselect();
        }
        self.selected_id = None;
        Ok(())
    }

    fn set_mode(&mut self, _: &mut Context, id: Id) -> ZResult {
        self.deselect()?;

        if self.state.parts().agent.get_opt(id).is_none() {
            // This object is not an agent or dead.
            return Ok(());
        }
        self.selected_id = Some(id);
        let state = &self.state;
        let _gui = &mut self.gui; // TODO: use it

        // self.pathfinder.fill_map(state, id);
        // self.panel_info = Some(build_panel_agent_info(context, &self.font, gui, state, id)?);
        // self.panel_abilities = build_panel_agent_abilities(context, &self.font, gui, state, id)?;

        // let map = self.pathfinder.map();
        self.view.set_mode(state, /*map,*/ id)?;
        Ok(())
    }

    fn try_move_selected_agent(&mut self, context: &mut Context, pos: PosHex) {
        if let Some(id) = self.selected_id {
            // let path = match self.pathfinder.path(pos) {
            //     Some(path) => path,
            //     None => return,
            // };
            // assert_eq!(path.from(), self.state.parts().pos.get(id).0);
            // let command_move = command::Command::MoveTo(command::MoveTo { id, path });
            let command_move = command::Command::MoveTo(command::MoveTo { id, to: pos });
            if command::check(&self.state, &command_move).is_err() {
                return;
            }
            self.do_command(context, &command_move);
            // self.fill_map();
        }
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
            Some(Message::EndTurn) => {
                unimplemented!("END TURN"); // TODO:
            }
            None => self.handle_event_click(context, pos)?,
        }
        Ok(Transition::None)
    }
}
