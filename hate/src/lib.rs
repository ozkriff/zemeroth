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

pub mod fs;
pub mod geom;
pub mod gui;
pub mod scene;
pub mod screen;

mod context;
mod event;
mod mesh;
mod pipeline;
mod screen_stack;
mod settings;
mod sprite;
mod text;
mod texture;
mod time;
mod visualizer;

pub use context::Context;
pub use event::Event;
pub use scene::Scene;
pub use screen::Screen;
pub use settings::Settings;
pub use sprite::Sprite;
pub use visualizer::Visualizer;
