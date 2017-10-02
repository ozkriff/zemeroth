extern crate hate;

pub fn main() {
    let settings = hate::Settings {
        text_texture_height: 80.0,
        tap_tolerance: 0.05,
        font: None,
        max_fps: 60.0,
    };
    let _visualizer = hate::Visualizer::new(settings);
}
