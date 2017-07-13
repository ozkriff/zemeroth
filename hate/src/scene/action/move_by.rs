use ::{Time, Sprite};
use scene::Action;
use geom::Point;

#[derive(Debug)]
pub struct MoveBy {
    sprite: Sprite,
    duration: Time,
    delta: Point,
    progress: Time,
}

impl MoveBy {
    pub fn new(sprite: &Sprite, delta: Point, duration: Time) -> Self {
        Self {
            sprite: sprite.clone(),
            delta,
            duration,
            progress: Time(0.0),
        }
    }
}

impl Action for MoveBy {
    fn update(&mut self, mut dtime: Time) {
        let old_pos = self.sprite.pos();
        if dtime.0 + self.progress.0 > self.duration.0 {
            dtime = Time(self.duration.0 - self.progress.0);
        }
        let new_pos = Point(old_pos.0 + dtime.0 * self.delta.0 / self.duration.0);
        self.sprite.set_pos(new_pos);
        self.progress.0 += dtime.0;
    }

    fn is_finished(&self) -> bool {
        let eps = 0.00001;
        self.progress.0 > (self.duration.0 - eps)
    }
}

