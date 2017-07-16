use hate::{self, Time, Sprite, Event, Screen, Context};
use hate::geom::Point;
use hate::gui::{self, Gui};
use hate::scene::action;
use visualize;
use map;
use game_view::GameView;
use core;
use core::{Unit, PlayerId, State, Simulator, ObjId};
use core::command;
use core::movement::{MovePoints, Pathfinder};

#[derive(Copy, Clone, Debug)]
enum Command {
    A,
    B,
    C,
    D,
    E,
    F,
    Exit,
    Deselect,
}

const WALKBALE_TILE_COLOR: [f32; 4] = [0.4, 1.0, 0.4, 0.8];

#[derive(Debug)]
pub struct Game {
    gui: Gui<Command>,
    button_f_id: gui::Id,
    simulator: Simulator,
    state: State,
    view: GameView,
    selected_unit_id: Option<ObjId>,
    pathfinder: Pathfinder,
    sprite_selection_marker: Sprite,
    block_timer: Option<Time>,
    sprites_walkable_tiles: Vec<Sprite>,
}

impl Game {
    pub fn new(context: &mut Context) -> Self {
        let mut state = State::new();
        let pathfinder = Pathfinder::new(state.map().radius());
        let mut simulator = Simulator::new();
        core::create_objects(&mut state, &mut simulator);

        let view = GameView::new(&state, context);

        let mut gui = Gui::new(context);

        let _ /*layout_a_id*/ = {
            // let sprite_a = gui::text_sprite(context, "A", 0.1);
            // let sprite_b = gui::text_sprite(context, "B", 0.1);
            // let sprite_c = gui::text_sprite(context, "C", 0.1);
            let sprite_a = Sprite::from_path(context, "tile.png", 0.2);
            let sprite_b = Sprite::from_path(context, "imp.png", 0.2);
            let sprite_c = Sprite::from_path(context, "swordsman.png", 0.2);
            let sprite_a_id = gui.add_button(context, sprite_a, Command::A);
            let sprite_b_id = gui.add_button(context, sprite_b, Command::B);
            let sprite_c_id = gui.add_button(context, sprite_c, Command::C);
            let anchor = gui::Anchor {
                vertical: gui::VAnchor::Top,
                horizontal: gui::HAnchor::Left,
            };
            let direction = gui::Direction::Right;
            gui.add_layout(anchor, direction, vec![
                sprite_a_id,
                sprite_b_id,
                sprite_c_id,
            ])
        };

        let button_f_id;
        let _ /*layout_b_id*/ = {
            let sprite_d = gui::text_sprite(context, "D", 0.1);
            let sprite_e = gui::text_sprite(context, "E", 0.1);
            let sprite_f = gui::text_sprite(context, "F", 0.1);
            let sprite_d_id = gui.add_button(context, sprite_d, Command::D);
            let sprite_e_id = gui.add_button(context, sprite_e, Command::E);
            let sprite_f_id = gui.add_button(context, sprite_f, Command::F);
            button_f_id = sprite_f_id;
            let anchor = gui::Anchor {
                vertical: gui::VAnchor::Bottom,
                horizontal: gui::HAnchor::Right,
            };
            let direction = gui::Direction::Up;
            gui.add_layout(anchor, direction, vec![
                sprite_d_id,
                sprite_e_id,
                // layout_a_id, // TODO: nested layouts
                sprite_f_id,
            ])
        };

        let _ /*layout_c_id*/ = {
            let sprite_a = gui::text_sprite(context, "move: A", 0.1);
            let sprite_b = gui::text_sprite(context, "attack: B", 0.1);
            let sprite_deselect = gui::text_sprite(context, "deselect", 0.1);
            let sprite_exit = gui::text_sprite(context, "exit", 0.1);
            let sprite_a_id = gui.add_button(context, sprite_a, Command::A);
            let sprite_b_id = gui.add_button(context, sprite_b, Command::B);
            let sprite_id_deselect = gui.add_button(context, sprite_deselect, Command::Deselect);
            let sprite_id_exit = gui.add_button(context, sprite_exit, Command::Exit);
            let anchor = gui::Anchor {
                vertical: gui::VAnchor::Middle,
                horizontal: gui::HAnchor::Left,
            };
            let direction = gui::Direction::Up;
            gui.add_layout(anchor, direction, vec![
                sprite_a_id,
                sprite_b_id,
                sprite_id_deselect,
                sprite_id_exit,
            ])
        };

        let mut sprite_selection_marker = Sprite::from_path(context, "selection.png", 0.2);
        sprite_selection_marker.set_color([0.0, 0.0, 1.0, 0.8]);

        let mut screen = Self {
            gui,
            button_f_id,
            simulator,
            state,
            view,
            selected_unit_id: None,
            pathfinder,
            block_timer: None,
            sprite_selection_marker,
            sprites_walkable_tiles: Vec::new(),
        };
        screen.process_core_events(context);
        screen
    }

