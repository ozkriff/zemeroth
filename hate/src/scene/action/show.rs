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

