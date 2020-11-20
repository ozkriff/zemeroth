#![windows_subsystem = "windows"]

use std::time::Duration;

use macroquad::{self as mq, input, window};

mod assets;
mod core;
mod error;
mod geom;
mod screen;
mod utils;

type ZResult<T = ()> = Result<T, error::ZError>;

struct MainState {
    screens: screen::ScreenStack,
}

impl MainState {
    fn new() -> ZResult<Self> {
        let start_screen = Box::new(screen::MainMenu::new()?);
        let screens = screen::ScreenStack::new(start_screen)?;
        Ok(Self { screens })
    }

    fn tick(&mut self) -> ZResult {
        // Handle possible window resize and create a camera.
        let aspect_ratio = utils::aspect_ratio();
        let camera = utils::make_and_set_camera(aspect_ratio);
        self.screens.resize(aspect_ratio)?;
        // Handle user input events.
        let pos = utils::get_world_mouse_pos(&camera);
        self.screens.move_mouse(pos)?;
        if input::is_mouse_button_pressed(input::MouseButton::Left) {
            self.screens.click(pos)?;
        }
        // Update the game state.
        let dtime = Duration::from_secs_f32(mq::time::get_frame_time());
        self.screens.update(dtime)?;
        // Draw everything.
        mq::window::clear_background(screen::COLOR_SCREEN_BG);
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
