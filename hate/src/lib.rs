//! Häte2d (Hate2d) is a simple 2d game engine full of _hate_.

// TODO: hide ALL gfx types

#[cfg(target_os = "android")]
extern crate android_glue;

#[macro_use]
extern crate gfx;

#[macro_use]
extern crate serde_derive;

extern crate gfx_window_glutin;
extern crate gfx_device_gl;
extern crate rand;
extern crate cgmath;
extern crate glutin;
extern crate png;
extern crate rusttype;
extern crate serde;

pub mod gui;
pub mod geom;
pub mod screen;
pub mod fs;

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
pub use time::Time;
pub use event::Event;
pub use scene::Scene;

// TODO: move to separate files
pub mod scene {
    use std::rc::Rc;
    use std::cell::RefCell;
    use ::{Sprite, Context, Time};

    pub use ::scene::action::Action;

    #[derive(Debug)]
    struct LayerData {
        sprites: Vec<Sprite>,
    }

    #[derive(Debug, Clone)]
    pub struct Layer {
        data: Rc<RefCell<LayerData>>,
    }

    impl Layer {
        #[cfg_attr(feature = "cargo-clippy", allow(new_without_default))]
        pub fn new() -> Self {
            let data = LayerData {
                sprites: Vec::new(),
            };
            Self {
                data: Rc::new(RefCell::new(data)),
            }
        }

        pub fn add(&mut self, sprite: &Sprite) {
            self.data.borrow_mut().sprites.push(sprite.clone());
        }
    }

    #[derive(Debug)]
    pub struct Scene {
        layers: Vec<Layer>,
        interpreter: ActionInterpreter,
    }

    impl Scene {
        pub fn new(layers: Vec<Layer>) -> Self {
            Self {
                layers,
                interpreter: ActionInterpreter::new(),
            }
        }

        pub fn draw(&self, context: &mut Context) {
            let projection_matrix = context.projection_matrix();
            for layer in &self.layers {
                for sprite in &layer.data.borrow().sprites {
                    sprite.draw(context, projection_matrix);
                }
            }
        }

        pub fn add_action(&mut self, action: Box<Action>) {
            self.interpreter.add(action);
        }

        pub fn tick(&mut self, dtime: Time) {
            self.interpreter.tick(dtime);
        }
    }

    #[derive(Debug)]
    struct ActionInterpreter {
        actions: Vec<Box<Action>>,
    }

    impl ActionInterpreter {
        pub fn new() -> Self {
            Self {
                actions: Vec::new(),
            }
        }

        pub fn add(&mut self, mut action: Box<Action>) {
            action.begin();
            self.actions.push(action);
        }

        pub fn tick(&mut self, dtime: Time) {
            let mut forked_actions = Vec::new();
            for action in &mut self.actions {
                action.update(dtime);
                if let Some(forked_action) = action.try_fork() {
                    forked_actions.push(forked_action);
                }
                if action.is_finished() {
                    action.end();
                }
            }
            for action in forked_actions {
                self.add(action);
            }
            self.actions.retain(|action| !action.is_finished());
        }
    }

    pub mod action {
        use std::fmt::Debug;
        use ::Time;

        pub use scene::action::sequence::Sequence;
        pub use scene::action::show::Show;
        pub use scene::action::hide::Hide;
        pub use scene::action::move_by::MoveBy;
        pub use scene::action::fork::Fork;
        pub use scene::action::sleep::Sleep;
        pub use scene::action::change_color_to::ChangeColorTo;
        pub use scene::action::set_color::SetColor;

        pub trait Action: Debug {
            fn begin(&mut self) {}
            fn update(&mut self, _dtime: Time) {}
            fn end(&mut self) {}

            fn try_fork(&mut self) -> Option<Box<Action>> {
                None
            }

            fn is_finished(&self) -> bool {
                true
            }
        }

        mod sequence {
            use std::collections::VecDeque;
            use ::Time;
            use scene::Action;

            #[derive(Debug)]
            pub struct Sequence {
                actions: VecDeque<Box<Action>>,
            }

            impl Sequence {
                pub fn new(actions: Vec<Box<Action>>) -> Self {
                    Self {
                        actions: actions.into(),
                    }
                }

                /// Current action
                fn action(&mut self) -> &mut Action {
                    &mut **self.actions.front_mut().unwrap()
                }

                fn end_current_action_and_start_next(&mut self) {
                    assert!(!self.actions.is_empty());
                    assert!(self.action().is_finished());
                    self.action().end();
                    self.actions.pop_front().unwrap();
                    if !self.actions.is_empty() {
                        self.action().begin();
                    }
                }
            }

            impl Action for Sequence {
                fn begin(&mut self) {
                    if !self.actions.is_empty() {
                        self.action().begin();
                    }
                }

                fn update(&mut self, dtime: Time) {
                    if self.actions.is_empty() {
                        return;
                    }
                    self.action().update(dtime);
                    // Skipping instant actions
                    while !self.actions.is_empty() && self.action().is_finished() {
                        self.end_current_action_and_start_next();
                    }
                }

                fn end(&mut self) {
                    assert!(self.actions.is_empty());
                }

                fn is_finished(&self) -> bool {
                    self.actions.is_empty()
                }

                fn try_fork(&mut self) -> Option<Box<Action>> {
                    if self.actions.is_empty() {
                        return None;
                    }
                    let forked_action = self.action().try_fork();
                    if forked_action.is_some() && self.action().is_finished() {
                        self.end_current_action_and_start_next();
                    }
                    forked_action
                }
            }
        }

