use crate::{Action, Facing, Sprite};

#[derive(Debug)]
pub struct SetFacing {
    sprite: Sprite,
    facing: Facing,
}

impl SetFacing {
    pub fn new(sprite: &Sprite, facing: Facing) -> Self {
        let sprite = sprite.clone();
        Self { sprite, facing }
    }
}

impl Action for SetFacing {
    fn begin(&mut self) {
        self.sprite.set_facing(self.facing);
    }
}
