#![windows_subsystem = "windows"]
#![warn(bare_trait_objects)]

use ggez::{
    conf, event,
    filesystem::Filesystem,
    graphics::{self, Rect},
    nalgebra::Point2,
    Context, GameResult,
};
use log::info;
#[cfg(not(target_arch = "wasm32"))]
use structopt::StructOpt;

mod core;
mod geom;
mod screen;
mod utils;

// TODO: Remove it with a real error type.
// TODO: https://github.com/ggez/ggez/issues/384
type ZResult<T = ()> = GameResult<T>;

const APP_ID: &str = "zemeroth";
const APP_AUTHOR: &str = "ozkriff";
const ASSETS_DIR_NAME: &str = "assets";

#[allow(dead_code)] // TODO: remove the attribute
const ASSETS_HASHSUM: &str = "18e7de361e74471aeaec3f209ef63c3e";

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
    fn update(&mut self, context: &mut Context) -> ZResult {
        self.screens.update(context)
    }

    fn draw(&mut self, context: &mut Context) -> ZResult {
        self.screens.draw(context)
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

    // This functions just overrides the default implementation,
    // because we don't want to quit from the game on `Esc`.
    #[cfg(not(target_arch = "wasm32"))] // we cant quit in wasm anyway
    fn key_down_event(&mut self, _: &mut Context, _: event::KeyCode, _: event::KeyMods, _: bool) {}
}

#[cfg(not(target_arch = "wasm32"))]
fn context() -> GameResult<(Context, event::EventsLoop)> {
    let window_conf = conf::WindowSetup::default().title("Zemeroth");
    let window_mode = conf::WindowMode::default().resizable(true);
    ggez::ContextBuilder::new(APP_ID, APP_AUTHOR)
        .window_setup(window_conf)
        .window_mode(window_mode)
        .add_resource_path(ASSETS_DIR_NAME)
        .build()
}

#[allow(dead_code)] // TODO: remove the attribute
fn fs() -> Filesystem {
    // let mut fs = Filesystem::new(APP_ID, APP_AUTHOR).expect("Can't create a filesystem");
    // fs.mount(std::path::Path::new(ASSETS_DIR_NAME), true);
    // fs
    unimplemented!() // TODO
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> ZResult {
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
    // utils::check_assets_hash(&mut fs(), ASSETS_HASHSUM)?; // TODO: un-comment later
    if opt.check_assets {
        // That's it. We don't need to run the game itself
        return Ok(());
    }
    info!("Creating context...");
    let (mut context, mut events_loop) = context()?;
    info!("Creating MainState...");
    let mut state = MainState::new(&mut context)?;
    info!("Starting the main loop...");
    event::run(&mut context, &mut events_loop, &mut state)
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

fn enable_backtrace() {
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }
}
