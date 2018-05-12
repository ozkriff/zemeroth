use std::time::Duration;

use geom::Point;
use scene::Action;
use sprite::Sprite;
use time;

#[derive(Debug)]
pub struct MoveBy {
    sprite: Sprite,
    duration: Duration,
    delta: Point,
    progress: Duration,
}

impl MoveBy {
    pub fn new(sprite: &Sprite, delta: Point, duration: Duration) -> Self {
        Self {
            sprite: sprite.clone(),
            delta,
            duration,
            progress: Duration::new(0, 0),
        }
    }
}

impl Action for MoveBy {
    fn duration(&self) -> Duration {
        self.duration
    }

    fn update(&mut self, mut dtime: Duration) {
        let old_pos = self.sprite.pos();
        if dtime + self.progress > self.duration {
            dtime = self.duration - self.progress;
        }
        let dtime_f = time::duration_to_f32(dtime);
        let duration_f = time::duration_to_f32(self.duration);
        let new_pos = Point(old_pos.0 + dtime_f * self.delta.0 / duration_f);
        self.sprite.set_pos(new_pos);
        self.progress += dtime;
    }

    fn is_finished(&self) -> bool {
        self.progress >= self.duration
    }
}
