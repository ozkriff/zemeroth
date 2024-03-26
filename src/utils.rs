use std::{sync::mpsc::Receiver, time::Duration};

use mq::{
    camera::{set_camera, Camera2D},
    math::{Rect, Vec2},
};

use crate::ZResult;

pub fn time_s(s: f32) -> Duration {
    let ms = s * 1000.0;
    Duration::from_millis(ms as u64)
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
    let bg = ui::ColoredRect::new(ui::SPRITE_COLOR_BG, w.rect()).stretchable(true);
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

pub fn aspect_ratio() -> f32 {
    mq::window::screen_width() / mq::window::screen_height()
}

pub fn make_and_set_camera(aspect_ratio: f32) -> Camera2D {
    let camera = Camera2D::from_display_rect(Rect {
        x: -aspect_ratio,
        y: 1.0,
        w: aspect_ratio * 2.0,
        h: -2.0,
    });
    set_camera(&camera);
    camera
}

pub fn get_world_mouse_pos(camera: &Camera2D) -> Vec2 {
    camera.screen_to_world(mq::input::mouse_position().into())
}

pub fn try_receive<Message>(opt_rx: &Option<Receiver<Message>>) -> Option<Message> {
    opt_rx.as_ref().and_then(|rx| rx.try_recv().ok())
}
