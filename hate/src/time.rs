use std::time::Duration;

/// Converts `time::Duration` to `f32` seconds.
pub fn duration_to_f32(duration: Duration) -> f32 {
    let seconds = duration.as_secs() as f32;
    let nanoseconds = duration.subsec_nanos() as f32;
    seconds + nanoseconds / 1_000_000_000.0
}
