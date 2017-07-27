use hate::{self, Context, Event, Screen, Sprite, Time};
use hate::geom::Point;
use hate::gui::{self, Gui};
use hate::scene::action::{self, Action};
use visualize;
use map;
use game_view::GameView;
use ai::Ai;
use core;
use core::{check, Attacks, Moves, ObjId, PlayerId, Simulator, State, Unit};
use core::command;
use core::movement::{MovePoints, Pathfinder};

#[derive(Copy, Clone, Debug)]
enum GuiCommand {
    Exit,
    Deselect,
    EndTurn,
}

const WALKBALE_TILE_COLOR: [f32; 4] = [0.4, 1.0, 0.4, 0.8];

#[derive(Debug)]
pub struct Game {
    gui: Gui<GuiCommand>,
    simulator: Simulator,
    state: State,
    view: GameView,
    selected_unit_id: Option<ObjId>,
    pathfinder: Pathfinder,
    sprite_selection_marker: Sprite,
    block_timer: Option<Time>,
    sprites_walkable_tiles: Vec<Sprite>,
    sprites_attackable_tiles: Vec<Sprite>,
    ai: Ai,
}

impl Game {
    pub fn new(context: &mut Context) -> Self {
        let mut state = State::new();
        let pathfinder = Pathfinder::new(state.map().radius());
        let ai = Ai::new(PlayerId(1), state.map().radius());
        let mut simulator = Simulator::new();
        core::create_objects(&mut state, &mut simulator);

        let view = GameView::new(&state, context);

        let mut gui = Gui::new(context);

        let _ /*layout_c_id*/ = {
            let sprite_deselect = gui::text_sprite(context, "deselect", 0.1);
            let sprite_exit = gui::text_sprite(context, "exit", 0.1);
            let sprite_end_turn = gui::text_sprite(context, "end turn", 0.1);
            let sprite_id_deselect = gui.add_button(context, sprite_deselect, GuiCommand::Deselect);
            let sprite_id_exit = gui.add_button(context, sprite_exit, GuiCommand::Exit);
            let sprite_id_end_turn = gui.add_button(context, sprite_end_turn, GuiCommand::EndTurn);
            let anchor = gui::Anchor {
                vertical: gui::VAnchor::Middle,
                horizontal: gui::HAnchor::Left,
            };
            let direction = gui::Direction::Up;
            gui.add_layout(anchor, direction, vec![
                sprite_id_deselect,
                sprite_id_exit,
                sprite_id_end_turn,
            ])
        };

        let mut sprite_selection_marker = Sprite::from_path(context, "selection.png", 0.2);
        sprite_selection_marker.set_color([0.0, 0.0, 1.0, 0.8]);

        let mut screen = Self {
            gui,
            simulator,
            state,
            view,
            selected_unit_id: None,
            pathfinder,
            block_timer: None,
            sprite_selection_marker,
            sprites_walkable_tiles: Vec::new(),
            sprites_attackable_tiles: Vec::new(),
            ai,
        };
        screen.process_core_events(context);
        screen
    }

    fn exit(&mut self, context: &mut Context) {
        context.add_command(hate::screen::Command::Pop);
    }

