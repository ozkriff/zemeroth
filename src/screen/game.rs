use ron;
use rand::{thread_rng, Rng};
use cgmath::Vector2;
use hate::{self, Context, Event, Screen, Sprite, Time};
use hate::geom::Point;
use hate::gui::{self, Gui};
use hate::scene::action::{self, Action};
use visualize;
use map;
use game_view::GameView;
use ai::Ai;
use core::{self, check, Jokers, Moves, ObjId, PlayerId, State, TileType};
use core::command;
use core::execute;
use core::map::PosHex;
use core::movement::Pathfinder;

#[derive(Copy, Clone, Debug)]
enum GuiCommand {
    Exit,
    Deselect,
    EndTurn,
}

const WALKBALE_TILE_COLOR: [f32; 4] = [0.2, 1.0, 0.2, 0.5];

fn make_action_show_tile(
    context: &mut Context,
    state: &State,
    view: &GameView,
    at: PosHex,
) -> Box<Action> {
    let screen_pos = map::hex_to_point(view.tile_size(), at);
    let mut sprite = Sprite::from_path(context, "tile.png", view.tile_size() * 2.0);
    match state.map().tile(at) {
        TileType::Floor => sprite.set_color([1.0, 1.0, 1.0, 1.0]),
        TileType::Lava => sprite.set_color([1.0, 0.7, 0.7, 1.0]),
    }
    sprite.set_pos(screen_pos);
    Box::new(action::Show::new(&view.layers().bg, &sprite))
}

fn make_action_grass(context: &mut Context, view: &GameView, at: PosHex) -> Box<Action> {
    let screen_pos = map::hex_to_point(view.tile_size(), at);
    let mut sprite = Sprite::from_path(context, "grass.png", view.tile_size() * 2.0);
    let n = view.tile_size() * 0.5;
    let screen_pos_grass = Point(Vector2 {
        x: screen_pos.0.x + thread_rng().gen_range(-n, n),
        y: screen_pos.0.y + thread_rng().gen_range(-n, n),
    });
    sprite.set_pos(screen_pos_grass);
    Box::new(action::Show::new(&view.layers().grass, &sprite))
}

fn make_action_create_map(state: &State, view: &GameView, context: &mut Context) -> Box<Action> {
    let mut actions = Vec::new();
    for hex_pos in state.map().iter() {
        actions.push(make_action_show_tile(context, state, view, hex_pos));
        if thread_rng().gen_range(0, 10) < 2 {
            actions.push(make_action_grass(context, view, hex_pos));
        }
    }
    Box::new(action::Sequence::new(actions))
}

fn build_unit_info_panel(
    context: &mut Context,
    gui: &mut Gui<GuiCommand>,
    state: &State,
    id: ObjId,
) -> gui::Id {
    let parts = state.parts();
    let st = parts.strength.get(id);
    let meta = parts.meta.get(id);
    let a = parts.agent.get(id);
    let anchor = gui::Anchor {
        vertical: gui::VAnchor::Bottom,
        horizontal: gui::HAnchor::Left,
    };
    let line_height = 0.08;
    let mut ids = Vec::new();
    {
        let mut line = |s: &str| {
            let sprite = gui::text_sprite(context, s, line_height);
            let id = gui.add_sprite(sprite);
            ids.push(id);
        };
        line(&format!("move points: {}", a.move_points.0));
        line(&format!("attack distance: {}", a.attack_distance.0));
        line(&format!("reactive attacks: {}", a.reactive_attacks.0));
        line(&format!("moves: {}/{}", a.moves.0, a.base_moves.0,));
        line(&format!("attacks: {}/{}", a.attacks.0, a.base_attacks.0,));
        line(&format!("jokers: {}/{}", a.jokers.0, a.base_jokers.0,));
        line(&format!(
            "strength: {}/{}",
            st.strength.0,
            st.base_strength.0
        ));
        line(&format!("[{}]", meta.name));
    }
    // TODO: Direction::Down
    gui.add_layout(anchor, gui::Direction::Up, ids)
}

