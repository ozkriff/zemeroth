//! Tiny and opinionated GUI.
#![allow(warnings)] // TODO: remove

use std::{
    cell::RefCell,
    error::Error as StdError,
    fmt::{self, Debug},
    rc::Rc,
    sync::mpsc::{channel, Receiver, Sender},
};

use log::{info, trace};
use macroquad::prelude::{
    draw_rectangle, draw_rectangle_lines, draw_text_ex, draw_texture_ex, measure_text,
    screen_height, screen_width, vec2, Color, DrawTextureParams, Font, Rect, TextParams, Texture2D,
    Vec2, BLACK, WHITE,
};

pub const SPRITE_COLOR: Color = BLACK;
pub const SPRITE_COLOR_INACTIVE: Color = Color::new_const(102, 102, 102, 127);
pub const SPRITE_COLOR_BG: Color = Color::new_const(204, 204, 204, 127);
pub const SPRITE_COLOR_BG_HIGHLIGHTED: Color = Color::new_const(229, 229, 229, 255);
pub const SPRITE_COLOR_BUTTON_BORDER: Color = Color::new_const(0, 0, 0, 229);

// TODO: What should we do if some widget changes its size?

// TODO: Add ScrollArea widget

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    GwgError,
    BadBorderCoefficient,
    BadContentCoefficient,
    NoDimensions,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::GwgError => write!(f, "gwg Error"),
            Error::BadBorderCoefficient => write!(f, "Border size is too large"),
            Error::BadContentCoefficient => write!(f, "Content size is too large"),
            Error::NoDimensions => write!(f, "The drawable has no dimensions"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            Error::GwgError => None,
            Error::BadBorderCoefficient | Error::BadContentCoefficient | Error::NoDimensions => {
                None
            }
        }
    }
}

fn quad_to_tris<T: Copy>(v: [T; 4]) -> [T; 6] {
    [v[0], v[1], v[2], v[0], v[2], v[3]]
}

pub fn pack<W: Widget + 'static>(widget: W) -> RcWidget {
    Rc::new(RefCell::new(widget))
}

#[derive(Debug, Clone)]
pub enum Drawable {
    Texture(Texture2D),
    Text {
        label: String,
        font: Font,
        font_size: u16,
    },
    SolidRect {
        rect: Rect,
    },
    LinesRect {
        rect: Rect,
        thickness: f32,
    },
}

impl Drawable {
    pub fn text(label: &str, font: Font, font_size: u16) -> Drawable {
        Drawable::Text {
            label: label.to_string(),
            font,
            font_size,
        }
    }

    fn dimensions(&self) -> Rect {
        match self {
            Drawable::Texture(texture) => {
                Rect::new(0.0, 0.0, texture.width() as _, texture.height() as _)
            }
            Drawable::Text {
                label,
                font,
                font_size,
            } => {
                let (w, h) = measure_text(&label, Some(*font), *font_size, 1.0);
                Rect::new(0.0, 0.0, w, h)
            }
            Drawable::SolidRect { rect, .. } => rect.clone(),
            Drawable::LinesRect { rect, .. } => rect.clone(),
        }
    }
}

struct Sprite {
    drawable: Drawable,
    dimensions: Rect,
    basic_scale: f32,

    pos: Vec2,
    scale: Vec2,
    color: Color,
}

impl Debug for Sprite {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SpriteData")
            .field("drawable", &format_args!("{:?}", self.drawable))
            .field("dimensions", &self.dimensions)
            .field("basic_scale", &self.basic_scale)
            //.field("param", &self.param)
            .finish()
    }
}

impl Sprite {
    fn new(drawable: Drawable, height: f32) -> Result<Self> {
        let dimensions = drawable.dimensions();
        let basic_scale = height / dimensions.h;
        Ok(Self {
            drawable,
            dimensions,
            basic_scale,

            pos: vec2(0.0, 0.0),
            scale: vec2(basic_scale, basic_scale),
            color: SPRITE_COLOR,
        })
    }

