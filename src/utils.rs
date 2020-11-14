use std::{sync::mpsc::Receiver, time::Duration};

use macroquad::{file::load_file, prelude::Rect};
use serde::de::DeserializeOwned;

use crate::{error::ZError, ZResult};

pub fn time_s(s: f32) -> Duration {
    let ms = s * 1000.0;
    Duration::from_millis(ms as u64)
}

// TODO: move to assets.rs and make private?
/// Read a file to a string.
pub async fn read_file(path: &str) -> ZResult<String> {
    // TODO: impl from for zerror
    let data = load_file(path).await.unwrap();
    let string = String::from_utf8_lossy(&data[..]).to_owned().to_string();
    Ok(string)
}

// TODO: move to assets.rs and make private?
pub async fn deserialize_from_file<D: DeserializeOwned>(path: &str) -> ZResult<D> {
    let s = read_file(path).await?;
    ron::de::from_str(&s).map_err(|e| ZError::from_ron_de_error(e, path.into()))
}

// TODO: Move to some config (https://github.com/ozkriff/zemeroth/issues/424)
pub const fn font_size() -> u16 {
    128
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

pub const OFFSET_SMALL: f32 = 0.02;
pub const OFFSET_BIG: f32 = 0.04;

pub fn add_bg(w: Box<dyn ui::Widget>) -> ZResult<ui::LayersLayout> {
    let bg = ui::ColoredRect::new(ui::SPRITE_COLOR_BG, w.rect())?.stretchable(true);
    let mut layers = ui::LayersLayout::new();
    layers.add(Box::new(bg));
    layers.add(w);
    Ok(layers)
}

pub fn add_offsets(w: Box<dyn ui::Widget>, offset: f32) -> Box<dyn ui::Widget> {
    let spacer = || {
        ui::Spacer::new(Rect {
            w: offset,
            h: offset,
            ..Default::default()
        })
    };
    let mut layout_h = ui::HLayout::new().stretchable(true);
    layout_h.add(Box::new(spacer()));
    layout_h.add(w);
    layout_h.add(Box::new(spacer()));
    let mut layout_v = ui::VLayout::new().stretchable(true);
    layout_v.add(Box::new(spacer()));
    layout_v.add(Box::new(layout_h));
    layout_v.add(Box::new(spacer()));
    Box::new(layout_v)
}

pub fn add_offsets_and_bg(w: Box<dyn ui::Widget>, offset: f32) -> ZResult<ui::LayersLayout> {
    add_bg(add_offsets(w, offset))
}

pub fn add_offsets_and_bg_big(w: Box<dyn ui::Widget>) -> ZResult<ui::LayersLayout> {
    add_offsets_and_bg(w, OFFSET_BIG)
}

pub fn remove_widget<M: Clone>(gui: &mut ui::Gui<M>, widget: &mut Option<ui::RcWidget>) -> ZResult {
    if let Some(w) = widget.take() {
        gui.remove(&w);
    }
    Ok(())
}

pub fn try_receive<Message>(opt_rx: &Option<Receiver<Message>>) -> Option<Message> {
    opt_rx.as_ref().and_then(|rx| rx.try_recv().ok())
}
