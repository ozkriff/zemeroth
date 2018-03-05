use std::time::Duration;
use scene::Action;

#[derive(Debug)]
pub struct Sleep {
    duration: Duration,
    time: Duration,
}

impl Sleep {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            time: Duration::new(0, 0),
        }
    }
}

impl Action for Sleep {
    fn duration(&self) -> Duration {
        self.duration
    }

    fn is_finished(&self) -> bool {
        self.duration < self.time
    }

    fn update(&mut self, dtime: Duration) {
        self.time += dtime;
    }
}