        mod fork {
            use scene::Action;

            #[derive(Debug)]
            pub struct Fork {
                action: Option<Box<Action>>,
            }

            impl Fork {
                pub fn new(action: Box<Action>) -> Self {
                    Self {
                        action: Some(action),
                    }
                }
            }

            impl Action for Fork {
                fn try_fork(&mut self) -> Option<Box<Action>> {
                    self.action.take()
                }

                fn is_finished(&self) -> bool {
                    self.action.is_none()
                }

                fn end(&mut self) {
                    assert!(self.action.is_none());
                }
            }
        }

        mod show {
            use ::Sprite;
            use scene::{Layer, Action};

            #[derive(Debug)]
            pub struct Show {
                layer: Layer,
                sprite: Sprite,
            }

            impl Show {
                pub fn new(layer: &Layer, sprite: &Sprite) -> Self {
                    Self {
                        layer: layer.clone(),
                        sprite: sprite.clone(),
                    }
                }
            }

            impl Action for Show {
                fn begin(&mut self) {
                    let mut data = self.layer.data.borrow_mut();
                    data.sprites.push(self.sprite.clone());
                }
            }
        }

        mod hide {
            use ::Sprite;
            use scene::{Layer, Action};

            #[derive(Debug)]
            pub struct Hide {
                layer: Layer,
                sprite: Sprite,
            }

            impl Hide {
                pub fn new(layer: &Layer, sprite: &Sprite) -> Self {
                    Self {
                        layer: layer.clone(),
                        sprite: sprite.clone(),
                    }
                }
            }

            impl Action for Hide {
                fn begin(&mut self) {
                    let mut data = self.layer.data.borrow_mut();
                    data.sprites.retain(|sprite| !self.sprite.is_same(sprite))
                }
            }
        }

        mod move_by {
            use ::{Time, Sprite};
            use scene::Action;
            use geom::Point;

            #[derive(Debug)]
            pub struct MoveBy {
                sprite: Sprite,
                duration: Time,
                delta: Point,
                progress: Time,
            }

            impl MoveBy {
                pub fn new(sprite: &Sprite, delta: Point, duration: Time) -> Self {
                    Self {
                        sprite: sprite.clone(),
                        delta,
                        duration,
                        progress: Time(0.0),
                    }
                }
            }

            impl Action for MoveBy {
                fn update(&mut self, mut dtime: Time) {
                    let old_pos = self.sprite.pos();
                    if dtime.0 + self.progress.0 > self.duration.0 {
                        dtime = Time(self.duration.0 - self.progress.0);
                    }
                    let new_pos = Point(old_pos.0 + dtime.0 * self.delta.0 / self.duration.0);
                    self.sprite.set_pos(new_pos);
                    self.progress.0 += dtime.0;
                }

                fn is_finished(&self) -> bool {
                    let eps = 0.00001;
                    self.progress.0 > (self.duration.0 - eps)
                }
            }
        }

        mod sleep {
            use ::Time;
            use ::scene::Action;

            #[derive(Debug)]
            pub struct Sleep {
                duration: Time,
                time: Time,
            }

            impl Sleep {
                pub fn new(duration: Time) -> Self {
                    Self {
                        duration: duration,
                        time: Time(0.0),
                    }
                }
            }

            impl Action for Sleep {
                fn is_finished(&self) -> bool {
                    self.time.0 / self.duration.0 > 1.0
                }

                fn update(&mut self, dtime: Time) {
                    self.time.0 += dtime.0;
                }
            }
        }

        mod set_color {
            use ::Sprite;
            use scene::Action;

            #[derive(Debug)]
            pub struct SetColor {
                sprite: Sprite,
                to: [f32; 4],
            }

            impl SetColor {
                pub fn new(sprite: &Sprite, to: [f32; 4]) -> Self {
                    Self {
                        sprite: sprite.clone(),
                        to,
                    }
                }
            }

            impl Action for SetColor {
                fn begin(&mut self) {
                    self.sprite.set_color(self.to);
                }
            }
        }

        mod change_color_to {
            use ::{Time, Sprite};
            use scene::Action;

            #[derive(Debug)]
            pub struct ChangeColorTo {
                sprite: Sprite,
                from: [f32; 4],
                to: [f32; 4],
                duration: Time,
                progress: Time,
            }

            impl ChangeColorTo {
                pub fn new(sprite: &Sprite, to: [f32; 4], duration: Time) -> Self {
                    Self {
                        sprite: sprite.clone(),
                        from: sprite.color(),
                        to,
                        duration,
                        progress: Time(0.0),
                    }
                }
            }

            impl Action for ChangeColorTo {
                fn begin(&mut self) {
                    self.from = self.sprite.color();
                }

                fn update(&mut self, mut dtime: Time) {
                    if dtime.0 + self.progress.0 > self.duration.0 {
                        dtime = Time(self.duration.0 - self.progress.0);
                    }
                    let k = self.progress.0 / self.duration.0;
                    let mut color = [0.0; 4];
                    for (i, color_i) in color.iter_mut().enumerate().take(4) {
                        let diff = self.to[i] - self.from[i];
                        *color_i = self.from[i] + diff * k;
                    }
                    self.sprite.set_color(color);
                    self.progress.0 += dtime.0;
                }

                fn is_finished(&self) -> bool {
                    let eps = 0.00001;
                    self.progress.0 > (self.duration.0 - eps)
                }
            }
        }

        // TODO: change size
        // TODO: change rotation
        // TODO: Easing
    }
}
