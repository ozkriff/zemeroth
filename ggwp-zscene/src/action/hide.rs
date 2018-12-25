use crate::{Action, Layer, Sprite};

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
        self.layer.remove(&self.sprite);
    }
}
