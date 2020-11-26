#![allow(dead_code)]

use mq::{
    camera::{set_camera, Camera2D},
    prelude::{Rect, Vec2},
    text::{load_ttf_font, Font},
    texture::{self, Texture2D},
};

pub fn aspect_ratio() -> f32 {
    mq::window::screen_width() / mq::window::screen_height()
}

pub fn make_and_set_camera(aspect_ratio: f32) -> Camera2D {
    let display_rect = Rect {
        x: -aspect_ratio,
        y: -1.0,
        w: aspect_ratio * 2.0,
        h: 2.0,
    };
    let camera = Camera2D::from_display_rect(display_rect);
    set_camera(camera);
    camera
}

pub fn get_world_mouse_pos(camera: &Camera2D) -> Vec2 {
    camera.screen_to_world(mq::input::mouse_position().into())
}

pub struct Assets {
    pub font: Font,
    pub texture: Texture2D,
}

impl Assets {
    pub async fn load() -> Self {
        let font = load_ttf_font("zgui/assets/Karla-Regular.ttf").await;
        let texture = texture::load_texture("zgui/assets/fire.png").await;
        Self { font, texture }
    }
}
