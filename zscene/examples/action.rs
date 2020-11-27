use std::time::Duration;

use mq::{
    camera::{set_camera, Camera2D},
    color::{Color, BLACK},
    math::{glam::Vec2, Rect},
    text,
    texture::{self, Texture2D},
    time, window,
};
use zscene::{self, action, Action, Boxed, Layer, Scene, Sprite};

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

struct Assets {
    font: text::Font,
    texture: Texture2D,
}

impl Assets {
    async fn load() -> Self {
        let font = text::load_ttf_font("zscene/assets/Karla-Regular.ttf").await;
        let texture = texture::load_texture("zscene/assets/fire.png").await;
        Self { font, texture }
    }
}

struct State {
    assets: Assets,
    scene: Scene,
    layers: Layers,
}

impl State {
    fn new(assets: Assets) -> Self {
        let layers = Layers::default();
        let scene = Scene::new(layers.clone().sorted());
        Self {
            assets,
            scene,
            layers,
        }
    }

    fn action_demo_move(&self) -> Box<dyn Action> {
        let mut sprite = Sprite::from_texture(self.assets.texture, 0.5);
        sprite.set_pos(Vec2::new(0.0, -1.0));
        let delta = Vec2::new(0.0, 1.5);
        let move_duration = Duration::from_millis(2_000);
        let action = action::Sequence::new(vec![
            action::Show::new(&self.layers.fg, &sprite).boxed(),
            action::MoveBy::new(&sprite, delta, move_duration).boxed(),
        ]);
        action.boxed()
    }

    fn action_demo_show_hide(&self) -> Box<dyn Action> {
        let mut sprite = {
            let font_size = 32;
            let mut sprite = Sprite::from_text(("some text", self.assets.font, font_size), 0.1);
            sprite.set_pos(Vec2::new(0.0, 0.0));
            sprite.set_scale(2.0); // just testing set_size method
            let scale = sprite.scale();
            assert!((scale - 2.0).abs() < 0.001);
            sprite
        };
        let visible = Color::new(0.0, 1.0, 0.0, 1.0);
        let invisible = Color::new(0.0, 1.0, 0.0, 0.0);
        sprite.set_color(invisible);
        let t = Duration::from_millis(1_000);
        let action = action::Sequence::new(vec![
            action::Show::new(&self.layers.bg, &sprite).boxed(),
            action::ChangeColorTo::new(&sprite, visible, t).boxed(),
            action::Sleep::new(t).boxed(),
            action::ChangeColorTo::new(&sprite, invisible, t).boxed(),
            action::Hide::new(&self.layers.bg, &sprite).boxed(),
        ]);
        action.boxed()
    }
}

fn update_aspect_ratio() {
    let aspect_ratio = window::screen_width() / window::screen_height();
    let coordinates = Rect::new(-aspect_ratio, -1.0, aspect_ratio * 2.0, 2.0);
    set_camera(Camera2D::from_display_rect(coordinates));
}

#[mq::main("ZScene: Actions Demo")]
#[macroquad(crate_rename = "mq")]
async fn main() {
    let assets = Assets::load().await;
    let mut state = State::new(assets);
    {
        // Run two demo demo actions in parallel.
        state.scene.add_action(state.action_demo_move());
        state.scene.add_action(state.action_demo_show_hide());
    }
    loop {
        window::clear_background(BLACK);
        update_aspect_ratio();
        let dtime = time::get_frame_time();
        state.scene.tick(Duration::from_secs_f32(dtime));
        state.scene.draw();
        window::next_frame().await;
    }
}