    fn end_turn(&mut self, context: &mut Context) {
        self.deselect(context);
        let command = command::Command::EndTurn(command::EndTurn);
        self.do_command(context, command);
        {
            println!("AI: <");
            let mut actions: Vec<Box<Action>> = Vec::new();
            loop {
                let command = self.ai.command(&self.state).unwrap();
                println!("AI: command = {:?}", command);
                self.simulator.do_command(&self.state, command.clone());
                actions.extend(self.prepare_actions(context));
                actions.push(Box::new(action::Sleep::new(Time(0.3))));
                if let command::Command::EndTurn(_) = command {
                    break;
                }
            }
            self.add_actions(actions);
            println!("AI: >");
        }
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

    fn do_command(&mut self, context: &mut Context, command: command::Command) {
        println!("Game: do_command: {:?}", command);
        self.simulator.do_command(&self.state, command);
        self.process_core_events(context);
    }

    fn process_core_events(&mut self, context: &mut Context) {
        let actions = self.prepare_actions(context);
        self.add_actions(actions);
    }

    fn prepare_actions(&mut self, context: &mut Context) -> Vec<Box<Action>> {
        let mut actions = Vec::new();
        while let Some(event) = self.simulator.tick() {
            let action = visualize::visualize(&self.state, &mut self.view, context, &event);
            actions.push(action);
            core::event::apply(&mut self.state, &event);
        }
        actions
    }

    fn add_actions(&mut self, actions: Vec<Box<Action>>) {
        let action_sequence = Box::new(action::Sequence::new(actions));
        self.block_timer = Some(action_sequence.duration());
        self.view.add_action(action_sequence);
    }

    fn show_selection_marker(&mut self, id: ObjId) {
        let pos = self.state.unit(id).pos;
        let point = map::hex_to_point(self.view.tile_size(), pos);
        self.sprite_selection_marker.set_pos(point);
        let action = Box::new(action::Show::new(
            &self.view.layers().selection_marker,
            &self.sprite_selection_marker,
        ));
        self.view.add_action(action);
    }

    fn show_attackable_tiles(&mut self, context: &mut Context, id: ObjId) {
        let selected_unit = self.state.unit(id);
        for target_id in self.state.obj_iter() {
            let target = self.state.unit(target_id);
            if target.player_id == selected_unit.player_id {
                continue;
            }
            let command_attack = command::Command::Attack(command::Attack {
                attacker_id: id,
                target_id: target_id,
            });
            if check(&self.state, &command_attack).is_err() {
                continue;
            }
            let mut sprite = Sprite::from_path(context, "tile.png", 0.2);
            self.sprites_attackable_tiles.push(sprite.clone());
            sprite.set_color([1.0, 0.3, 0.3, 0.8]);
            sprite.set_pos(map::hex_to_point(self.view.tile_size(), target.pos));
            let action = Box::new(action::Show::new(
                &self.view.layers().attackable_tiles,
                &sprite,
            ));
            self.view.add_action(action);
        }
    }

    fn show_walkable_tiles(&mut self, context: &mut Context, id: ObjId) {
        let unit = self.state.unit(id);
        if unit.moves == Moves(0) {
            return;
        }
        let map = self.pathfinder.map();
        for pos in map.iter() {
            let tile = map.tile(pos);
            if tile.cost() <= unit.move_points {
                let mut sprite = Sprite::from_path(context, "tile.png", 0.2);
                self.sprites_walkable_tiles.push(sprite.clone());
                let mut color_from = WALKBALE_TILE_COLOR;
                color_from[3] = 0.0;
                sprite.set_color(color_from);
                sprite.set_pos(map::hex_to_point(self.view.tile_size(), pos));
                let sleep_time = Time(0.05 * tile.cost().0 as f32);
                let color_to = WALKBALE_TILE_COLOR;
                let action = {
                    let layer = &self.view.layers().walkable_tiles;
                    Box::new(action::Sequence::new(vec![
                        Box::new(action::Sleep::new(sleep_time)),
                        Box::new(action::Show::new(layer, &sprite)),
                        Box::new(action::ChangeColorTo::new(&sprite, color_to, Time(0.2))),
                    ]))
                };
                self.view.add_action(action);
            }
        }
    }

    fn deselect(&mut self, _: &mut Context) {
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
        self.pathfinder.fill_map(&self.state, self.state.unit(id));
        self.show_selection_marker(id);
        self.show_walkable_tiles(context, id);
        self.show_attackable_tiles(context, id);
    }

    fn handle_event_click(&mut self, context: &mut Context, pos: Point) {
        let hex_pos = map::point_to_hex(self.view.tile_size(), pos);
        self.gui.click(pos);
        if self.block_timer.is_some() {
            return;
        }
        if self.state.map().is_inboard(hex_pos) {
            let object_ids = self.state.object_ids_at(hex_pos);
            println!("object_ids: {:?}", object_ids);
            if !object_ids.is_empty() {
                assert_eq!(object_ids.len(), 1);
                let id = object_ids[0];
                let other_unit_player_id = self.state.unit(id).player_id;
                if other_unit_player_id == self.state.player_id() {
                    self.select_unit(context, id);
                } else if let Some(selected_unit_id) = self.selected_unit_id {
                    let command_attack = command::Command::Attack(command::Attack {
                        attacker_id: selected_unit_id,
                        target_id: id,
                    });
                    self.do_command(context, command_attack);
                    let selected_unit = self.state.unit(selected_unit_id);
                    self.pathfinder.fill_map(&self.state, selected_unit);
                }
            } else if let Some(id) = self.selected_unit_id {
                let path = self.pathfinder.path(hex_pos).unwrap();
                let command_move = command::Command::MoveTo(command::MoveTo { id, path });
                self.do_command(context, command_move);
                self.pathfinder.fill_map(&self.state, self.state.unit(id));
            } else {
                let id = self.state.alloc_id();
                println!("new id = {:?}", id);
                let command_create = command::Command::Create(command::Create {
                    id,
                    unit: Unit {
                        pos: hex_pos,
                        player_id: PlayerId(0),
                        // TODO: remove code duplication
                        move_points: MovePoints(3),
                        attacks: Attacks(2),
                        moves: Moves(2),
                    },
                });
                self.do_command(context, command_create);
            }
        }
    }

    fn update_block_timer(&mut self, context: &mut Context, dtime: Time) {
        self.block_timer.as_mut().map(|t| t.0 -= dtime.0);
        if let Some(time) = self.block_timer {
            if time <= Time(0.0) {
                self.block_timer = None;
                if let Some(id) = self.selected_unit_id {
                    self.select_unit(context, id);
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
