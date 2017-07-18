use time::Time;
use scene::Action;

#[derive(Debug)]
pub struct Sleep {
    duration: Time,
    time: Time,
}

impl Sleep {
    pub fn new(duration: Time) -> Self {
        Self {
            duration: duration,
            time: Time(0.0),
        }
    }
}

impl Action for Sleep {
    fn duration(&self) -> Time {
        self.duration
    }

    fn is_finished(&self) -> bool {
        self.time.0 / self.duration.0 > 1.0
    }

    fn update(&mut self, dtime: Time) {
        self.time.0 += dtime.0;
    }
}

