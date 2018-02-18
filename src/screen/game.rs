use ron;
use rand::{thread_rng, Rng};
use cgmath::Vector2;
use hate::{self, Context, Event, Screen, Sprite, Time};
use hate::geom::Point;
use hate::gui::{self, Gui};
use hate::scene::action::{self, Action};
use visualize;
use map;
use game_view::{GameView, SelectionMode};
use ai::Ai;
use core::{check, ObjId, PlayerId, State, TileType};
use core::{self, command, execute, state};
use core::map::PosHex;
use core::movement::Pathfinder;
use core::effect::Duration;
use core::ability::{self, Ability};

#[derive(Copy, Clone, Debug)]
enum GuiCommand {
    Exit,
    Deselect,
    EndTurn,
    Ability(Ability),
}

fn make_action_show_tile(
    context: &mut Context,
    state: &State,
    view: &GameView,
    at: PosHex,
) -> Box<Action> {
    let screen_pos = map::hex_to_point(view.tile_size(), at);
    let texture_name = match state.map().tile(at) {
        TileType::Plain => "tile.png",
        TileType::Rocks => "tile_rocks.png",
    };
    let size = view.tile_size() * 2.0;
    let mut sprite = Sprite::from_path(context, texture_name, size);
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

fn line_height() -> f32 {
    0.08
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
    let mut ids = Vec::new();
    {
        let mut line = |s: &str| {
            let sprite = gui::text_sprite(context, s, line_height());
            let id = gui.add_sprite(sprite);
            ids.push(id);
        };
        if let Some(effects) = parts.effects.get_opt(id) {
            if !effects.0.is_empty() {
                for effect in &effects.0 {
                    let s = effect.effect.to_str();
                    match effect.duration {
                        Duration::Forever => line(&format!("'{}'", s)),
                        Duration::Rounds(n) => line(&format!("'{}' ({})", s, n)),
                    }
                }
                line("[effects]:");
            }
        }
        if let Some(abilities) = parts.abilities.get_opt(id) {
            if !abilities.0.is_empty() {
                for ability in &abilities.0 {
                    let s = ability.ability.to_str();
                    line(&format!("'{}'", s));
                }
                line("[abilities]:");
            }
        }
        if let Some(abilities) = parts.passive_abilities.get_opt(id) {
            if !abilities.0.is_empty() {
                for ability in &abilities.0 {
                    let s = ability.to_str();
                    line(&format!("'{}'", s));
                }
                line("[passive abilities]:");
            }
        }
        line(&format!("move points: {}", a.move_points.0));
        line(&format!("attack strength: {}", a.attack_strength.0));
        if a.attack_distance.0 != 1 {
            line(&format!("attack distance: {}", a.attack_distance.0));
        }
        if a.reactive_attacks.0 != 0 {
            line(&format!("reactive attacks: {}", a.reactive_attacks.0));
        }
        line(&format!("moves: {}/{}", a.moves.0, a.base_moves.0));
        line(&format!("attacks: {}/{}", a.attacks.0, a.base_attacks.0));
        if a.jokers.0 != 0 || a.base_jokers.0 != 0 {
            line(&format!("jokers: {}/{}", a.jokers.0, a.base_jokers.0));
        }
        line(&format!(
            "strength: {}/{}",
            st.strength.0, st.base_strength.0
        ));
        line(&format!("[{}]", meta.name));
    }
    // TODO: Direction::Down
    gui.add_layout(anchor, gui::Direction::Up, ids)
}

fn build_unit_abilities_panel(
    context: &mut Context,
    gui: &mut Gui<GuiCommand>,
    state: &State,
    id: ObjId,
) -> Option<gui::Id> {
    let parts = state.parts();
    let abilities = match parts.abilities.get_opt(id) {
        Some(abilities) => &abilities.0,
        None => return None,
    };
    let agent = parts.agent.get(id);
    if agent.attacks <= core::Attacks(0) && agent.jokers <= core::Jokers(0) {
        return None;
    }
    let mut ids = Vec::new();
    let anchor = gui::Anchor {
        vertical: gui::VAnchor::Middle,
        horizontal: gui::HAnchor::Right,
    };
    for &ability in abilities {
        let text = match ability.status {
            ability::Status::Ready => format!("{}", ability.ability.to_str()),
            ability::Status::Cooldown(n) => format!("{} ({})", ability.ability.to_str(), n),
        };
        let sprite = gui::text_sprite(context, &text, line_height());
        let id = gui.add_button(context, sprite, GuiCommand::Ability(ability.ability));
        ids.push(id);
    }
    Some(gui.add_layout(anchor, gui::Direction::Up, ids))
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
    execute::create_terrain(state);
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
    mode: SelectionMode,
    view: GameView,
    selected_unit_id: Option<ObjId>,
    pathfinder: Pathfinder,
    block_timer: Option<Time>,
    ai: Ai,
    layout_id_info: Option<gui::Id>,
    layout_id_abilities: Option<gui::Id>,
}

impl Game {
    pub fn new(context: &mut Context) -> Self {
        let prototypes_str = hate::fs::load_as_string("objects.ron");
        let prototypes = ron::de::from_str(&prototypes_str).unwrap();
        debug!("{:?}", prototypes);
        let mut state = State::new(prototypes);
        let radius = state.map().radius();
        let mut view = GameView::new(&state, context);
        prepare_map_and_state(context, &mut state, &mut view);
        Self {
            mode: SelectionMode::Normal,
            gui: build_gui(context),
            state,
            view,
            selected_unit_id: None,
            pathfinder: Pathfinder::new(radius),
            block_timer: None,
            ai: Ai::new(PlayerId(1), radius),
            layout_id_info: None,
            layout_id_abilities: None,
        }
    }

    fn exit(&mut self, context: &mut Context) {
        context.add_command(hate::screen::Command::Pop);
    }

    fn end_turn(&mut self, context: &mut Context) {
        if self.block_timer.is_some() {
            return;
        }
        self.deselect();
        let command = command::Command::EndTurn(command::EndTurn);
        let mut actions = Vec::new();
        actions.push(self.do_command_inner(context, &command));
        actions.push(self.do_ai(context));
        self.add_actions(actions);
    }

    fn do_ai(&mut self, context: &mut Context) -> Box<Action> {
        debug!("AI: <");
        let mut actions = Vec::new();
        loop {
            let command = self.ai.command(&self.state).unwrap();
            debug!("AI: command = {:?}", command);
            actions.push(self.do_command_inner(context, &command));
            actions.push(Box::new(action::Sleep::new(Time(0.3)))); // ??
            if let command::Command::EndTurn(_) = command {
                break;
            }
        }
        debug!("AI: >");
        Box::new(action::Sequence::new(actions))
    }

    fn use_ability(&mut self, context: &mut Context, ability: Ability) {
        // TODO: code duplication (see check.rs and event.rs)
        let id = self.selected_unit_id.unwrap(); // TODO: Extract to some specific method
        for rechargeable in &self.state.parts().abilities.get(id).0 {
            if rechargeable.ability == ability && rechargeable.status != ability::Status::Ready {
                debug!("ability isn't ready yet");
                return;
            }
        }
        self.set_mode(context, id, SelectionMode::Ability(ability));
    }

    fn handle_commands(&mut self, context: &mut Context) {
        while let Some(command) = self.gui.try_recv() {
            match command {
                GuiCommand::Exit => self.exit(context),
                GuiCommand::Deselect => self.deselect(),
                GuiCommand::EndTurn => self.end_turn(context),
                GuiCommand::Ability(ability) => self.use_ability(context, ability),
            }
        }
    }

    fn do_command_inner(
        &mut self,
        context: &mut Context,
        command: &command::Command,
    ) -> Box<Action> {
        debug!("do_command_inner: {:?}", command);
        let mut actions = Vec::new();
        let state = &mut self.state;
        let view = &mut self.view;
        core::execute(state, command, &mut |state, event, phase| {
            actions.push(visualize::visualize(state, view, context, event, phase));
        }).expect("Can't execute command");
        Box::new(action::Sequence::new(actions))
    }

    fn do_command(&mut self, context: &mut Context, command: &command::Command) {
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

    fn deselect(&mut self) {
        if let Some(layout_id_info) = self.layout_id_info.take() {
            self.gui.remove(layout_id_info).unwrap();
        }
        if let Some(layout_id_abilities) = self.layout_id_abilities.take() {
            self.gui.remove(layout_id_abilities).unwrap();
        }
        if self.selected_unit_id.is_some() {
            self.view.deselect();
        }
        self.selected_unit_id = None;
        self.mode = SelectionMode::Normal;
    }

    fn set_mode(&mut self, context: &mut Context, id: ObjId, mode: SelectionMode) {
        self.deselect();
        assert!(self.state.parts().agent.get_opt(id).is_some());
        self.selected_unit_id = Some(id);
        let state = &self.state;
        let gui = &mut self.gui;
        match mode {
            SelectionMode::Ability(_) => {
                // TODO: Update the GUI here: explain how to use or cancel the ability.
                // 'Select target tile'
            }
            SelectionMode::Normal => {
                self.pathfinder.fill_map(state, id);
                self.layout_id_info = Some(build_unit_info_panel(context, gui, state, id));
                self.layout_id_abilities = build_unit_abilities_panel(context, gui, state, id);
            }
        }
        let map = self.pathfinder.map();
        self.view.set_mode(state, map, context, id, &mode);
        self.mode = mode;
    }

    fn handle_unit_click(&mut self, context: &mut Context, id: ObjId) {
        if self.state.parts().agent.get_opt(id).is_none() {
            // only agents can be selected
            return;
        }
        let other_unit_player_id = self.state.parts().belongs_to.get(id).0;
        if let Some(selected_unit_id) = self.selected_unit_id {
            let selected_unit_player_id = self.state.parts().belongs_to.get(selected_unit_id).0;
            if selected_unit_id == id {
                self.deselect();
                return;
            }
            if other_unit_player_id == selected_unit_player_id
                || other_unit_player_id == self.state.player_id()
            {
                self.set_mode(context, id, SelectionMode::Normal);
                return;
            }
            let command_attack = command::Command::Attack(command::Attack {
                attacker_id: selected_unit_id,
                target_id: id,
            });
            if check(&self.state, &command_attack).is_err() {
                return;
            }
            self.do_command(context, &command_attack);
            self.fill_map();
        } else {
            self.set_mode(context, id, SelectionMode::Normal);
        }
    }

    fn fill_map(&mut self) {
        let selected_unit_id = self.selected_unit_id.unwrap();
        let parts = self.state.parts();
        if parts.agent.get_opt(selected_unit_id).is_some() {
            self.pathfinder.fill_map(&self.state, selected_unit_id);
        }
    }

    fn try_move_selected_unit(&mut self, context: &mut Context, pos: PosHex) {
        if let Some(id) = self.selected_unit_id {
            let path = match self.pathfinder.path(pos) {
                Some(path) => path,
                None => return,
            };
            let command_move = command::Command::MoveTo(command::MoveTo { id, path });
            if check(&self.state, &command_move).is_err() {
                return;
            }
            self.do_command(context, &command_move);
            self.fill_map();
        }
    }

    fn handle_event_click(&mut self, context: &mut Context, point: Point) {
        let pos = map::point_to_hex(self.view.tile_size(), point);
        self.gui.click(point);
        if self.block_timer.is_some() {
            return;
        }
        if self.state.map().is_inboard(pos) {
            if let SelectionMode::Ability(ability) = self.mode {
                let selected_id = self.selected_unit_id.unwrap();
                let command = command::Command::UseAbility(command::UseAbility {
                    id: selected_id,
                    pos,
                    ability,
                });
                if check(&self.state, &command).is_ok() {
                    self.do_command(context, &command);
                } else {
                    self.view.message(context, pos, "cancelled");
                }
                self.set_mode(context, selected_id, SelectionMode::Normal);
            } else if let Some(id) = state::agent_id_at_opt(&self.state, pos) {
                self.handle_unit_click(context, id);
            } else {
                self.try_move_selected_unit(context, pos);
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
                        self.set_mode(context, id, SelectionMode::Normal);
                    } else {
                        self.deselect();
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