fn build_gui(context: &mut Context) -> Gui<GuiCommand> {
    let mut gui = Gui::new(context);
    let direction = gui::Direction::Up;
    {
        let sprite_exit = gui::text_sprite(context, "exit", 0.1);
        let sprite_id_exit = gui.add_button(context, sprite_exit, GuiCommand::Exit);
        let anchor = gui::Anchor {
            vertical: gui::VAnchor::Top,
            horizontal: gui::HAnchor::Left,
        };
        gui.add_layout(anchor, direction, vec![sprite_id_exit]);
    }
    {
        let sprite_deselect = gui::text_sprite(context, "deselect", 0.1);
        let sprite_id_deselect = gui.add_button(context, sprite_deselect, GuiCommand::Deselect);
        let anchor = gui::Anchor {
            vertical: gui::VAnchor::Top,
            horizontal: gui::HAnchor::Right,
        };
        gui.add_layout(anchor, direction, vec![sprite_id_deselect]);
    }
    {
        let sprite_end_turn = gui::text_sprite(context, "end turn", 0.1);
        let sprite_id_end_turn = gui.add_button(context, sprite_end_turn, GuiCommand::EndTurn);
        let anchor = gui::Anchor {
            vertical: gui::VAnchor::Bottom,
            horizontal: gui::HAnchor::Right,
        };
        gui.add_layout(anchor, direction, vec![sprite_id_end_turn]);
    }
    gui
}

fn prepare_map_and_state(context: &mut Context, state: &mut State, view: &mut GameView) {
    let mut actions = Vec::new();
    actions.push(make_action_create_map(state, view, context));
    execute::create_objects(state, &mut |state, event, phase| {
        let action = visualize::visualize(state, view, context, event, phase);
        let action = Box::new(action::Fork::new(action));
        actions.push(action);
    });
    view.add_action(Box::new(action::Sequence::new(actions)));
}

#[derive(Debug)]
pub struct Game {
    gui: Gui<GuiCommand>,
    state: State,
    view: GameView,
    selected_unit_id: Option<ObjId>,
    pathfinder: Pathfinder,
    sprite_selection_marker: Sprite,
    block_timer: Option<Time>,
    sprites_walkable_tiles: Vec<Sprite>, // TODO: move to GameView
    sprites_attackable_tiles: Vec<Sprite>, // TODO: move to GameView
    ai: Ai,
    layout_id_info: Option<gui::Id>,
}

impl Game {
    pub fn new(context: &mut Context) -> Self {
        let prototypes_str = hate::fs::load_as_string("objects.ron");
        let prototypes = ron::de::from_str(&prototypes_str).unwrap();
        debug!("{:?}", prototypes);
        let mut state = State::new(prototypes);
        let radius = state.map().radius();
        let mut view = GameView::new();
        prepare_map_and_state(context, &mut state, &mut view);
        let size = view.tile_size() * 2.0;
        let mut sprite_selection_marker = Sprite::from_path(context, "selection.png", size);
        sprite_selection_marker.set_color([0.0, 0.0, 1.0, 0.8]);
        Self {
            gui: build_gui(context),
            state,
            view,
            selected_unit_id: None,
            pathfinder: Pathfinder::new(radius),
            block_timer: None,
            sprite_selection_marker,
            sprites_walkable_tiles: Vec::new(),
            sprites_attackable_tiles: Vec::new(),
            ai: Ai::new(PlayerId(1), radius),
            layout_id_info: None,
        }
    }

    fn exit(&mut self, context: &mut Context) {
        context.add_command(hate::screen::Command::Pop);
    }

    fn end_turn(&mut self, context: &mut Context) {
        if self.block_timer.is_some() {
            return;
        }
        self.deselect(context);
        let command = command::Command::EndTurn(command::EndTurn);
        let mut actions = Vec::new();
        actions.push(self.do_command_inner(context, command));
        actions.push(self.do_ai(context));
        self.add_actions(actions);
    }

    fn do_ai(&mut self, context: &mut Context) -> Box<Action> {
        debug!("AI: <");
        let mut actions = Vec::new();
        loop {
            let command = self.ai.command(&self.state).unwrap();
            debug!("AI: command = {:?}", command);
            actions.push(self.do_command_inner(context, command.clone()));
            actions.push(Box::new(action::Sleep::new(Time(0.3)))); // ??
            if let command::Command::EndTurn(_) = command {
                break;
            }
        }
        debug!("AI: >");
        Box::new(action::Sequence::new(actions))
    }

