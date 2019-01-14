use std::time::Duration;

use ggez::{
    conf, event,
    nalgebra::{Vector2, Point2},
    graphics::{self, Rect},
    {Context, ContextBuilder, GameResult},
};
use ggwp_zscene::{action, Boxed, Layer, Scene, Sprite};

#[derive(Debug, Clone, Default)]
pub struct Layers {
    pub bg: Layer, // TODO: show how to use layers
    pub fg: Layer,
}

impl Layers {
    fn sorted(self) -> Vec<Layer> {
        vec![self.bg, self.fg]
    }
}

struct State {
    scene: Scene,
    layers: Layers,
}

impl State {
    fn new(context: &mut Context) -> GameResult<Self> {
        let layers = Layers::default();
        let scene = Scene::new(layers.clone().sorted());
        let mut this = Self {
            scene,
            layers,
        };
        this.demo_move(context)?;
        this.demo_show_hide(context)?;
        {
            let (w, h) = graphics::drawable_size(context);
            this.resize(context, w as _, h as _);
        }
        Ok(this)
    }

    fn demo_move(&mut self, context: &mut Context) -> GameResult<()> {
        let mut sprite = Sprite::from_path(context, "/fire.png", 0.5)?;
        sprite.set_pos(Point2::new(0.0, -1.0));
        let delta = Vector2::new(0.0, 1.5);
        let move_duration = Duration::from_millis(2_000);
        let action = action::Sequence::new(vec![
            action::Show::new(&self.layers.fg, &sprite).boxed(),
            action::MoveBy::new(&sprite, delta, move_duration).boxed(),
        ]);
        self.scene.add_action(action.boxed());
        Ok(())
    }

    fn demo_show_hide(&mut self, context: &mut Context) -> GameResult<()> {
        let mut sprite = Sprite::from_path(context, "/fire.png", 0.5)?;
        sprite.set_scale(2.0); // just testing set_size method
        let visible = [0.0, 1.0, 0.0, 1.0].into();
        let invisible = graphics::Color { a: 0.0, ..visible };
        sprite.set_color(invisible);
        sprite.set_centered(true);
        let t = Duration::from_millis(1_000);
        let action = action::Sequence::new(vec![
            action::Show::new(&self.layers.bg, &sprite).boxed(),
            action::ChangeColorTo::new(&sprite, visible, t).boxed(),
            action::Sleep::new(t).boxed(),
            action::ChangeColorTo::new(&sprite, invisible, t).boxed(),
            action::Hide::new(&self.layers.bg, &sprite).boxed(),
        ]);
        self.scene.add_action(action.boxed());
        Ok(())
    }

    fn resize(&mut self, context: &mut Context, w: u32, h: u32) {
        let aspect_ratio = w as f32 / h as f32;
        let coordinates = Rect::new(-aspect_ratio, -1.0, aspect_ratio * 2.0, 2.0);
        graphics::set_screen_coordinates(context, coordinates).unwrap();
    }
}

impl event::EventHandler for State {
    fn update(&mut self, context: &mut Context) -> GameResult<()> {
        let dtime = ggez::timer::delta(context);
        self.scene.tick(dtime);
        Ok(())
    }

    fn draw(&mut self, context: &mut Context) -> GameResult<()> {
        graphics::clear(context, [0.0, 0.0, 0.0, 1.0].into());
        self.scene.draw(context)?;
        graphics::present(context)
    }

    fn resize_event(&mut self, context: &mut Context, w: f32, h: f32) {
        self.resize(context, w as _, h as _);
    }
}

fn main() -> GameResult<()> {
    let title = "ggwp-zscene";
    let window_conf = conf::WindowSetup::default()
        // .resizable(true)
        .title(title);
    let (mut context, mut events_loop) = ContextBuilder::new(title, "ozkriff")
        .window_setup(window_conf)
        .add_resource_path("resources")
        .build()?;
    let mut state = State::new(&mut context)?;
    event::run(&mut context, &mut events_loop, &mut state)
}
