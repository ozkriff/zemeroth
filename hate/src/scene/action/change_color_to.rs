use std::time::Duration;

use scene::Action;
use sprite::Sprite;
use time;

#[derive(Debug)]
pub struct ChangeColorTo {
    sprite: Sprite,
    from: [f32; 4],
    to: [f32; 4],
    duration: Duration,
    progress: Duration,
}

impl ChangeColorTo {
    pub fn new(sprite: &Sprite, to: [f32; 4], duration: Duration) -> Self {
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
    fn duration(&self) -> Duration {
        self.duration
    }

    fn begin(&mut self) {
        self.from = self.sprite.color();
    }

    fn end(&mut self) {
        self.sprite.set_color(self.to);
    }

    fn update(&mut self, mut dtime: Duration) {
        if dtime + self.progress > self.duration {
            dtime = self.duration - self.progress;
        }
        let progress_f = time::duration_to_f32(self.progress);
        let duration_f = time::duration_to_f32(self.duration);
        let k = progress_f / duration_f;
        let mut color = [0.0; 4];
        for (i, color_i) in color.iter_mut().enumerate().take(4) {
            let diff = self.to[i] - self.from[i];
            *color_i = self.from[i] + diff * k;
        }
        self.sprite.set_color(color);
        self.progress += dtime;
    }

    fn is_finished(&self) -> bool {
        self.progress >= self.duration
    }
}