    fn handle_commands(&mut self, context: &mut Context) {
        while let Some(command) = self.gui.try_recv() {
            match command {
                GuiCommand::Exit => self.exit(context),
                GuiCommand::Deselect => self.deselect(context),
                GuiCommand::EndTurn => self.end_turn(context),
            }
        }
    }

    fn do_command_inner(
        &mut self,
        context: &mut Context,
        command: command::Command,
    ) -> Box<Action> {
        debug!("do_command_inner: {:?}", command);
        let mut actions = Vec::new();
        let state = &mut self.state;
        let view = &mut self.view;
        core::execute(state, &command, &mut |state, event, phase| {
            actions.push(visualize::visualize(state, view, context, event, phase));
        }).expect("Can't execute command");
        Box::new(action::Sequence::new(actions))
    }

    fn do_command(&mut self, context: &mut Context, command: command::Command) {
        let action = self.do_command_inner(context, command);
        self.add_action(action);
    }

    fn add_actions(&mut self, actions: Vec<Box<Action>>) {
        self.add_action(Box::new(action::Sequence::new(actions)));
    }

    fn add_action(&mut self, action: Box<Action>) {
        self.block_timer = Some(action.duration());
        self.view.add_action(action);
    }

    fn show_selection_marker(&mut self, id: ObjId) {
        let pos = self.state.parts().pos.get(id).0;
        let point = map::hex_to_point(self.view.tile_size(), pos);
        self.sprite_selection_marker.set_pos(point);
        let action = Box::new(action::Show::new(
            &self.view.layers().selection_marker,
            &self.sprite_selection_marker,
        ));
        self.view.add_action(action);
    }

    fn show_attackable_tiles(&mut self, context: &mut Context, id: ObjId) {
        let parts = self.state.parts();
        let selected_unit_player_id = parts.belongs_to.get(id).0;
        for target_id in parts.agent.ids() {
            let target_pos = parts.pos.get(target_id).0;
            let target_player_id = parts.belongs_to.get(target_id).0;
            if target_player_id == selected_unit_player_id {
                continue;
            }
            let command_attack = command::Command::Attack(command::Attack {
                attacker_id: id,
                target_id: target_id,
            });
            if check(&self.state, &command_attack).is_err() {
                continue;
            }
            let size = self.view.tile_size() * 2.0;
            let mut sprite = Sprite::from_path(context, "tile.png", size);
            self.sprites_attackable_tiles.push(sprite.clone());
            sprite.set_color([1.0, 0.3, 0.3, 0.8]);
            sprite.set_pos(map::hex_to_point(self.view.tile_size(), target_pos));
            let action = Box::new(action::Show::new(
                &self.view.layers().attackable_tiles,
                &sprite,
            ));
            self.view.add_action(action);
        }
    }

    fn show_walkable_tiles(&mut self, context: &mut Context, id: ObjId) {
        let agent = self.state.parts().agent.get(id);
        if agent.moves == Moves(0) && agent.jokers == Jokers(0) {
            return;
        }
        let map = self.pathfinder.map();
        for pos in map.iter() {
            let tile = map.tile(pos);
            if tile.cost() <= agent.move_points {
                let size = self.view.tile_size() * 2.0;
                let mut sprite = Sprite::from_path(context, "tile.png", size);
                self.sprites_walkable_tiles.push(sprite.clone());
                let mut color_from = WALKBALE_TILE_COLOR;
                color_from[3] = 0.0;
                sprite.set_color(color_from);
                sprite.set_pos(map::hex_to_point(self.view.tile_size(), pos));
                let color_to = WALKBALE_TILE_COLOR;
                let action = {
                    let layer = &self.view.layers().walkable_tiles;
                    Box::new(action::Sequence::new(vec![
                        Box::new(action::Show::new(layer, &sprite)),
                        Box::new(action::ChangeColorTo::new(&sprite, color_to, Time(0.2))),
                    ]))
                };
                self.view.add_action(action);
            }
        }
    }

