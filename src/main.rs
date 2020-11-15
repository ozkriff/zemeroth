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

struct MainState {
    screens: screen::Screens,
}

impl MainState {
    fn new() -> ZResult<Self> {
        let start_screen = Box::new(screen::MainMenu::new()?);
        let screens = screen::Screens::new(start_screen)?;
        let this = Self { screens };
        Ok(this)
    }

    #[allow(dead_code)] // TODO
    fn resize(&mut self, _w: f32, _h: f32) {} // TODO: ??
}

//     fn resize_event(&mut self, w: f32, h: f32) {
//         self.resize(w, h);
//     }

//     fn mouse_motion_event(&mut self, x: f32, y: f32, _dx: f32, _dy: f32) {
//         let window_pos = Vec2::new(x, y);
//         let pos = ui::window_to_screen(window_pos);
//         self.screens
//             .move_mouse(pos)
//             .expect("Can't move the mouse");
//     }

//     fn mouse_button_up_event(
//         &mut self,
//
//         _: gwg::event::MouseButton,
//         x: f32,
//         y: f32,
//     ) {
//     }

//     // This functions just overrides the default implementation,
//     // because we don't want to quit from the game on `Esc`.
//     fn key_down_event(&mut self, _: &mut Context, _: event::KeyCode, _: event::KeyMods, _: bool) {}
// }

// #[cfg(not(target_arch = "wasm32"))]
// fn conf() -> Conf {
//     Conf {
//         physical_root_dir: Some("assets".into()),
//         ..Default::default()
//     }
// }

// #[cfg(target_arch = "wasm32")]
// fn conf() -> Conf {
//     Conf {
//         cache: gwg::conf::Cache::Tar(include_bytes!("../assets.tar").to_vec()),
//         loading: gwg::conf::Loading::Embedded,
//         ..Default::default()
//     }
// }

// fn main() -> gwg::GameResult {
//     #[cfg(not(target_arch = "wasm32"))]
//     {
//         // std::env isn't supported on WASM.
//         if std::env::var("RUST_BACKTRACE").is_err() {
//             std::env::set_var("RUST_BACKTRACE", "1");
//         }
//     }
//     env_logger::init();
//     quad_rand::srand(gwg::timer::time() as _);
//     gwg::start(conf(), |context| {
//         log::info!("Increasing the default font size...");
//         gwg::graphics::set_font_size(120);
//         log::info!("Creating MainState...");
//
//         log::info!("Starting the main loop...");
//         Box::new(state)
//     })
// }

#[macroquad::main("Zemeroth")]
async fn main() {
    // TODO: init logger
    // TODO: init random!
    assets::load_assets().await;
    let mut state = MainState::new().expect("Can't create the main state");

    loop {
        state.screens.update().expect("Update call failed");

        state.screens.draw().expect("Draw call failed");

        // TODO: extract helper function
        let aspect_ratio = window::screen_width() / window::screen_height();
        let coordinates = Rect::new(-aspect_ratio, -1.0, aspect_ratio * 2.0, 2.0);
        let camera = Camera2D::from_display_rect(coordinates);
        set_camera(camera);
        state
            .screens
            .resize(aspect_ratio)
            .expect("Can't resize screens");

        if input::is_mouse_button_pressed(input::MouseButton::Left) {
            let window_pos = input::mouse_position();
            let pos = camera.screen_to_world(Vec2::new(window_pos.0, window_pos.1));
            state.screens.click(pos).expect("Can't handle click event");
        }
        window::next_frame().await;
    }
}
