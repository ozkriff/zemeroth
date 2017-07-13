use ::{Time, Sprite};
use scene::Action;

#[derive(Debug)]
pub struct ChangeColorTo {
    sprite: Sprite,
    from: [f32; 4],
    to: [f32; 4],
    duration: Time,
    progress: Time,
}

impl ChangeColorTo {
    pub fn new(sprite: &Sprite, to: [f32; 4], duration: Time) -> Self {
        Self {
            sprite: sprite.clone(),
            from: sprite.color(),
            to,
            duration,
            progress: Time(0.0),
        }
    }
}

impl Action for ChangeColorTo {
    fn begin(&mut self) {
        self.from = self.sprite.color();
    }

    fn update(&mut self, mut dtime: Time) {
        if dtime.0 + self.progress.0 > self.duration.0 {
            dtime = Time(self.duration.0 - self.progress.0);
        }
        let k = self.progress.0 / self.duration.0;
        let mut color = [0.0; 4];
        for (i, color_i) in color.iter_mut().enumerate().take(4) {
            let diff = self.to[i] - self.from[i];
            *color_i = self.from[i] + diff * k;
        }
        self.sprite.set_color(color);
        self.progress.0 += dtime.0;
    }

    fn is_finished(&self) -> bool {
        let eps = 0.00001;
        self.progress.0 > (self.duration.0 - eps)
    }
}

