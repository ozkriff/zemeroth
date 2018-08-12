#![windows_subsystem = "windows"]
#![warn(bare_trait_objects)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate zcomponents;

extern crate env_logger;
extern crate ggez;
extern crate ggwp_zgui as ui;
extern crate ggwp_zscene as scene;
extern crate num;
extern crate rand;
extern crate ron;

use ggez::conf;
use ggez::event;
use ggez::graphics::{self, Point2, Rect};
use ggez::{Context, ContextBuilder, GameResult};

mod core;
mod geom;
mod screen;

// TODO: move to some submodule
// TODO: https://github.com/ggez/ggez/issues/384
type ZResult<T = ()> = GameResult<T>;

// TODO: What should we do in case of adding a widget to a nested layout?
// call `gui.resize()`?

// TODO: rename to TopState, Game or something
struct MainState {
    screens: screen::Screens,
}

impl MainState {
    fn new(context: &mut Context) -> ZResult<MainState> {
        let start_screen = Box::new(screen::MainMenu::new(context)?);
        let screens = screen::Screens::new(start_screen);
        let mut this = MainState { screens };
        {
            let (w, h) = graphics::get_drawable_size(context);
            this.resize(context, w, h);
        }
        Ok(this)
    }

    fn resize(&mut self, context: &mut Context, w: u32, h: u32) {
        let aspect_ratio = w as f32 / h as f32;
        let coordinates = Rect::new(-aspect_ratio, -1.0, aspect_ratio * 2.0, 2.0);
        graphics::set_screen_coordinates(context, coordinates).unwrap();
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

    fn resize_event(&mut self, context: &mut Context, w: u32, h: u32) {
        self.resize(context, w, h);
    }

    fn mouse_button_up_event(
        &mut self,
        context: &mut Context,
        _: ggez::event::MouseButton,
        x: i32,
        y: i32,
    ) {
        let window_pos = Point2::new(x as _, y as _);
        let pos = ui::window_to_screen(context, window_pos);
        self.screens
            .click(context, pos)
            .expect("Can't handle click event");
    }
}

fn context() -> Context {
    let window_conf = conf::WindowSetup::default()
        .resizable(true)
        .title("Zemeroth");
    ContextBuilder::new("zemeroth", "ozkriff")
        .window_setup(window_conf)
        .add_resource_path("assets")
        .build()
        .expect("Can't build context")
}

fn main() -> ZResult {
    env_logger::init();
    enable_backtrace();
    info!("Creating context...");
    let mut context = context();
    info!("Creating MainState...");
    let mut state = MainState::new(&mut context)?;
    info!("Starting the main loop...");
    event::run(&mut context, &mut state)
}

fn enable_backtrace() {
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }
}
