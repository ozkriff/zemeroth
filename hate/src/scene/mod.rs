use std::rc::Rc;
use std::cell::RefCell;
use sprite::Sprite;
use context::Context;
use time::Time;

pub use scene::action::Action;

pub mod action;

#[derive(Debug)]
struct LayerData {
    sprites: Vec<Sprite>,
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
        self.data.borrow_mut().sprites.push(sprite.clone());
    }

    pub fn has_sprite(&self, sprite: &Sprite) -> bool {
        let data = self.data.borrow();
        data.sprites
            .iter()
            .any(|other| other.is_same(sprite))
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