    fn draw(&self) {
        match &self.drawable {
            Drawable::Texture(texture) => {
                draw_texture_ex(
                    *texture,
                    self.pos.x(),
                    self.pos.y(),
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(
                            self.scale * vec2(texture.width() as f32, texture.height() as f32),
                        ),
                        ..Default::default()
                    },
                );
            }
            Drawable::Text {
                label,
                font,
                font_size,
            } => {
                draw_text_ex(
                    &label,
                    self.pos.x(),
                    // TODO: this actually looks like macroquad bug in text positioning :/
                    self.pos.y() - *font_size as f32 * self.scale.x() * 0.35,
                    TextParams {
                        font_size: *font_size,
                        font: *font,
                        font_scale: self.scale.x(),
                        color: self.color,
                        ..Default::default()
                    },
                );
            }
            Drawable::SolidRect { rect } => {
                draw_rectangle(self.pos.x(), self.pos.y(), rect.w, rect.h, self.color);
            }
            Drawable::LinesRect { rect, thickness } => {
                draw_rectangle_lines(
                    self.pos.x(),
                    self.pos.y(),
                    rect.w,
                    rect.h,
                    *thickness,
                    self.color,
                );
            }
        }
    }

    fn rect(&self) -> Rect {
        let w = self.dimensions.w;
        let h = self.dimensions.h;
        // TODO: Transform Drawable's dimensions
        Rect {
            x: self.pos.x(),
            y: self.pos.y(),
            w: w * self.scale.x(),
            h: h * self.scale.y(),
        }
    }

    fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    fn set_pos(&mut self, pos: Vec2) {
        self.pos = pos.into();
    }
}

fn make_bg(rect: Rect) -> Result<Sprite> {
    make_rect(rect, SPRITE_COLOR_BG)
}

fn make_rect(rect: Rect, color: Color) -> Result<Sprite> {
    let mesh = Drawable::SolidRect { rect };
    let mut sprite = Sprite::new(mesh, rect.h)?;
    sprite.set_color(color);
    Ok(sprite)
}

pub fn window_to_screen(pos: Vec2) -> Vec2 {
    // let (w, h) = graphics::drawable_size();
    // let w = w as f32;
    // let h = h as f32;
    // let aspect_ratio = w / h;
    // Vec2::new(
    //     (2.0 * pos.x / w - 1.0) * aspect_ratio,
    //     2.0 * pos.y / h - 1.0,
    // )
    unimplemented!()
}

#[derive(Clone, Copy, Debug)]
pub enum VAnchor {
    Top,
    Middle,
    Bottom,
}

#[derive(Clone, Copy, Debug)]
pub enum HAnchor {
    Left,
    Middle,
    Right,
}

// TODO: Use some kind of slots? There's no point in having two panes in the same corner.
#[derive(Clone, Copy, Debug)]
pub struct Anchor(pub HAnchor, pub VAnchor);

#[derive(Clone, Copy, Debug)]
pub enum StretchStatus {
    Stretched,
    AlreadyWider,
    Unstretchable,
}

pub trait Widget: Debug {
    fn draw(&self);
    fn click(&self, _: Vec2) {}
    fn move_mouse(&mut self, _: Vec2) {}
    fn rect(&self) -> Rect;
    fn set_pos(&mut self, pos: Vec2);

    fn can_stretch(&self) -> bool {
        false
    }

    fn stretch(&mut self, _width: f32) -> Result<StretchStatus> {
        // The default impl assumes the widget can't stretch.
        assert!(!self.can_stretch());
        Ok(StretchStatus::Unstretchable)
    }

    fn stretch_to_self(&mut self) -> Result<StretchStatus> {
        let w = self.rect().w;
        self.stretch(w)
    }
}

fn stretch_checks(widget: &impl Widget, width: f32) -> Option<StretchStatus> {
    if !widget.can_stretch() {
        return Some(StretchStatus::Unstretchable);
    }
    if widget.rect().w > width {
        return Some(StretchStatus::AlreadyWider);
    }
    None
}

pub type RcWidget = Rc<RefCell<dyn Widget>>;

#[derive(Debug)]
pub struct AnchoredWidget {
    widget: RcWidget,
    anchor: Anchor,
}

#[derive(Debug)]
pub struct Gui<Message: Clone> {
    aspect_ratio: f32,
    anchored_widgets: Vec<AnchoredWidget>,
    receiver: Receiver<Message>,
    sender: Sender<Message>,
}

impl<Message: Clone> Gui<Message> {
    pub fn new() -> Self {
        let (w, h) = (screen_width(), screen_height());
        let aspect_ratio = w / h;
        trace!("Gui: aspect_ratio: {}", aspect_ratio);
        let (sender, receiver) = channel();
        Self {
            anchored_widgets: Vec::new(),
            receiver,
            sender,
            aspect_ratio,
        }
    }

