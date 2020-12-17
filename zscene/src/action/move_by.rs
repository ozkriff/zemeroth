use std::time::Duration;

use mq::math::Vec2;

use crate::{Action, Sprite};

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
        let dtime_f = dtime.as_secs_f32();
        let duration_f = self.duration.as_secs_f32();
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
