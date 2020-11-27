use mq::color::Color;

use crate::{Action, Sprite};

#[derive(Debug)]
pub struct SetColor {
    sprite: Sprite,
    to: Color,
}

impl SetColor {
    pub fn new(sprite: &Sprite, to: Color) -> Self {
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