    /// Returns a clone of sender
    pub fn sender(&self) -> Sender<Message> {
        self.sender.clone()
    }

    pub fn add(&mut self, widget: &RcWidget, anchor: Anchor) {
        let widget = widget.clone();
        let anchored_widget = AnchoredWidget { widget, anchor };
        self.anchored_widgets.push(anchored_widget);
        let ratio = self.aspect_ratio;
        self.resize(ratio);
    }

    pub fn remove(&mut self, widget: &RcWidget) {
        let len_before = self.anchored_widgets.len();
        self.anchored_widgets
            .retain(|w| !Rc::ptr_eq(&w.widget, widget));
        let len_after = self.anchored_widgets.len();
        info!("len_before={}, len_after={}", len_before, len_after);
        if len_after != len_before - 1 {
            panic!("Can't remove the widget");
        }
    }

    pub fn draw(&self) {
        use macroquad::prelude::{set_camera, Camera2D};
        //let old_coordinates = graphics::screen_coordinates();
        let ui_coordinates = Rect::new(-self.aspect_ratio, -1.0, self.aspect_ratio * 2.0, 2.0);
        let camera = Camera2D::from_display_rect(ui_coordinates);
        set_camera(camera);
        //graphics::set_screen_coordinates( ui_coordinates)?;
        for AnchoredWidget { widget, .. } in &self.anchored_widgets {
            widget.borrow().draw();
        }
        //graphics::set_screen_coordinates( old_coordinates)?;
    }

    pub fn click(&mut self, pos: Vec2) -> Option<Message> {
        for AnchoredWidget { widget, .. } in &self.anchored_widgets {
            widget.borrow_mut().click(pos);
        }
        self.receiver.try_recv().ok()
    }

    pub fn move_mouse(&mut self, pos: Vec2) {
        for AnchoredWidget { widget, .. } in &self.anchored_widgets {
            widget.borrow_mut().move_mouse(pos);
        }
    }

    pub fn resize(&mut self, ratio: f32) {
        self.aspect_ratio = ratio;
        trace!("Gui::resize: {}", ratio);
        let offset = 0.02; // TODO: make configurable
        for AnchoredWidget { widget, anchor } in &mut self.anchored_widgets {
            let mut widget = widget.borrow_mut();
            let rect = widget.rect();
            let mut pos = rect.point();
            match anchor.0 {
                HAnchor::Left => *pos.x_mut() = (-ratio) + offset,
                HAnchor::Middle => *pos.x_mut() = -rect.w / 2.0,
                HAnchor::Right => *pos.x_mut() = (ratio - rect.w) - offset,
            }
            match anchor.1 {
                VAnchor::Top => *pos.y_mut() = (-1.0) + offset,
                VAnchor::Middle => *pos.y_mut() = -rect.h / 2.0,
                VAnchor::Bottom => *pos.y_mut() = (1.0 - rect.h) - offset,
            }
            widget.set_pos(pos.into());
        }
    }
}

#[derive(Debug, Clone)]
pub struct LabelParam {
    /// Percentage of the drawable's size.
    pub drawable_k: f32,

    pub bg: bool,

    pub is_stretchable: bool,
}

impl Default for LabelParam {
    fn default() -> Self {
        LabelParam {
            drawable_k: 0.8,
            bg: false,
            is_stretchable: false,
        }
    }
}

