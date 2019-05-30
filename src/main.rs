#![windows_subsystem = "windows"]
#![warn(bare_trait_objects)]

#[cfg(not(target_arch = "wasm32"))]
extern crate ggez;
#[cfg(target_arch = "wasm32")]
extern crate good_web_game as ggez;

use ggez::{
    conf, event,
    graphics::{self, Rect},
    nalgebra::Point2,
    Context, GameResult,
};

mod core;
mod error;
mod geom;
mod screen;
mod utils;

type ZResult<T = ()> = Result<T, error::ZError>;

struct MainState {
    screens: screen::Screens,
}

impl MainState {
    fn new(context: &mut Context) -> ZResult<Self> {
        let start_screen = Box::new(screen::MainMenu::new(context)?);
        let screens = screen::Screens::new(start_screen);
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
        self.screens.resize(aspect_ratio);
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
        _: ggez::event::MouseButton,
        x: f32,
        y: f32,
    ) {
        let window_pos = Point2::new(x, y);
        let pos = ui::window_to_screen(context, window_pos);
        self.screens
            .click(context, pos)
            .expect("Can't handle click event");
    }

    fn mouse_motion_event(&mut self, context: &mut Context, x: f32, y: f32, dx: f32, dy: f32) {
        if dx.abs() < 1.0 && dy.abs() < 1.0 {
            // Don't do anything on touch devices
            return;
        }
        let window_pos = Point2::new(x, y);
        let pos = ui::window_to_screen(context, window_pos);
        self.screens
            .move_mouse(context, pos)
            .expect("Can't move the mouse");
    }

    // This functions just overrides the default implementation,
    // because we don't want to quit from the game on `Esc`.
    #[cfg(not(target_arch = "wasm32"))] // <- we cant quit in wasm anyway
    fn key_down_event(&mut self, _: &mut Context, _: event::KeyCode, _: event::KeyMods, _: bool) {}
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> ZResult {
    use ggez::filesystem::Filesystem;
    use log::info;
    use structopt::StructOpt;

    const APP_ID: &str = "zemeroth";
    const APP_AUTHOR: &str = "ozkriff";
    const ASSETS_DIR_NAME: &str = "assets";
    const ASSETS_HASHSUM: &str = "a42fb4a97d6529620dabec3defea8aa9";

    fn enable_backtrace() {
        if std::env::var("RUST_BACKTRACE").is_err() {
            std::env::set_var("RUST_BACKTRACE", "1");
        }
    }

    fn context() -> GameResult<(Context, event::EventsLoop)> {
        let window_conf = conf::WindowSetup::default().title("Zemeroth");
        let window_mode = conf::WindowMode::default().resizable(true);
        ggez::ContextBuilder::new(APP_ID, APP_AUTHOR)
            .window_setup(window_conf)
            .window_mode(window_mode)
            .add_resource_path(ASSETS_DIR_NAME)
            .build()
    }

    // TODO: un-comment when GGEZ's issue is fixed (what issue?)
    fn fs() -> Filesystem {
        Filesystem::new(APP_ID, APP_AUTHOR).expect("Can't create a filesystem")
        // let mut fs = Filesystem::new(APP_ID, APP_AUTHOR).expect("Can't create a filesystem");
        // fs.mount(std::path::Path::new(ASSETS_DIR_NAME), true);
        // fs
    }

    #[derive(StructOpt, Debug)]
    #[structopt(name = "Zemeroth")]
    struct Options {
        /// Only check assets' hash
        #[structopt(long = "check-assets")]
        check_assets: bool,
    }

    let opt = Options::from_args();
    env_logger::init();
    enable_backtrace();
    info!("Checking assets hash file...");
    utils::check_assets_hash(&mut fs(), ASSETS_HASHSUM)?;
    if opt.check_assets {
        // That's it. We don't need to run the game itself
        return Ok(());
    }
    info!("Creating context...");
    let (mut context, mut events_loop) = context()?;
    info!("Creating MainState...");
    let mut state = MainState::new(&mut context)?;
    info!("Starting the main loop...");
    event::run(&mut context, &mut events_loop, &mut state)?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() -> GameResult {
    ggez::start(
        conf::Conf {
            cache: conf::Cache::Index,
            loading: conf::Loading::Embedded,
            ..Default::default()
        },
        |mut context| {
            let state = MainState::new(&mut context).unwrap();
            event::run(context, state)
        },
    )
}
