extern crate hate;
extern crate cgmath;
extern crate ron;
extern crate rand;

mod screen;
mod map;
mod core;
mod game_view;
mod visualize;
mod ai;

pub fn main() {
    enable_backtrace();
    let settings = ron::de::from_str(&hate::fs::load_as_string("settings.ron")).unwrap();
    let mut visualizer = hate::Visualizer::new(settings);
    let start_screen = Box::new(screen::MainMenu::new(visualizer.context_mut()));
    visualizer.run(start_screen);
}

fn enable_backtrace() {
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }
}
