use std::time;

// TODO: migrate to `std::time::Duration`
// https://randomascii.wordpress.com/2012/02/13/dont-store-that-in-a-float/
// https://www.reddit.com/r/rust/comments/6dct7o/float_duration_a_simple_floatingpoint_duration/

/// Time in seconds
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Time(pub f32);

impl Time {
    pub fn delta(self, other: Self) -> Self {
        Time(self.0 - other.0)
    }

    pub fn from_duration(duration: time::Duration) -> Self {
        let seconds = duration.as_secs() as f32;
        let nanoseconds = duration.subsec_nanos() as f32;
        Time(seconds + nanoseconds / 1_000_000_000.0)
    }

    pub fn to_duration(self) -> time::Duration {
        time::Duration::from_millis((self.0 * 1000.0) as u64)
    }
}
