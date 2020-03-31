use std::time::Duration;

use cgmath::{Point2, Vector2};
use gwg::{
    conf, event,
    graphics::{self, Font, Rect, Text},
    Context, GameResult,
};
use zscene::{self as zscene, action, Boxed, Layer, Scene, Sprite};

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
    font: Font,
    scene: Scene,
    layers: Layers,
}

impl State {
    fn new(context: &mut Context) -> zscene::Result<Self> {
        let font = graphics::Font::new(context, "/Karla-Regular.ttf")?;
        let layers = Layers::default();
        let scene = Scene::new(layers.clone().sorted());
        let mut this = Self {
            font,
            scene,
            layers,
        };
        this.demo_move(context)?;
        this.demo_show_hide(context)?;
        {
            let (w, h) = graphics::drawable_size(context);
            this.resize(context, w as _, h as _)?;
        }
        Ok(this)
    }

    fn demo_move(&mut self, context: &mut Context) -> zscene::Result {
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

    fn demo_show_hide(&mut self, context: &mut Context) -> zscene::Result {
        let mut sprite = {
            let font_size = 32.0;
            let text = Box::new(Text::new(("some text", self.font, font_size)));
            let mut sprite = Sprite::from_drawable(context, text, 0.1)?;
            sprite.set_pos(Point2::new(0.0, 0.0));
            sprite.set_scale(2.0); // just testing set_size method
            let scale = sprite.scale();
            assert!((scale - 2.0).abs() < 0.001);
            sprite
        };
        let visible = [0.0, 1.0, 0.0, 1.0].into();
        let invisible = graphics::Color { a: 0.0, ..visible };
        sprite.set_color(invisible);
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

    fn resize(&mut self, context: &mut Context, w: f32, h: f32) -> zscene::Result {
        let aspect_ratio = w / h;
        let coordinates = Rect::new(-aspect_ratio, -1.0, aspect_ratio * 2.0, 2.0);
        graphics::set_screen_coordinates(context, coordinates)?;
        Ok(())
    }
}

impl event::EventHandler for State {
    fn update(&mut self, context: &mut Context) -> GameResult {
        let dtime = gwg::timer::delta(context);
        self.scene.tick(dtime);
        Ok(())
    }

    fn draw(&mut self, context: &mut Context) -> GameResult {
        graphics::clear(context, [0.0, 0.0, 0.0, 1.0].into());
        self.scene.draw(context)?;
        graphics::present(context)
    }

    fn resize_event(&mut self, context: &mut Context, w: f32, h: f32) {
        self.resize(context, w, h).expect("Can't resize the window");
    }
}

fn main() -> gwg::GameResult {
    gwg::start(
        conf::Conf {
            physical_root_dir: Some("resources".into()),
            ..Default::default()
        },
        |mut context| Box::new(State::new(&mut context).expect("Can't create the state")),
    )
}