    fn exit(&mut self, context: &mut Context) {
        context.add_command(hate::screen::Command::Pop);
    }

    fn handle_commands(&mut self, context: &mut Context) {
        while let Some(command) = self.gui.try_recv() {
            match command {
                Command::A => println!("A"),
                Command::B => println!("B"),
                Command::C => println!("C"),
                Command::D => println!("D"),
                Command::E => println!("E"),
                Command::F => {
                    println!("F");
                    let new_sprite = gui::text_sprite(context, "FF", 0.1);
                    self.gui
                        .update_sprite(context, self.button_f_id, new_sprite);
                }
                Command::Exit => self.exit(context),
                Command::Deselect => self.deselect(context),
            }
        }
    }

    fn do_command(&mut self, context: &mut Context, command: command::Command) {
        println!("Game: do_command: {:?}", command);
        self.simulator.do_command(&self.state, command);
        self.process_core_events(context);
    }

    fn process_core_events(&mut self, context: &mut Context) {
        while let Some(event) = self.simulator.tick() {
            let (action, time) = visualize::visualize(&self.state, &mut self.view, context, &event);
            self.block_timer = Some(time);
            self.view.add_action(action);
            core::event::apply(&mut self.state, &event);
        }
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

    fn show_walkable_tiles(&mut self, context: &mut Context, id: ObjId) {
        let move_points = self.state.unit(id).move_points;
        let map = self.pathfinder.map();
        for pos in map.iter() {
            let tile = map.tile(pos);
            if tile.cost() <= move_points {
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
        }
        self.selected_unit_id = None;
    }

    fn select_unit(&mut self, context: &mut Context, id: ObjId) {
        self.deselect(context);
        self.selected_unit_id = Some(id);
        self.pathfinder.fill_map(&self.state, self.state.unit(id));
        self.show_selection_marker(id);
        self.show_walkable_tiles(context, id);
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
            if object_ids.len() == 1 {
                let id = object_ids[0];
                if let Some(selected_unit_id) = self.selected_unit_id {
                    let selected_unit_player_id = self.state.unit(selected_unit_id).player_id;
                    let other_unit_player_id = self.state.unit(id).player_id;
                    if selected_unit_player_id == other_unit_player_id {
                        self.select_unit(context, id);
                    } else {
                        let command_attack = command::Command::Attack(command::Attack {
                            attacker_id: selected_unit_id,
                            target_id: id,
                        });
                        self.do_command(context, command_attack);
                        self.pathfinder
                            .fill_map(&self.state, self.state.unit(selected_unit_id));
                    }
                } else {
                    self.select_unit(context, id);
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
                        move_points: MovePoints(6),
                    },
                });
                self.do_command(context, command_create);
            }
        }
    }

    fn update_block_timer(&mut self, context: &mut Context, dtime: Time) {
        self.block_timer.as_mut().map(|t| t.0 -= dtime.0);
        if let Some(time) = self.block_timer.clone() {
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
