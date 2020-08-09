#![windows_subsystem = "windows"]

use gwg::{
    conf::Conf,
    event,
    graphics::{self, Point2, Rect},
    Context, GameResult,
};

mod core;
mod error;
mod geom;
mod screen;
mod sprite_info;
mod utils;

const ASSETS_HASHSUM: &str = "cf0e1e21e434c36e1d896d6c26b03204";

type ZResult<T = ()> = Result<T, error::ZError>;

struct MainState {
    screens: screen::Screens,
}

impl MainState {
    fn new(context: &mut Context) -> ZResult<Self> {
        let start_screen = Box::new(screen::MainMenu::new(context)?);
        let screens = screen::Screens::new(context, start_screen)?;
        let mut this = Self { screens };
        {
            let (w, h) = graphics::drawable_size(context);
            this.resize(context, w as _, h as _);
        }
        Ok(this)
    }

    fn resize(&mut self, context: &mut Context, w: f32, h: f32) {
        let aspect_ratio = w / h;
        let coordinates = Rect::new(-aspect_ratio, -1.0, aspect_ratio * 2.0, 2.0);
        graphics::set_screen_coordinates(context, coordinates).expect("Can't resize the window");
        self.screens
            .resize(context, aspect_ratio)
            .expect("Can't resize screens");
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, context: &mut Context) -> GameResult {
        self.screens.update(context).expect("Update call failed");
        Ok(())
    }

    fn draw(&mut self, context: &mut Context) -> GameResult {
        self.screens.draw(context).expect("Draw call failed");
        Ok(())
    }

    fn resize_event(&mut self, context: &mut Context, w: f32, h: f32) {
        self.resize(context, w, h);
    }

    fn mouse_button_up_event(
        &mut self,
        context: &mut Context,
        _: gwg::event::MouseButton,
        x: f32,
        y: f32,
    ) {
        let window_pos = Point2::new(x, y);
        let pos = ui::window_to_screen(context, window_pos);
        self.screens
            .click(context, pos)
            .expect("Can't handle click event");
    }

    fn mouse_motion_event(&mut self, context: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        let window_pos = Point2::new(x, y);
        let pos = ui::window_to_screen(context, window_pos);
        self.screens
            .move_mouse(context, pos)
            .expect("Can't move the mouse");
    }

    // This functions just overrides the default implementation,
    // because we don't want to quit from the game on `Esc`.
    fn key_down_event(&mut self, _: &mut Context, _: event::KeyCode, _: event::KeyMods, _: bool) {}
}

#[cfg(not(target_arch = "wasm32"))]
fn conf() -> Conf {
    Conf {
        physical_root_dir: Some("assets".into()),
        ..Default::default()
    }
}

#[cfg(target_arch = "wasm32")]
fn conf() -> Conf {
    Conf {
        cache: gwg::conf::Cache::Tar(include_bytes!("../assets.tar").to_vec()),
        loading: gwg::conf::Loading::Embedded,
        ..Default::default()
    }
}

fn main() -> gwg::GameResult {
    #[cfg(not(target_arch = "wasm32"))]
    {
        // std::env isn't supported on WASM.
        if std::env::var("RUST_BACKTRACE").is_err() {
            std::env::set_var("RUST_BACKTRACE", "1");
        }
    }
    env_logger::init();
    gwg::start(conf(), |mut context| {
        log::info!("Checking assets hash file...");
        utils::check_assets_hash(context, ASSETS_HASHSUM).expect("Wrong assets check sum");
        log::info!("Creating MainState...");
        let state = MainState::new(&mut context).expect("Can't create the main state");
        log::info!("Starting the main loop...");
        Box::new(state)
    })
}
