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
        assert!(self.layer.has_sprite(&self.sprite)); // TODO: add unit test for this
        let mut data = self.layer.data.borrow_mut();
        data.sprites.retain(|sprite| !self.sprite.is_same(sprite))
    }
}

