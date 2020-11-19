#![windows_subsystem = "windows"]
#![allow(clippy::eval_order_dependence)] // TODO
#![allow(clippy::clone_on_copy)] // TODO

mod assets;
mod core;
mod error;
mod geom;
mod screen;
mod sprite_info;
mod utils;

use macroquad::{
    camera::{set_camera, Camera2D},
    input,
    prelude::{Rect, Vec2},
    window,
};

type ZResult<T = ()> = Result<T, error::ZError>;

// TODO: Move to utils.rs
fn aspect_ratio() -> f32 {
    window::screen_width() / window::screen_height()
}

// TODO: Move to utils.rs
fn make_and_set_camera(aspect_ratio: f32) -> Camera2D {
    let camera = Camera2D::from_display_rect(Rect {
        x: -aspect_ratio,
        y: -1.0,
        w: aspect_ratio * 2.0,
        h: 2.0,
    });
    set_camera(camera);
    camera
}

// TODO: Move to utils.rs
pub fn get_world_mouse_pos(camera: &Camera2D) -> Vec2 {
    let (x, y) = input::mouse_position();
    camera.screen_to_world(Vec2::new(x, y))
}

// TODO: Do I really need a state at this point? Merge with Screens?
// TODO: Merge this with Screens?
struct MainState {
    screens: screen::Screens,
}

impl MainState {
    fn new() -> ZResult<Self> {
        let start_screen = Box::new(screen::MainMenu::new()?);
        let screens = screen::Screens::new(start_screen)?;
        Ok(Self { screens })
    }

    fn tick(&mut self) -> ZResult {
        let aspect_ratio = aspect_ratio();
        let camera = make_and_set_camera(aspect_ratio);
        self.screens.resize(aspect_ratio)?;
        let pos = get_world_mouse_pos(&camera);
        self.screens.move_mouse(pos)?;
        if input::is_mouse_button_pressed(input::MouseButton::Left) {
            self.screens.click(pos)?;
        }
        self.screens.update()?;
        self.screens.draw()?;
        Ok(())
    }
}

#[macroquad::main("Zemeroth")]
async fn main() {
    // std::env isn't supported on WASM.
    #[cfg(not(target_arch = "wasm32"))]
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }
    env_logger::init();
    quad_rand::srand(macroquad::prelude::miniquad::date::now() as _);
    assets::load_assets().await;
    let mut state = MainState::new().expect("Can't create the main state");
    loop {
        state.tick().expect("Tick failed");
        window::next_frame().await;
    }
}
