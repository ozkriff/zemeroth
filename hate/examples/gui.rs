extern crate hate;

use std::env;

use hate::{Context, Event, Screen, Sprite, Time};
use hate::geom::Point;
use hate::gui::{self, Gui};

#[derive(Copy, Clone, Debug)]
enum GuiCommand {
    A,
    B,
    C,
    D,
    E,
    F,
    RemoveButton,
    NextMap,
    Exit,
}

#[derive(Debug)]
pub struct GuiTest {
    gui: Gui<GuiCommand>,
    layout_id_a: gui::Id,
    button_f_id: gui::Id,
    button_id_next_map: gui::Id,
    button_id_will_be_removed: gui::Id,
    map_names: Vec<&'static str>,
    selected_map_index: usize,
}

impl GuiTest {
    pub fn new(context: &mut Context) -> Self {
        let mut gui = Gui::new(context);
        let map_names = vec!["map01", "map02", "map03"];
        let selected_map_index = 0;

        let layout_id_a = {
            let sprite_a = Sprite::from_path(context, "tile.png", 0.2);
            let sprite_b = Sprite::from_path(context, "imp.png", 0.2);
            let sprite_c = Sprite::from_path(context, "swordsman.png", 0.2);
            let sprite_a_id = gui.add_button(context, sprite_a, GuiCommand::A);
            let sprite_b_id = gui.add_button(context, sprite_b, GuiCommand::B);
            let sprite_c_id = gui.add_button(context, sprite_c, GuiCommand::C);
            let anchor = gui::Anchor {
                vertical: gui::VAnchor::Top,
                horizontal: gui::HAnchor::Left,
            };
            let direction = gui::Direction::Right;
            gui.add_layout(
                anchor,
                direction,
                vec![sprite_a_id, sprite_b_id, sprite_c_id],
            )
        };

        let button_f_id;
        let _ /*layout_b_id*/ = {
            let sprite_d = gui::text_sprite(context, "D", 0.1);
            let sprite_e = gui::text_sprite(context, "E", 0.1);
            let sprite_f = gui::text_sprite(context, "F", 0.1);
            let sprite_d_id = gui.add_button(context, sprite_d, GuiCommand::D);
            let sprite_e_id = gui.add_button(context, sprite_e, GuiCommand::E);
            let sprite_f_id = gui.add_button(context, sprite_f, GuiCommand::F);
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

        let button_id_next_map;
        let button_id_will_be_removed;
        let _ /*layout_c_id*/ = {
            let sprite_will_be_removed = gui::text_sprite(context, "will be removed (A)", 0.1);
            let sprite_remove_next = gui::text_sprite(context, "remove next button", 0.1);
            let sprite_exit = gui::text_sprite(context, "exit", 0.1);
            let label_next_map = format!("map: {}", map_names[selected_map_index]);
            let sprite_next_map = gui::text_sprite(context, &label_next_map, 0.1);
            button_id_will_be_removed = gui.add_button(
                context, sprite_will_be_removed, GuiCommand::A);
            let button_id_remove_button = gui.add_button(
                context, sprite_remove_next, GuiCommand::RemoveButton);
            button_id_next_map = gui.add_button(context, sprite_next_map, GuiCommand::NextMap);
            let sprite_id_exit = gui.add_button(context, sprite_exit, GuiCommand::Exit);
            let anchor = gui::Anchor {
                vertical: gui::VAnchor::Middle,
                horizontal: gui::HAnchor::Left,
            };
            let direction = gui::Direction::Up;
            gui.add_layout(anchor, direction, vec![
                button_id_will_be_removed,
                button_id_remove_button,
                button_id_next_map,
                sprite_id_exit,
            ])
        };

        let mut sprite_selection_marker = Sprite::from_path(context, "selection.png", 0.2);
        sprite_selection_marker.set_color([0.0, 0.0, 1.0, 0.8]);

        Self {
            layout_id_a,
            gui,
            button_f_id,
            button_id_will_be_removed,
            map_names,
            selected_map_index,
            button_id_next_map,
        }
    }

    fn exit(&mut self, context: &mut Context) {
        context.add_command(hate::screen::Command::Pop);
    }

    fn select_next_map(&mut self, context: &mut Context) {
        self.selected_map_index += 1;
        if self.selected_map_index == self.map_names.len() {
            self.selected_map_index = 0;
        }
        let text = &format!("map: {}", self.map_names[self.selected_map_index]);
        let new_sprite = gui::text_sprite(context, text, 0.1);
        let button_id = self.button_id_next_map;
        self.gui.update_sprite(context, button_id, new_sprite);
    }

    fn handle_commands(&mut self, context: &mut Context) {
        while let Some(command) = self.gui.try_recv() {
            match command {
                GuiCommand::A => println!("A"),
                GuiCommand::B => println!("B"),
                GuiCommand::RemoveButton => {
                    let result = self.gui.remove(self.button_id_will_be_removed);
                    println!("button_id_will_be_removed remove result: {:?}", result);
                    let result2 = self.gui.remove(self.layout_id_a);
                    println!("layout_id_a remove result: {:?}", result2);
                }
                GuiCommand::C => println!("C"),
                GuiCommand::D => println!("D"),
                GuiCommand::E => println!("E"),
                GuiCommand::F => {
                    println!("F");
                    let new_sprite = gui::text_sprite(context, "FF", 0.1);
                    self.gui
                        .update_sprite(context, self.button_f_id, new_sprite);
                }
                GuiCommand::NextMap => self.select_next_map(context),
                GuiCommand::Exit => self.exit(context),
            }
        }
    }

    fn handle_event_click(&mut self, _: &mut Context, pos: Point) {
        self.gui.click(pos);
    }
}

impl Screen for GuiTest {
    fn tick(&mut self, context: &mut Context, _: Time) {
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

pub fn main() {
    // so that assets can be accessed correctly
    env::set_current_dir(env::current_dir().unwrap().parent().unwrap()).unwrap();

    let settings = hate::Settings::default();
    let mut visualizer = hate::Visualizer::new(settings);
    let start_screen = Box::new(GuiTest::new(visualizer.context_mut()));
    visualizer.run(start_screen);
}
