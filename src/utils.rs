use std::{io::Read, path::Path, process, sync::mpsc::Receiver, time::Duration};

use gwg::{
    graphics::{Font, Rect},
    Context,
};
use log;
use serde::de::DeserializeOwned;

use crate::{error::ZError, ZResult};

pub fn time_s(s: f32) -> Duration {
    let ms = s * 1000.0;
    Duration::from_millis(ms as u64)
}

pub fn check_assets_hash(context: &mut Context, expected: &str) -> ZResult {
    let data = read_file(context, "/.checksum.md5")?;
    let real = data.trim();
    if real != expected {
        let error_code = 1;
        log::error!("Bad assets checksum {} (expected {})", real, expected);
        process::exit(error_code);
    }
    log::info!("Assets checksum is Ok");
    Ok(())
}

/// Read a file to a string.
pub fn read_file<P: AsRef<Path>>(context: &mut Context, path: P) -> ZResult<String> {
    let mut buf = String::new();
    let mut file = gwg::filesystem::open(context, path)?;
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

pub fn deserialize_from_file<P, D>(context: &mut Context, path: P) -> ZResult<D>
where
    P: AsRef<Path>,
    D: DeserializeOwned,
{
    let path = path.as_ref();
    let s = read_file(context, path)?;
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

pub fn wrap_widget_and_add_bg(
    context: &mut Context,
    w: Box<dyn ui::Widget>,
) -> ZResult<ui::LayersLayout> {
    let bg = ui::ColoredRect::new(context, ui::SPRITE_COLOR_BG, w.rect())?;
    let mut layers = ui::LayersLayout::new();
    layers.add(Box::new(bg));
    layers.add(w);
    Ok(layers)
}

pub fn pack_widget_into_offset_table(w: Box<dyn ui::Widget>, offset: f32) -> Box<dyn ui::Widget> {
    let spacer = || {
        ui::Spacer::new(Rect {
            w: offset,
            h: offset,
            ..Default::default()
        })
    };
    let mut layout_h = ui::HLayout::new();
    layout_h.add(Box::new(spacer()));
    layout_h.add(w);
    layout_h.add(Box::new(spacer()));
    let mut layout_v = ui::VLayout::new();
    layout_v.add(Box::new(spacer()));
    layout_v.add(Box::new(layout_h));
    layout_v.add(Box::new(spacer()));
    Box::new(layout_v)
}

pub fn remove_widget<M: Clone>(gui: &mut ui::Gui<M>, widget: &mut Option<ui::RcWidget>) -> ZResult {
    if let Some(w) = widget.take() {
        gui.remove(&w)?;
    }
    Ok(())
}

pub fn try_receive<Message>(opt_rx: &Option<Receiver<Message>>) -> Option<Message> {
    opt_rx.as_ref().and_then(|rx| rx.try_recv().ok())
}
