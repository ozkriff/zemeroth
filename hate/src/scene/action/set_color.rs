use sprite::Sprite;
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
