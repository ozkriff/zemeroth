use std::{cell::RefCell, fmt, rc::Rc, time::Duration};

pub use crate::{
    action::{Action, Boxed},
    sprite::{Facing, Sprite},
};

pub mod action;

mod sprite;

pub type Result<T = ()> = std::result::Result<T, Error>;

pub fn duration_to_f64(d: Duration) -> f64 {
    d.as_secs() as f64 + d.subsec_nanos() as f64 * 1e-9
}

#[derive(Debug)]
pub enum Error {
    GwgError, // TODO: rename. maybe remove.
    NoDimensions,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::GwgError => write!(f, "gwg Error"),
            Error::NoDimensions => write!(f, "The drawable has no dimensions"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::GwgError => None,
            Error::NoDimensions => None,
        }
    }
}

// impl From<gwg::GameError> for Error {
//     fn from(e: gwg::GameError) -> Self {
//         Error::GwgError(e)
//     }
// }

#[derive(Debug)]
struct SpriteWithZ {
    sprite: Sprite,
    z: f32,
}

#[derive(Debug)]
struct LayerData {
    sprites: Vec<SpriteWithZ>,
}

#[derive(Debug, Clone)]
pub struct Layer {
    data: Rc<RefCell<LayerData>>,
}

impl Layer {
    pub fn new() -> Self {
        let data = LayerData {
            sprites: Vec::new(),
        };
        Self {
            data: Rc::new(RefCell::new(data)),
        }
    }

    pub fn add(&mut self, sprite: &Sprite) {
        let sprite = SpriteWithZ {
            sprite: sprite.clone(),
            z: 0.0,
        };
        self.data.borrow_mut().sprites.push(sprite);
        self.sort();
    }

    pub fn set_z(&mut self, sprite: &Sprite, z: f32) {
        {
            let sprites = &mut self.data.borrow_mut().sprites;
            let sprite = sprites
                .iter_mut()
                .find(|other| other.sprite.is_same(sprite))
                .expect("can't find the sprite");
            sprite.z = z;
        }
        self.sort();
    }

    fn sort(&mut self) {
        let sprites = &mut self.data.borrow_mut().sprites;
        sprites.sort_by(|a, b| a.z.partial_cmp(&b.z).expect("can't find the sprite"));
    }

    pub fn remove(&mut self, sprite: &Sprite) {
        let mut data = self.data.borrow_mut();
        data.sprites.retain(|other| !sprite.is_same(&other.sprite))
    }

    pub fn has_sprite(&self, sprite: &Sprite) -> bool {
        let sprites = &self.data.borrow_mut().sprites;
        sprites.iter().any(|other| other.sprite.is_same(sprite))
    }
}

impl Default for Layer {
    fn default() -> Self {
        Self::new()
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

    pub fn draw(&self) {
        for layer in &self.layers {
            for z_sprite in &layer.data.borrow().sprites {
                z_sprite.sprite.draw();
            }
        }
    }

    pub fn add_action(&mut self, action: Box<dyn Action>) {
        self.interpreter.add(action);
    }

    pub fn tick(&mut self, dtime: Duration) {
        self.interpreter.tick(dtime);
    }
}

#[derive(Debug)]
struct ActionInterpreter {
    actions: Vec<Box<dyn Action>>,
}

impl ActionInterpreter {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    pub fn add(&mut self, mut action: Box<dyn Action>) {
        action.begin();
        self.actions.push(action);
    }

    pub fn tick(&mut self, dtime: Duration) {
        let mut forked_actions = Vec::new();
        for action in &mut self.actions {
            action.update(dtime);
            while let Some(forked_action) = action.try_fork() {
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
