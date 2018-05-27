use ggez::graphics::Point2;
use ggez::timer;
use std::time::Duration;
use {Action, Sprite};

#[derive(Debug)]
pub struct MoveBy {
    sprite: Sprite,
    duration: Duration,
    delta: Point2, // TODO: Vector2
    progress: Duration,
}

impl MoveBy {
    // TODO: delta: Point2 -> Vector2
    pub fn new(sprite: &Sprite, delta: Point2, duration: Duration) -> Self {
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
        let old_pos: Point2 = self.sprite.pos();
        if dtime + self.progress > self.duration {
            dtime = self.duration - self.progress;
        }
        let dtime_f = timer::duration_to_f64(dtime) as f32;
        let duration_f = timer::duration_to_f64(self.duration) as f32;
        // let new_pos = Point(old_pos.0 + dtime_f * self.delta.0 / duration_f);
        let d: Point2 = self.delta * (dtime_f / duration_f);
        let new_pos = old_pos + d.coords;
        self.sprite.set_pos(new_pos);
        self.progress += dtime;
    }

    fn is_finished(&self) -> bool {
        self.progress >= self.duration
    }
}
