use crate::{Action, Sprite};

#[derive(Debug)]
pub struct SetFrame {
    sprite: Sprite,
    frame_name: String,
}

impl SetFrame {
    pub fn new(sprite: &Sprite, frame_name: impl Into<String>) -> Self {
        let frame_name = frame_name.into();
        assert!(sprite.has_frame(&frame_name));
        let sprite = sprite.clone();
        Self { sprite, frame_name }
    }
}

impl Action for SetFrame {
    fn begin(&mut self) {
        self.sprite.set_frame(&self.frame_name);
    }
}
