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

    // TODO: remove empty lines
    fn tick(&mut self) -> ZResult {
        let aspect_ratio = aspect_ratio();
        let camera = make_and_set_camera(aspect_ratio);
        self.screens.resize(aspect_ratio)?;

        // TODO: extract helper func?
        let window_pos = input::mouse_position();
        let window_pos = Vec2::new(window_pos.0, window_pos.1);
        let pos = camera.screen_to_world(window_pos);
        self.screens.move_mouse(pos)?;
        if input::is_mouse_button_pressed(input::MouseButton::Left) {
            self.screens.click(pos)?;
        }

        self.screens.update()?;
        self.screens.draw()?;

        Ok(())
    }
}

// TODO: clean this all up.

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

// TODO: remove empty lines
#[macroquad::main("Zemeroth")]
async fn main() {
    // TODO: init logger
    env_logger::init();

    // TODO: init random!
    // quad_rand::srand(gwg::timer::time() as _);

    assets::load_assets().await;

    // TODO: Do I really need a state at this point? Merge with Screens?
    let mut state = MainState::new().expect("Can't create the main state");

    // TODO: handle mouse motion (highlight tiles, buttons, etc)
    loop {
        state.tick().expect("Tick failed");
        window::next_frame().await;
    }
}
