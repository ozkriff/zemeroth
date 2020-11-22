use std::time::Duration;

use mq::prelude::Vec2;

use crate::{duration_to_f32, Action, Sprite};

#[derive(Debug)]
pub struct MoveBy {
    sprite: Sprite,
    duration: Duration,
    delta: Vec2,
    progress: Duration,
}

impl MoveBy {
    pub fn new(sprite: &Sprite, delta: Vec2, duration: Duration) -> Self {
        Self {
            sprite: sprite.clone(),
            delta,
            duration,
            progress: Duration::new(0, 0),
        }
    }
}

impl Action for MoveBy {
    fn update(&mut self, mut dtime: Duration) {
        let old_pos = self.sprite.pos();
        if dtime + self.progress > self.duration {
            dtime = self.duration - self.progress;
        }
        let dtime_f = duration_to_f32(dtime);
        let duration_f = duration_to_f32(self.duration);
        let new_pos = old_pos + self.delta * (dtime_f / duration_f);
        self.sprite.set_pos(new_pos);
        self.progress += dtime;
    }

    fn duration(&self) -> Duration {
        self.duration
    }

    fn is_finished(&self) -> bool {
        self.progress >= self.duration
    }
}
