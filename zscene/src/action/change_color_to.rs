use std::time::Duration;

use mq::color::Color;

use crate::{Action, Sprite};

#[derive(Debug)]
pub struct ChangeColorTo {
    sprite: Sprite,
    from: Color,
    to: Color,
    duration: Duration,
    progress: Duration,
}

impl ChangeColorTo {
    pub fn new(sprite: &Sprite, to: Color, duration: Duration) -> Self {
        Self {
            sprite: sprite.clone(),
            from: sprite.color(),
            to,
            duration,
            progress: Duration::new(0, 0),
        }
    }
}

impl Action for ChangeColorTo {
    fn begin(&mut self) {
        self.from = self.sprite.color();
    }

    fn update(&mut self, mut dtime: Duration) {
        if dtime + self.progress > self.duration {
            dtime = self.duration - self.progress;
        }
        let progress_f = self.progress.as_secs_f32();
        let duration_f = self.duration.as_secs_f32();
        let k = progress_f / duration_f;
        self.sprite.set_color(interpolate(self.from, self.to, k));
        self.progress += dtime;
    }

    fn end(&mut self) {
        self.sprite.set_color(self.to);
    }

    fn duration(&self) -> Duration {
        self.duration
    }

    fn is_finished(&self) -> bool {
        self.progress >= self.duration
    }
}

fn interpolate(from: Color, to: Color, k: f32) -> Color {
    let calc = |a, b| a + (b - a) * k;
    Color {
        r: calc(from.r, to.r),
        g: calc(from.g, to.g),
        b: calc(from.b, to.b),
        a: calc(from.a, to.a),
    }
}
