use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub tap_tolerance: f32,
    pub text_texture_height: f32,
    pub font: Option<PathBuf>,
    pub max_fps: f32,
}
