extern crate hate;

pub fn main() {
    let settings = hate::Settings::default();
    let _visualizer = hate::Visualizer::new(settings);
}
