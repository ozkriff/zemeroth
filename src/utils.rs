use std::io::Read;
use std::process;
use std::time::Duration;

use ggez::filesystem::Filesystem;

use ZResult;

pub fn time_s(s: f32) -> Duration {
    let ms = s * 1000.0;
    Duration::from_millis(ms as u64)
}

pub fn check_assets_hash(fs: &mut Filesystem, expected: &str) -> ZResult {
    let mut file = fs.open("/.checksum.md5")?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let real = data.trim();
    if real != expected {
        let error_code = 1;
        error!("Bad assets checksum {} (expected {})", real, expected);
        process::exit(error_code);
    }
    info!("Assets checksum is Ok");
    Ok(())
}