    fn deselect(&mut self, _: &mut Context) {
        if let Some(layout_id_info) = self.layout_id_info.take() {
            self.gui.remove(layout_id_info).unwrap();
        }
        if self.selected_unit_id.is_some() {
            let action_hide = Box::new(action::Hide::new(
                &self.view.layers().selection_marker,
                &self.sprite_selection_marker,
            ));
            self.view.add_action(action_hide);
            for sprite in self.sprites_walkable_tiles.split_off(0) {
                let mut color = WALKBALE_TILE_COLOR;
                color[3] = 0.0;
                let action = {
                    let layer = &self.view.layers().walkable_tiles;
                    Box::new(action::Sequence::new(vec![
                        Box::new(action::ChangeColorTo::new(&sprite, color, Time(0.2))),
                        Box::new(action::Hide::new(layer, &sprite)),
                    ]))
                };
                self.view.add_action(action);
            }
            for sprite in self.sprites_attackable_tiles.split_off(0) {
                let action = {
                    let layer = &self.view.layers().attackable_tiles;
                    Box::new(action::Hide::new(layer, &sprite))
                };
                self.view.add_action(action);
            }
        }
        self.selected_unit_id = None;
    }

    fn select_unit(&mut self, context: &mut Context, id: ObjId) {
        self.deselect(context);
        self.selected_unit_id = Some(id);
        self.pathfinder.fill_map(&self.state, id);
        self.show_selection_marker(id);
        self.show_walkable_tiles(context, id);
        self.show_attackable_tiles(context, id);
        {
            let gui = &mut self.gui;
            let layout_id_info = build_unit_info_panel(context, gui, &self.state, id);
            self.layout_id_info = Some(layout_id_info);
        }
    }

    fn handle_event_click(&mut self, context: &mut Context, point: Point) {
        let pos = map::point_to_hex(self.view.tile_size(), point);
        self.gui.click(point);
        if self.block_timer.is_some() {
            return;
        }
        if self.state.map().is_inboard(pos) {
            let object_ids = core::object_ids_at(&self.state, pos);
            debug!("object_ids: {:?}", object_ids);
            if !object_ids.is_empty() {
                assert_eq!(object_ids.len(), 1);
                let id = object_ids[0];
                let other_unit_player_id = self.state.parts().belongs_to.get(id).0;
                if let Some(selected_unit_id) = self.selected_unit_id {
                    let selected_unit_player_id =
                        self.state.parts().belongs_to.get(selected_unit_id).0;
                    if selected_unit_id == id {
                        self.deselect(context);
                        return;
                    }
                    if other_unit_player_id == selected_unit_player_id
                        || other_unit_player_id == self.state.player_id()
                    {
                        self.select_unit(context, id);
                        return;
                    }
                    let command_attack = command::Command::Attack(command::Attack {
                        attacker_id: selected_unit_id,
                        target_id: id,
                    });
                    if check(&self.state, &command_attack).is_err() {
                        return;
                    }
                    self.do_command(context, command_attack);
                    let parts = self.state.parts();
                    if parts.agent.get_opt(selected_unit_id).is_some() {
                        self.pathfinder.fill_map(&self.state, selected_unit_id);
                    }
                } else {
                    self.select_unit(context, id);
                }
            } else if let Some(id) = self.selected_unit_id {
                let path = self.pathfinder.path(pos).unwrap();
                let command_move = command::Command::MoveTo(command::MoveTo { id, path });
                if check(&self.state, &command_move).is_err() {
                    return;
                }
                self.do_command(context, command_move);
                if self.state.parts().agent.get_opt(id).is_some() {
                    self.pathfinder.fill_map(&self.state, id);
                }
            }
        }
    }

    fn update_block_timer(&mut self, context: &mut Context, dtime: Time) {
        self.block_timer.as_mut().map(|t| t.0 -= dtime.0);
        if let Some(time) = self.block_timer {
            if time <= Time(0.0) {
                self.block_timer = None;
                if let Some(id) = self.selected_unit_id {
                    if self.state.parts().agent.get_opt(id).is_some() {
                        self.select_unit(context, id);
                    } else {
                        self.deselect(context);
                    }
                }
            }
        }
    }
}

impl Screen for Game {
    fn tick(&mut self, context: &mut Context, dtime: Time) {
        self.view.tick(context, dtime);
        self.update_block_timer(context, dtime);
        self.gui.draw(context);
    }

    fn handle_event(&mut self, context: &mut Context, event: Event) {
        match event {
            Event::Click { pos } => self.handle_event_click(context, pos),
            Event::Resize { aspect_ratio } => self.gui.resize(aspect_ratio),
        }
        self.handle_commands(context);
    }
}
