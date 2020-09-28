use crate::{Action, Layer, Sprite};

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
        self.layer.add(&self.sprite);
    }
}
