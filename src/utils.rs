use std::io::Read;
use std::time::Duration;

use ggez::Context;

use ZResult;

pub fn time_s(s: f32) -> Duration {
    let ms = s * 1000.0;
    Duration::from_millis(ms as u64)
}

pub fn check_assets_hash(context: &mut Context, expected_hash: &str) -> ZResult {
    let mut file = context.filesystem.open("/hash.md5")?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    assert_eq!(data.trim(), expected_hash);
    Ok(())
}
