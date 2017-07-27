use hate::{self, Context, Event, Screen, Sprite, Time};
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
    Exit,
}

#[derive(Debug)]
pub struct GuiTest {
    gui: Gui<GuiCommand>,
    button_f_id: gui::Id,
}

impl GuiTest {
    pub fn new(context: &mut Context) -> Self {
        let mut gui = Gui::new(context);

        let _ /*layout_a_id*/ = {
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

        let _ /*layout_c_id*/ = {
            let sprite_a = gui::text_sprite(context, "move: A", 0.1);
            let sprite_b = gui::text_sprite(context, "attack: B", 0.1);
            let sprite_exit = gui::text_sprite(context, "exit", 0.1);
            let sprite_a_id = gui.add_button(context, sprite_a, GuiCommand::A);
            let sprite_b_id = gui.add_button(context, sprite_b, GuiCommand::B);
            let sprite_id_exit = gui.add_button(context, sprite_exit, GuiCommand::Exit);
            let anchor = gui::Anchor {
                vertical: gui::VAnchor::Middle,
                horizontal: gui::HAnchor::Left,
            };
            let direction = gui::Direction::Up;
            gui.add_layout(anchor, direction, vec![
                sprite_a_id,
                sprite_b_id,
                sprite_id_exit,
            ])
        };

        let mut sprite_selection_marker = Sprite::from_path(context, "selection.png", 0.2);
        sprite_selection_marker.set_color([0.0, 0.0, 1.0, 0.8]);

        Self { gui, button_f_id }
    }

    fn exit(&mut self, context: &mut Context) {
        context.add_command(hate::screen::Command::Pop);
    }

    fn handle_commands(&mut self, context: &mut Context) {
        while let Some(command) = self.gui.try_recv() {
            match command {
                GuiCommand::A => println!("A"),
                GuiCommand::B => println!("B"),
                GuiCommand::C => println!("C"),
                GuiCommand::D => println!("D"),
                GuiCommand::E => println!("E"),
                GuiCommand::F => {
                    println!("F");
                    let new_sprite = gui::text_sprite(context, "FF", 0.1);
                    self.gui
                        .update_sprite(context, self.button_f_id, new_sprite);
                }
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
