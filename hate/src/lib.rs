//! Häte2d (Hate2d) is a simple 2d game engine full of _hate_.

// TODO: hide ALL gfx types

#[cfg(target_os = "android")]
extern crate android_glue;

#[macro_use]
extern crate gfx;

#[macro_use]
extern crate serde_derive;

extern crate cgmath;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate png;
extern crate rusttype;

pub mod gui;
pub mod geom;
pub mod screen;
pub mod fs;
pub mod scene;

mod texture;
mod event;
mod mesh;
mod text;
mod pipeline;
mod visualizer;
mod screen_stack;
mod time;
mod sprite;
mod context;
mod settings;

pub use settings::Settings;
pub use visualizer::Visualizer;
pub use sprite::Sprite;
pub use screen::Screen;
pub use context::Context;
pub use event::Event;
pub use scene::Scene;
