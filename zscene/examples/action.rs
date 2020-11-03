use std::time::Duration;

use macroquad::prelude::*;

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
    async fn new() -> zscene::Result<Self> {
        let font = load_ttf_font("./resources/Karla-Regular.ttf").await;
        let layers = Layers::default();
        let scene = Scene::new(layers.clone().sorted());
        let mut this = Self {
            font,
            scene,
            layers,
        };
        this.demo_move().await?;
        this.demo_show_hide()?;
        Ok(this)
    }

    async fn demo_move(&mut self) -> zscene::Result {
        let mut sprite = Sprite::from_path("./resources/fire.png", 0.5).await?;
        sprite.set_pos(Vec2::new(0.0, -1.0));
        let delta = Vec2::new(0.0, 1.5);
        let move_duration = Duration::from_millis(2_000);
        let action = action::Sequence::new(vec![
            action::Show::new(&self.layers.fg, &sprite).boxed(),
            action::MoveBy::new(&sprite, delta, move_duration).boxed(),
        ]);
        self.scene.add_action(action.boxed());
        Ok(())
    }

    fn demo_show_hide(&mut self) -> zscene::Result {
        let mut sprite = {
            let font_size = 32;
            let mut sprite = Sprite::from_text(("some text", self.font, font_size), 0.1)?;
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
        self.scene.add_action(action.boxed());
        Ok(())
    }
}

#[macroquad::main("Text")]
async fn main() {
    let mut state = State::new().await.expect("Can't create the state");

    loop {
        clear_background(BLACK);

        {
            let w = screen_width();
            let h = screen_height();
            let aspect_ratio = w / h;
            let coordinates = Rect::new(-aspect_ratio, -1.0, aspect_ratio * 2.0, 2.0);
            let camera = Camera2D::from_display_rect(coordinates);

            set_camera(camera);
        }

        let dtime = get_frame_time();

        state.scene.tick(Duration::from_secs_f32(dtime));
        state.scene.draw();

        next_frame().await;
    }
}