impl LabelParam {
    pub fn check(&self) -> Result {
        if self.drawable_k < 0.0 || self.drawable_k > 1.0 {
            return Err(Error::BadContentCoefficient);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Label {
    sprite: Sprite,
    bg: Option<Sprite>,
    param: LabelParam,
    rect: Rect,
    height: f32,
}

impl Label {
    pub fn new_with_bg(drawable: Drawable, height: f32) -> Result<Self> {
        let param = LabelParam {
            bg: true,
            ..LabelParam::default()
        };
        Self::from_params(drawable, height, param)
    }

    pub fn new(drawable: Drawable, height: f32) -> Result<Self> {
        let param = LabelParam::default();
        Self::from_params(drawable, height, param)
    }

    pub fn from_params(drawable: Drawable, height: f32, param: LabelParam) -> Result<Self> {
        param.check()?;
        let sprite = Sprite::new(drawable, height * param.drawable_k)?;
        let rect = Rect {
            w: sprite.rect().w,
            h: sprite.rect().h / param.drawable_k,
            ..Default::default()
        };
        let bg = if param.bg { Some(make_bg(rect)?) } else { None };
        Ok(Self {
            sprite,
            bg,
            param,
            height,
            rect,
        })
    }

    pub fn stretchable(mut self, value: bool) -> Self {
        self.set_stretchable(value);
        self
    }

    pub fn set_stretchable(&mut self, value: bool) {
        self.param.is_stretchable = value;
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.set_color(color);
        self
    }

    pub fn set_color(&mut self, color: Color) {
        self.sprite.color = color;
    }
}

impl Widget for Label {
    fn draw(&self) {
        if let Some(ref bg) = self.bg {
            bg.draw();
        }
        self.sprite.draw();
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    fn set_pos(&mut self, pos: Vec2) {
        let h = (1.0 - self.param.drawable_k) * self.height;
        let w = self.rect.w - self.sprite.rect().w;
        self.sprite.set_pos(pos + vec2(w, h) * 0.5);
        if let Some(ref mut bg) = &mut self.bg {
            bg.set_pos(pos);
        }
        self.rect.x = pos.x();
        self.rect.y = pos.y();
    }

    fn can_stretch(&self) -> bool {
        self.param.is_stretchable
    }

    fn stretch(&mut self, width: f32) -> Result<StretchStatus> {
        if let Some(status) = stretch_checks(self, width) {
            return Ok(status);
        }
        let pos: Vec2 = vec2(self.rect().x, self.rect().y);
        let rect = Rect {
            w: width,
            h: self.rect.h,
            x: 0.0,
            y: 0.0,
        };
        self.rect = rect;
        if self.param.bg {
            self.bg = Some(make_bg(rect)?);
        }
        self.set_pos(pos);
        Ok(StretchStatus::Stretched)
    }
}

#[derive(Debug)]
pub struct ColoredRect {
    sprite: Sprite,
    color: Color,
    is_stretchable: bool,
}

impl ColoredRect {
    pub fn new(color: Color, rect: Rect) -> Result<Self> {
        Ok(Self {
            sprite: make_rect(rect, color)?,
            color,
            is_stretchable: false,
        })
    }

    pub fn stretchable(mut self, value: bool) -> Self {
        self.set_stretchable(value);
        self
    }

    pub fn set_stretchable(&mut self, value: bool) {
        self.is_stretchable = value;
    }
}

impl Widget for ColoredRect {
    fn draw(&self) {
        self.sprite.draw()
    }

    fn rect(&self) -> Rect {
        self.sprite.rect()
    }

    fn set_pos(&mut self, pos: Vec2) {
        self.sprite.set_pos(pos);
    }

    fn can_stretch(&self) -> bool {
        self.is_stretchable
    }

    fn stretch(&mut self, width: f32) -> Result<StretchStatus> {
        if let Some(status) = stretch_checks(self, width) {
            return Ok(status);
        }
        let pos: Vec2 = self.rect().point().into();
        let rect = Rect {
            w: width,
            h: self.rect().h,
            ..Default::default()
        };
        self.sprite = make_rect(rect, self.color)?;
        self.set_pos(pos);
        Ok(StretchStatus::Stretched)
    }
}

#[derive(Debug)]
pub struct Spacer {
    rect: Rect,
    is_stretchable: bool,
}

impl Spacer {
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            is_stretchable: false,
        }
    }

    pub fn new_vertical(h: f32) -> Self {
        let rect = Rect {
            h,
            ..Default::default()
        };
        Self {
            rect,
            is_stretchable: false,
        }
    }

    pub fn new_horizontal(w: f32) -> Self {
        let rect = Rect {
            w,
            ..Default::default()
        };
        Self {
            rect,
            is_stretchable: false,
        }
    }

    pub fn stretchable(mut self, value: bool) -> Self {
        self.set_stretchable(value);
        self
    }

    pub fn set_stretchable(&mut self, value: bool) {
        self.is_stretchable = value;
    }
}

impl Widget for Spacer {
    fn draw(&self) {}

    fn rect(&self) -> Rect {
        self.rect
    }

    fn set_pos(&mut self, pos: Vec2) {
        self.rect.move_to(pos)
    }

    fn can_stretch(&self) -> bool {
        self.is_stretchable
    }

    fn stretch(&mut self, width: f32) -> Result<StretchStatus> {
        if let Some(status) = stretch_checks(self, width) {
            return Ok(status);
        }
        self.rect.w = width;
        Ok(StretchStatus::Stretched)
    }
}

#[derive(Debug, Clone)]
pub struct ButtonParam {
    /// Percentage of one border's size.
    pub border_k: f32,

    /// Percentage of the drawable's size.
    pub drawable_k: f32,

    pub is_stretchable: bool,
}

impl Default for ButtonParam {
    fn default() -> Self {
        let label_param = LabelParam::default();
        Self {
            border_k: 0.06,
            drawable_k: label_param.drawable_k,
            is_stretchable: false,
        }
    }
}

impl ButtonParam {
    pub fn check(&self) -> Result {
        if self.drawable_k < 0.0 || self.drawable_k > 1.0 {
            return Err(Error::BadContentCoefficient);
        }
        if self.border_k * 2.0 > 1.0 - self.drawable_k {
            return Err(Error::BadBorderCoefficient);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Button<Message: Clone> {
    is_active: bool,
    sprite: Sprite,
    bg: Sprite,
    border: Sprite,
    param: ButtonParam,
    sender: Sender<Message>,
    message: Message,
    color: Color,
}

fn rect_to_vertices(r: Rect) -> [[f32; 2]; 4] {
    let x = r.x;
    let y = r.y;
    [[x, y], [x, y + r.h], [x + r.w, y + r.h], [x + r.w, y]]
}

impl<Message: Clone + Debug> Button<Message> {
    pub fn new(
        drawable: Drawable,
        height: f32,
        sender: Sender<Message>,
        message: Message,
    ) -> Result<Self> {
        let param = ButtonParam::default();
        Self::from_params(drawable, height, sender, message, param)
    }

    pub fn from_params(
        drawable: Drawable,
        height: f32,
        sender: Sender<Message>,
        message: Message,
        param: ButtonParam,
    ) -> Result<Self> {
        param.check()?;
        let sprite = Sprite::new(drawable, height * param.drawable_k)?;
        let outer = Self::outer_rect(&sprite, height, &param);
        let inner = Self::inner_rect(&param, outer);
        let border = Self::make_border(height, outer, inner)?;
        let bg = Self::make_bg_mesh(height, outer)?;
        Ok(Self {
            is_active: true,
            sprite,
            bg,
            border,
            param,
            sender,
            message,
            color: SPRITE_COLOR,
        })
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
        self.sprite.color = self.color;
    }

    pub fn set_active(&mut self, value: bool) {
        self.is_active = value;
        let color = if value {
            SPRITE_COLOR
        } else {
            SPRITE_COLOR_INACTIVE
        };
        self.set_color(color);
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn stretchable(mut self, value: bool) -> Self {
        self.set_stretchable(value);
        self
    }

    pub fn set_stretchable(&mut self, value: bool) {
        self.param.is_stretchable = value;
    }

    fn outer_rect(sprite: &Sprite, height: f32, param: &ButtonParam) -> Rect {
        let free_area_k = 1.0 - param.drawable_k - param.border_k * 2.0;
        let free_area = height * free_area_k;
        let border = height * param.border_k;
        Rect {
            w: border * 2.0 + free_area + sprite.rect().w,
            h: height,
            ..Default::default()
        }
    }

    fn inner_rect(param: &ButtonParam, rect: Rect) -> Rect {
        let border = rect.h * param.border_k;
        Rect::new(border, border, rect.w - border * 2.0, rect.h - border * 2.0)
    }

    fn make_border(height: f32, outer: Rect, inner: Rect) -> Result<Sprite> {
        let bg_mesh = Drawable::LinesRect {
            rect: outer,
            thickness: (outer.w - inner.w),
            // thickness: (outer.w - inner.w) / 2.,
            // thickness: (outer.w - inner.w) * 8.0,
        };
        let mut bg = Sprite::new(bg_mesh, height)?;
        bg.set_color(SPRITE_COLOR_BUTTON_BORDER);
        Ok(bg)
    }

    fn make_bg_mesh(height: f32, outer: Rect) -> Result<Sprite> {
        let bg_mesh = Drawable::SolidRect { rect: outer };
        let mut bg = Sprite::new(bg_mesh, height)?;
        bg.set_color(SPRITE_COLOR_BG);
        Ok(bg)
    }
}

impl<Message: Clone + Debug> Widget for Button<Message> {
    fn draw(&self) {
        self.bg.draw();
        self.sprite.draw();
        self.border.draw();
    }

    fn click(&self, pos: Vec2) {
        trace!("Label: rect={:?}, pos={:?}", self.sprite.rect(), pos);
        if self.border.rect().contains(pos) {
            let message = self.message.clone();
            self.sender.send(message).unwrap();
            return;
        }
    }

    fn move_mouse(&mut self, pos: Vec2) {
        let highlighted = self.border.rect().contains(pos);
        if highlighted {
            self.bg.color = SPRITE_COLOR_BG_HIGHLIGHTED;
        } else {
            self.sprite.color = self.color;
            self.bg.color = SPRITE_COLOR_BG;
        };
    }

    fn rect(&self) -> Rect {
        self.border.rect()
    }

    fn set_pos(&mut self, pos: Vec2) {
        let h = self.border.rect().h - self.sprite.rect().h;
        let w = self.border.rect().w - self.sprite.rect().w;
        self.sprite.set_pos(pos + Vec2::new(w, h) * 0.5);
        self.border.set_pos(pos);
        self.bg.set_pos(pos);
    }

    fn can_stretch(&self) -> bool {
        self.param.is_stretchable
    }

    fn stretch(&mut self, width: f32) -> Result<StretchStatus> {
        if let Some(status) = stretch_checks(self, width) {
            return Ok(status);
        }
        let pos: Vec2 = self.rect().point().into();
        let height = self.bg.dimensions.h;
        let outer = Rect {
            w: width,
            h: self.rect().h,
            ..Default::default()
        };
        let inner = Self::inner_rect(&self.param, outer);
        self.border = Self::make_border(height, outer, inner)?;
        self.bg = Self::make_bg_mesh(height, outer)?;
        self.set_pos(pos);
        Ok(StretchStatus::Stretched)
    }
}

#[derive(Debug, Default)]
struct Layout {
    widgets: Vec<Box<dyn Widget>>,
    rect: Rect,
    is_stretchable: bool,
}

impl Layout {
    fn new() -> Self {
        Self {
            widgets: Vec::new(),
            rect: Rect::default(),
            is_stretchable: false,
        }
    }

    pub fn set_stretchable(&mut self, value: bool) {
        self.is_stretchable = value;
    }
}

impl Widget for Layout {
    fn draw(&self) {
        for widget in &self.widgets {
            widget.draw();
        }
    }

    fn click(&self, pos: Vec2) {
        for widget in &self.widgets {
            widget.click(pos);
        }
    }

    fn move_mouse(&mut self, pos: Vec2) {
        for widget in &mut self.widgets {
            widget.move_mouse(pos);
        }
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    fn set_pos(&mut self, pos: Vec2) {
        let point: Vec2 = self.rect.point().into();
        let diff = pos - point;
        for widget in &mut self.widgets {
            let pos: Vec2 = widget.rect().point().into();
            widget.set_pos(pos + diff);
        }
        self.rect.move_to(pos);
    }

    fn can_stretch(&self) -> bool {
        self.is_stretchable
    }

    fn stretch(&mut self, width: f32) -> Result<StretchStatus> {
        if let Some(status) = stretch_checks(self, width) {
            return Ok(status);
        }
        for widget in &mut self.widgets {
            widget.stretch(width)?;
            self.rect.w = self.rect.w.max(widget.rect().w);
        }
        Ok(StretchStatus::Stretched)
    }
}

#[derive(Debug, Default)]
pub struct VLayout {
    internal: Layout,
}

impl VLayout {
    pub fn new() -> Self {
        Self {
            internal: Layout::new(),
        }
    }

    pub fn stretchable(mut self, value: bool) -> Self {
        self.internal.set_stretchable(value);
        self
    }

    pub fn from_widget(widget: Box<dyn Widget>) -> Self {
        let mut this = Self::new();
        this.add(widget);
        this
    }

    pub fn add(&mut self, mut widget: Box<dyn Widget>) {
        let rect = widget.rect();
        if let Some(last) = self.internal.widgets.last() {
            let rect = last.rect();
            let mut pos = rect.point();
            *pos.y_mut() += rect.h;
            widget.set_pos(pos.into());
        } else {
            widget.set_pos(self.internal.rect.point().into());
        }
        self.internal.widgets.push(widget);
        self.internal.rect.h += rect.h;
        if self.internal.rect.w < rect.w {
            self.internal.rect.w = rect.w;
        }
    }
}

impl Widget for VLayout {
    fn draw(&self) {
        self.internal.draw()
    }

    fn click(&self, pos: Vec2) {
        self.internal.click(pos);
    }

    fn move_mouse(&mut self, pos: Vec2) {
        self.internal.move_mouse(pos);
    }

    fn rect(&self) -> Rect {
        self.internal.rect()
    }

    fn set_pos(&mut self, pos: Vec2) {
        self.internal.set_pos(pos);
    }

    fn can_stretch(&self) -> bool {
        self.internal.can_stretch()
    }

    fn stretch(&mut self, width: f32) -> Result<StretchStatus> {
        self.internal.stretch(width)
    }
}

#[derive(Debug, Default)]
pub struct HLayout {
    internal: Layout,
}

impl HLayout {
    pub fn new() -> Self {
        Self {
            internal: Layout::new(),
        }
    }

    pub fn stretchable(mut self, value: bool) -> Self {
        self.internal.set_stretchable(value);
        self
    }

    pub fn add(&mut self, mut widget: Box<dyn Widget>) {
        let rect = widget.rect();
        if let Some(last) = self.internal.widgets.last() {
            let rect = last.rect();
            let mut pos: Vec2 = rect.point().into();
            *pos.x_mut() += rect.w;
            widget.set_pos(pos);
        } else {
            widget.set_pos(self.internal.rect.point().into());
        }
        self.internal.rect.w += rect.w;
        if self.internal.rect.h < rect.h {
            self.internal.rect.h = rect.h;
        }
        self.internal.widgets.push(widget);
    }
}

impl Widget for HLayout {
    fn draw(&self) {
        self.internal.draw()
    }

    fn click(&self, pos: Vec2) {
        self.internal.click(pos);
    }

    fn move_mouse(&mut self, pos: Vec2) {
        self.internal.move_mouse(pos);
    }

    fn rect(&self) -> Rect {
        self.internal.rect()
    }

    fn set_pos(&mut self, pos: Vec2) {
        self.internal.set_pos(pos);
    }

    fn can_stretch(&self) -> bool {
        self.internal.can_stretch()
    }

    fn stretch(&mut self, width: f32) -> Result<StretchStatus> {
        if let Some(status) = stretch_checks(self, width) {
            return Ok(status);
        }
        let widgets = &mut self.internal.widgets;
        let stretchable_count = widgets.iter().filter(|w| w.can_stretch()).count();
        let taken_w: f32 = widgets.iter().fold(0.0, |acc, w| acc + w.rect().w);
        let additional_w_per_stretchable = (width - taken_w) / stretchable_count as f32;
        let mut diff_w = 0.0;
        for widget in widgets {
            let r = widget.rect();
            let mut pos: Vec2 = r.point().into();
            *pos.x_mut() += diff_w;
            widget.set_pos(pos);
            if widget.can_stretch() {
                let new_w = r.w + additional_w_per_stretchable;
                widget.stretch(new_w)?;
                diff_w += additional_w_per_stretchable;
            }
        }
        self.internal.rect.w = width;
        Ok(StretchStatus::Stretched)
    }
}

#[derive(Debug, Default)]
pub struct LayersLayout {
    internal: Layout,
}

impl LayersLayout {
    pub fn new() -> Self {
        Self {
            internal: Layout::new(),
        }
    }

    pub fn stretchable(mut self, value: bool) -> Self {
        self.internal.set_stretchable(value);
        self
    }

    pub fn add(&mut self, mut widget: Box<dyn Widget>) {
        let rect = widget.rect();
        widget.set_pos(self.internal.rect.point().into());
        self.internal.widgets.push(widget);
        if self.internal.rect.h < rect.h {
            self.internal.rect.h = rect.h;
        }
        if self.internal.rect.w < rect.w {
            self.internal.rect.w = rect.w;
        }
    }
}

impl Widget for LayersLayout {
    fn draw(&self) {
        self.internal.draw()
    }

    fn click(&self, pos: Vec2) {
        self.internal.click(pos);
    }

    fn move_mouse(&mut self, pos: Vec2) {
        self.internal.move_mouse(pos);
    }

    fn rect(&self) -> Rect {
        self.internal.rect()
    }

    fn set_pos(&mut self, pos: Vec2) {
        self.internal.set_pos(pos);
    }

    fn can_stretch(&self) -> bool {
        self.internal.can_stretch()
    }

    fn stretch(&mut self, width: f32) -> Result<StretchStatus> {
        self.internal.stretch(width)
    }
}
