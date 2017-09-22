use cgmath::vec2;
use hate::{self, Context, Event, Scene, Screen, Time};
use hate::geom::Point;
use hate::scene::Layer;
use hate::scene::action;
use hate::gui::{self, Gui};

#[derive(Copy, Clone, Debug)]
enum GuiCommand {
    ShowHide,
    Move,
    Exit,
}

#[derive(Debug, Clone, Default)]
pub struct Layers {
    pub bg: Layer,
    pub fg: Layer,
}

impl Layers {
    fn sorted(self) -> Vec<Layer> {
        vec![self.bg, self.fg]
    }
}

#[derive(Debug)]
pub struct ActionsTest {
    gui: Gui<GuiCommand>,
    scene: Scene,
    layers: Layers,
}

impl ActionsTest {
    pub fn new(context: &mut Context) -> Self {
        let mut gui = Gui::new(context);
        let layers = Layers::default();
        let scene = Scene::new(layers.clone().sorted());
        {
            let sprite_exit = gui::text_sprite(context, "exit", 0.1);
            let sprite_show_hide = gui::text_sprite(context, "show/hide", 0.1);
            let sprite_move = gui::text_sprite(context, "move", 0.1);
            let button_id_exit = gui.add_button(context, sprite_exit, GuiCommand::Exit);
            let button_id_show_hide =
                gui.add_button(context, sprite_show_hide, GuiCommand::ShowHide);
            let button_id_move = gui.add_button(context, sprite_move, GuiCommand::Move);
            let anchor = gui::Anchor {
                vertical: gui::VAnchor::Bottom,
                horizontal: gui::HAnchor::Right,
            };
            let direction = gui::Direction::Up;
            gui.add_layout(
                anchor,
                direction,
                vec![button_id_exit, button_id_show_hide, button_id_move],
            );
        }
        Self { gui, scene, layers }
    }

    fn exit(&mut self, context: &mut Context) {
        context.add_command(hate::screen::Command::Pop);
    }

    fn demo_move(&mut self, context: &mut Context) {
        let mut sprite = gui::text_sprite(context, "move", 0.2);
        sprite.set_pos(Point(vec2(0.0, -1.0)));
        let delta = Point(vec2(0.0, 2.0));
        let action = Box::new(action::Sequence::new(vec![
            Box::new(action::Show::new(&self.layers.fg, &sprite)),
            Box::new(action::MoveBy::new(&sprite, delta, Time(2.0))),
            Box::new(action::Hide::new(&self.layers.fg, &sprite)),
        ]));
        self.scene.add_action(action);
    }

    fn demo_show_hide(&mut self, context: &mut Context) {
        let visible = [0.0, 0.0, 0.0, 1.0];
        let invisible = [0.0, 0.0, 0.0, 0.0];
        let mut sprite = gui::text_sprite(context, "abc", 0.3);
        sprite.set_color(invisible);
        let action = Box::new(action::Sequence::new(vec![
            Box::new(action::Show::new(&self.layers.fg, &sprite)),
            Box::new(action::ChangeColorTo::new(&sprite, visible, Time(0.3))),
            Box::new(action::Sleep::new(Time(1.0))),
            Box::new(action::ChangeColorTo::new(&sprite, invisible, Time(1.0))),
            Box::new(action::Hide::new(&self.layers.fg, &sprite)),
        ]));
        self.scene.add_action(action);
    }

    fn handle_commands(&mut self, context: &mut Context) {
        while let Some(command) = self.gui.try_recv() {
            match command {
                GuiCommand::Move => self.demo_move(context),
                GuiCommand::ShowHide => self.demo_show_hide(context),
                GuiCommand::Exit => self.exit(context),
            }
        }
    }

    fn handle_event_click(&mut self, _: &mut Context, pos: Point) {
        self.gui.click(pos);
    }
}

impl Screen for ActionsTest {
    fn tick(&mut self, context: &mut Context, dtime: Time) {
        self.scene.tick(dtime);
        self.scene.draw(context);
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
