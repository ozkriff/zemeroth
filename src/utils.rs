use std::{io::Read, path::Path, process, time::Duration};

use ggez::{filesystem::Filesystem, graphics::Font, Context};
use log::{error, info};
use serde::de::DeserializeOwned;

use crate::{error::ZError, ZResult};

pub fn time_s(s: f32) -> Duration {
    let ms = s * 1000.0;
    Duration::from_millis(ms as u64)
}

pub fn check_assets_hash(_fs: &mut Filesystem, expected: &str) -> ZResult {
    use std::fs::File;
    let mut file = File::open("assets/.checksum.md5")?;

    // TODO: un-comment this when https://github.com/ggez/ggez/issues/570 is fixed
    // let mut file = fs.open("/.checksum.md5")?;

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

pub fn read_file_to_string<P: AsRef<Path>>(context: &mut Context, path: P) -> ZResult<String> {
    let mut buf = String::new();
    let mut file = ggez::filesystem::open(context, path)?;
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

pub fn deserialize_from_file<P, D>(context: &mut Context, path: P) -> ZResult<D>
where
    P: AsRef<Path>,
    D: DeserializeOwned,
{
    let path = path.as_ref();
    let s = read_file_to_string(context, path)?;
    ron::de::from_str(&s).map_err(|e| ZError::from_ron_de_error(e, path.into()))
}

pub fn default_font(context: &mut Context) -> Font {
    Font::new(context, "/OpenSans-Regular.ttf").expect("Can't load the default font")
}

// TODO: Move to some config (https://github.com/ozkriff/zemeroth/issues/424)
pub const fn font_size() -> f32 {
    128.0
}

pub struct LineHeights {
    pub small: f32,
    pub normal: f32,
    pub big: f32,
    pub large: f32,
}

pub fn line_heights() -> LineHeights {
    LineHeights {
        small: 1.0 / 20.0,
        normal: 1.0 / 12.0,
        big: 1.0 / 9.0,
        large: 1.0 / 6.0,
    }
}
