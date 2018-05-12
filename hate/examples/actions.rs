extern crate cgmath;
extern crate hate;

use std::time::Duration;

use cgmath::vec2;

use hate::geom::Point;
use hate::gui;
use hate::scene::action;
use hate::scene::Layer;
use hate::{Context, Event, Scene, Screen};

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
struct ActionsScreen {
    scene: Scene,
    layers: Layers,
}

impl ActionsScreen {
    fn new(context: &mut Context) -> Self {
        let layers = Layers::default();
        let scene = Scene::new(layers.clone().sorted());
        let mut screen = Self { scene, layers };
        screen.demo_move(context);
        screen.demo_show_hide(context);
        screen
    }

    fn demo_move(&mut self, context: &mut Context) {
        let mut sprite = gui::text_sprite(context, "move", 0.2);
        sprite.set_pos(Point(vec2(0.0, -1.0)));
        let delta = Point(vec2(0.0, 2.0));
        let action = Box::new(action::Sequence::new(vec![
            Box::new(action::Show::new(&self.layers.fg, &sprite)),
            Box::new(action::MoveBy::new(
                &sprite,
                delta,
                Duration::from_millis(2_000),
            )),
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
            Box::new(action::ChangeColorTo::new(
                &sprite,
                visible,
                Duration::from_millis(300),
            )),
            Box::new(action::Sleep::new(Duration::from_millis(1_000))),
            Box::new(action::ChangeColorTo::new(
                &sprite,
                invisible,
                Duration::from_millis(1_000),
            )),
            Box::new(action::Hide::new(&self.layers.fg, &sprite)),
        ]));
        self.scene.add_action(action);
    }
}

impl Screen for ActionsScreen {
    fn tick(&mut self, context: &mut Context, dtime: Duration) {
        self.scene.tick(dtime);
        self.scene.draw(context);
    }

    fn handle_event(&mut self, _: &mut Context, _: Event) {}
}

fn main() {
    let settings = hate::Settings::default();
    let mut visualizer = hate::Visualizer::new(settings);
    let screen = Box::new(ActionsScreen::new(visualizer.context_mut()));
    visualizer.run(screen);
}
