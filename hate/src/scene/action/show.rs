use sprite::Sprite;
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
        assert!(!self.layer.has_sprite(&self.sprite)); // TODO: add unit test for this
        let mut data = self.layer.data.borrow_mut();
        data.sprites.push(self.sprite.clone());
    }
}
