use std::default::Default;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub tap_tolerance: f32,
    pub text_texture_height: f32,
    pub font: PathBuf,
    pub max_fps: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            tap_tolerance: 0.05,
            text_texture_height: 80.0,
            font: "<embedded>".into(),
            max_fps: 60.0,
        }
    }
}
