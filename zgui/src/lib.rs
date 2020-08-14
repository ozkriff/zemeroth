//! Tiny and opinionated GUI.

use std::{
    cell::RefCell,
    error::Error as StdError,
    fmt::{self, Debug},
    rc::Rc,
    sync::mpsc::{channel, Receiver, Sender},
};

use gwg::{
    graphics::{self, Color, Drawable, Point2, Rect, Vector2},
    Context, GameError, GameResult,
};
use log::{info, trace};

pub const SPRITE_COLOR: Color = graphics::BLACK;
pub const SPRITE_COLOR_INACTIVE: Color = Color::new(0.4, 0.4, 0.4, 0.5);
pub const SPRITE_COLOR_BG: Color = Color::new(0.8, 0.8, 0.8, 0.5);
pub const SPRITE_COLOR_BG_HIGHLIGHTED: Color = Color::new(0.9, 0.9, 0.9, 1.0);
pub const SPRITE_COLOR_BUTTON_BORDER: Color = Color::new(1.0, 0.0, 0.0, 0.9);

// TODO: What should we do if some widget changes its size?

// TODO: Add ScrollArea widget

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    GwgError(GameError),
    BadBorderCoefficient,
    BadContentCoefficient,
    NoDimensions,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::GwgError(ref e) => write!(f, "gwg Error: {}", e),
            Error::BadBorderCoefficient => write!(f, "Border size is too large"),
            Error::BadContentCoefficient => write!(f, "Content size is too large"),
            Error::NoDimensions => write!(f, "The drawable has no dimensions"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            Error::GwgError(ref e) => Some(e),
            Error::BadBorderCoefficient | Error::BadContentCoefficient | Error::NoDimensions => {
                None
            }
        }
    }
}

impl From<GameError> for Error {
    fn from(e: GameError) -> Self {
        Error::GwgError(e)
    }
}

fn quad_to_tris<T: Copy>(v: [T; 4]) -> [T; 6] {
    [v[0], v[1], v[2], v[0], v[2], v[3]]
}

pub fn pack<W: Widget + 'static>(widget: W) -> RcWidget {
    Rc::new(RefCell::new(widget))
}

struct Sprite {
    drawable: Box<dyn Drawable>,
    dimensions: Rect,
    basic_scale: f32,
    param: graphics::DrawParam,
}

impl Debug for Sprite {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SpriteData")
            .field("drawable", &format_args!("{:p}", self.drawable))
            .field("dimensions", &self.dimensions)
            .field("basic_scale", &self.basic_scale)
            .field("param", &self.param)
            .finish()
    }
}

impl Sprite {
    fn new(context: &mut Context, drawable: Box<dyn Drawable>, height: f32) -> Result<Self> {
        let dimensions = match drawable.dimensions(context) {
            Some(dimensions) => dimensions,
            None => return Err(Error::NoDimensions),
        };
        let basic_scale = height / dimensions.h;
        let param = graphics::DrawParam {
            scale: [basic_scale, basic_scale].into(),
            color: SPRITE_COLOR,
            ..Default::default()
        };
        Ok(Self {
            drawable,
            dimensions,
            param,
            basic_scale,
        })
    }

    fn draw(&self, context: &mut Context) -> GameResult<()> {
        self.drawable.draw(context, self.param)
    }

    fn rect(&self) -> Rect {
        let w = self.dimensions.w;
        let h = self.dimensions.h;
        // TODO: Transform Drawable's dimensions
        Rect {
            x: self.param.dest.x,
            y: self.param.dest.y,
            w: w * self.param.scale.x,
            h: h * self.param.scale.y,
        }
    }

    fn set_color(&mut self, color: Color) {
        self.param.color = color;
    }

    fn set_pos(&mut self, pos: Point2) {
        self.param.dest = pos.into();
    }
}

fn make_bg(context: &mut Context, rect: Rect) -> Result<Sprite> {
    make_rect(context, rect, SPRITE_COLOR_BG)
}

fn make_rect(context: &mut Context, rect: Rect, color: Color) -> Result<Sprite> {
    let mode = graphics::DrawMode::fill();
    let white = [1.0, 1.0, 1.0, 1.0].into();
    let mesh = graphics::Mesh::new_rectangle(context, mode, rect, white)?;
    let mut sprite = Sprite::new(context, Box::new(mesh), rect.h)?;
    sprite.set_color(color);
    Ok(sprite)
}

pub fn window_to_screen(context: &Context, pos: Point2) -> Point2 {
    let (w, h) = graphics::drawable_size(context);
    let w = w as f32;
    let h = h as f32;
    let aspect_ratio = w / h;
    Point2::new(
        (2.0 * pos.x / w - 1.0) * aspect_ratio,
        2.0 * pos.y / h - 1.0,
    )
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
    fn draw(&self, _: &mut Context) -> GameResult<()>;
    fn click(&self, _: Point2) {}
    fn move_mouse(&mut self, _: Point2) {}
    fn rect(&self) -> Rect;
    fn set_pos(&mut self, pos: Point2);

    fn can_stretch(&self) -> bool {
        false
    }

    fn stretch(&mut self, _: &mut Context, _width: f32) -> Result<StretchStatus> {
        // The default impl assumes the widget can't stretch.
        assert!(!self.can_stretch());
        Ok(StretchStatus::Unstretchable)
    }

    fn stretch_to_self(&mut self, context: &mut Context) -> Result<StretchStatus> {
        let w = self.rect().w;
        self.stretch(context, w)
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
    pub fn new(context: &Context) -> Self {
        let (w, h) = graphics::drawable_size(context);
        let aspect_ratio = w as f32 / h as f32;
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

    pub fn remove(&mut self, widget: &RcWidget) -> GameResult<()> {
        let len_before = self.anchored_widgets.len();
        self.anchored_widgets
            .retain(|w| !Rc::ptr_eq(&w.widget, widget));
        let len_after = self.anchored_widgets.len();
        info!("len_before={}, len_after={}", len_before, len_after);
        if len_after != len_before - 1 {
            panic!("Can't remove the widget");
        }
        Ok(())
    }

    pub fn draw(&self, context: &mut Context) -> GameResult<()> {
        let old_coordinates = graphics::screen_coordinates(context);
        let ui_coordinates = Rect::new(-self.aspect_ratio, -1.0, self.aspect_ratio * 2.0, 2.0);
        graphics::set_screen_coordinates(context, ui_coordinates)?;
        for AnchoredWidget { widget, .. } in &self.anchored_widgets {
            widget.borrow().draw(context)?;
        }
        graphics::set_screen_coordinates(context, old_coordinates)?;
        Ok(())
    }

    pub fn click(&mut self, pos: Point2) -> Option<Message> {
        for AnchoredWidget { widget, .. } in &self.anchored_widgets {
            widget.borrow_mut().click(pos);
        }
        self.receiver.try_recv().ok()
    }

    pub fn move_mouse(&mut self, pos: Point2) {
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
                HAnchor::Left => pos.x = (-ratio) + offset,
                HAnchor::Middle => pos.x = -rect.w / 2.0,
                HAnchor::Right => pos.x = (ratio - rect.w) - offset,
            }
            match anchor.1 {
                VAnchor::Top => pos.y = (-1.0) + offset,
                VAnchor::Middle => pos.y = -rect.h / 2.0,
                VAnchor::Bottom => pos.y = (1.0 - rect.h) - offset,
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
    pub fn new_with_bg(
        context: &mut Context,
        drawable: Box<dyn Drawable>,
        height: f32,
    ) -> Result<Self> {
        let param = LabelParam {
            bg: true,
            ..LabelParam::default()
        };
        Self::from_params(context, drawable, height, param)
    }

    pub fn new(context: &mut Context, drawable: Box<dyn Drawable>, height: f32) -> Result<Self> {
        let param = LabelParam::default();
        Self::from_params(context, drawable, height, param)
    }

    pub fn from_params(
        context: &mut Context,
        drawable: Box<dyn Drawable>,
        height: f32,
        param: LabelParam,
    ) -> Result<Self> {
        param.check()?;
        let sprite = Sprite::new(context, drawable, height * param.drawable_k)?;
        let rect = Rect {
            w: sprite.rect().w,
            h: sprite.rect().h / param.drawable_k,
            ..Default::default()
        };
        let bg = if param.bg {
            Some(make_bg(context, rect)?)
        } else {
            None
        };
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
        self.sprite.param.color = color;
    }
}

impl Widget for Label {
    fn draw(&self, context: &mut Context) -> GameResult<()> {
        if let Some(ref bg) = self.bg {
            bg.draw(context)?;
        }
        self.sprite.draw(context)
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    fn set_pos(&mut self, pos: Point2) {
        let h = (1.0 - self.param.drawable_k) * self.height;
        let w = self.rect.w - self.sprite.rect().w;
        self.sprite.set_pos(pos + Vector2::new(w, h) * 0.5);
        if let Some(ref mut bg) = &mut self.bg {
            bg.set_pos(pos);
        }
        self.rect.move_to(pos);
    }

    fn can_stretch(&self) -> bool {
        self.param.is_stretchable
    }

    fn stretch(&mut self, context: &mut Context, width: f32) -> Result<StretchStatus> {
        if let Some(status) = stretch_checks(self, width) {
            return Ok(status);
        }
        let pos: Point2 = self.rect().point().into();
        let rect = Rect {
            w: width,
            h: self.rect.h,
            ..Default::default()
        };
        self.rect = rect;
        if self.param.bg {
            self.bg = Some(make_bg(context, rect)?);
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
    pub fn new(context: &mut Context, color: Color, rect: Rect) -> Result<Self> {
        Ok(Self {
            sprite: make_rect(context, rect, color)?,
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
    fn draw(&self, context: &mut Context) -> GameResult<()> {
        self.sprite.draw(context)
    }

    fn rect(&self) -> Rect {
        self.sprite.rect()
    }

    fn set_pos(&mut self, pos: Point2) {
        self.sprite.set_pos(pos);
    }

    fn can_stretch(&self) -> bool {
        self.is_stretchable
    }

    fn stretch(&mut self, context: &mut Context, width: f32) -> Result<StretchStatus> {
        if let Some(status) = stretch_checks(self, width) {
            return Ok(status);
        }
        let pos: Point2 = self.rect().point().into();
        let rect = Rect {
            w: width,
            h: self.rect().h,
            ..Default::default()
        };
        self.sprite = make_rect(context, rect, self.color)?;
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
    fn draw(&self, _: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    fn set_pos(&mut self, pos: Point2) {
        self.rect.move_to(pos)
    }

    fn can_stretch(&self) -> bool {
        self.is_stretchable
    }

    fn stretch(&mut self, _: &mut Context, width: f32) -> Result<StretchStatus> {
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
        context: &mut Context,
        drawable: Box<dyn Drawable>,
        height: f32,
        sender: Sender<Message>,
        message: Message,
    ) -> Result<Self> {
        let param = ButtonParam::default();
        Self::from_params(context, drawable, height, sender, message, param)
    }

    pub fn from_params(
        context: &mut Context,
        drawable: Box<dyn Drawable>,
        height: f32,
        sender: Sender<Message>,
        message: Message,
        param: ButtonParam,
    ) -> Result<Self> {
        param.check()?;
        let sprite = Sprite::new(context, drawable, height * param.drawable_k)?;
        let outer = Self::outer_rect(&sprite, height, &param);
        let inner = Self::inner_rect(&param, outer);
        let border = Self::make_border(context, height, outer, inner)?;
        let bg = Self::make_bg_mesh(context, height, outer)?;
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
        self.sprite.param.color = self.color;
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

    fn make_border(context: &mut Context, height: f32, outer: Rect, inner: Rect) -> Result<Sprite> {
        let mut vertices: Vec<[f32; 2]> = Vec::new();
        let outer = rect_to_vertices(outer);
        let inner = rect_to_vertices(inner);
        vertices.extend(quad_to_tris([outer[0], outer[1], inner[1], inner[0]]).iter());
        vertices.extend(quad_to_tris([outer[1], outer[2], inner[2], inner[1]]).iter());
        vertices.extend(quad_to_tris([outer[2], outer[3], inner[3], inner[2]]).iter());
        vertices.extend(quad_to_tris([outer[3], outer[0], inner[0], inner[3]]).iter());
        let color = SPRITE_COLOR_BUTTON_BORDER;
        let border_mesh = graphics::Mesh::from_triangles(context, &vertices, color)?;
        Sprite::new(context, Box::new(border_mesh), height)
    }

    fn make_bg_mesh(context: &mut Context, height: f32, outer: Rect) -> Result<Sprite> {
        let outer = rect_to_vertices(outer);
        let triangles = quad_to_tris(outer);
        let bg_mesh = graphics::Mesh::from_triangles(context, &triangles, graphics::WHITE)?;
        let mut bg = Sprite::new(context, Box::new(bg_mesh), height)?;
        bg.set_color(SPRITE_COLOR_BG);
        Ok(bg)
    }
}

impl<Message: Clone + Debug> Widget for Button<Message> {
    fn draw(&self, context: &mut Context) -> GameResult {
        self.bg.draw(context)?;
        self.sprite.draw(context)?;
        self.border.draw(context)?;
        Ok(())
    }

    fn click(&self, pos: Point2) {
        trace!("Label: rect={:?}, pos={:?}", self.sprite.rect(), pos);
        if self.border.rect().contains(pos) {
            let message = self.message.clone();
            self.sender.send(message).unwrap();
            return;
        }
    }

    fn move_mouse(&mut self, pos: Point2) {
        let highlighted = self.border.rect().contains(pos);
        if highlighted {
            self.bg.param.color = SPRITE_COLOR_BG_HIGHLIGHTED;
        } else {
            self.sprite.param.color = self.color;
            self.bg.param.color = SPRITE_COLOR_BG;
        };
    }

    fn rect(&self) -> Rect {
        self.border.rect()
    }

    fn set_pos(&mut self, pos: Point2) {
        let h = self.border.rect().h - self.sprite.rect().h;
        let w = self.border.rect().w - self.sprite.rect().w;
        self.sprite.set_pos(pos + Vector2::new(w, h) * 0.5);
        self.border.set_pos(pos);
        self.bg.set_pos(pos);
    }

    fn can_stretch(&self) -> bool {
        self.param.is_stretchable
    }

    fn stretch(&mut self, context: &mut Context, width: f32) -> Result<StretchStatus> {
        if let Some(status) = stretch_checks(self, width) {
            return Ok(status);
        }
        let pos: Point2 = self.rect().point().into();
        let height = self.bg.dimensions.h;
        let outer = Rect {
            w: width,
            h: self.rect().h,
            ..Default::default()
        };
        let inner = Self::inner_rect(&self.param, outer);
        self.border = Self::make_border(context, height, outer, inner)?;
        self.bg = Self::make_bg_mesh(context, height, outer)?;
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
    fn draw(&self, context: &mut Context) -> GameResult {
        for widget in &self.widgets {
            widget.draw(context)?;
        }
        Ok(())
    }

    fn click(&self, pos: Point2) {
        for widget in &self.widgets {
            widget.click(pos);
        }
    }

    fn move_mouse(&mut self, pos: Point2) {
        for widget in &mut self.widgets {
            widget.move_mouse(pos);
        }
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    fn set_pos(&mut self, pos: Point2) {
        let point: Point2 = self.rect.point().into();
        let diff = pos - point;
        for widget in &mut self.widgets {
            let pos: Point2 = widget.rect().point().into();
            widget.set_pos(pos + diff);
        }
        self.rect.move_to(pos);
    }

    fn can_stretch(&self) -> bool {
        self.is_stretchable
    }

    fn stretch(&mut self, context: &mut Context, width: f32) -> Result<StretchStatus> {
        if let Some(status) = stretch_checks(self, width) {
            return Ok(status);
        }
        for widget in &mut self.widgets {
            widget.stretch(context, width)?;
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
            pos.y += rect.h;
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
    fn draw(&self, context: &mut Context) -> GameResult {
        self.internal.draw(context)
    }

    fn click(&self, pos: Point2) {
        self.internal.click(pos);
    }

    fn move_mouse(&mut self, pos: Point2) {
        self.internal.move_mouse(pos);
    }

    fn rect(&self) -> Rect {
        self.internal.rect()
    }

    fn set_pos(&mut self, pos: Point2) {
        self.internal.set_pos(pos);
    }

    fn can_stretch(&self) -> bool {
        self.internal.can_stretch()
    }

    fn stretch(&mut self, context: &mut Context, width: f32) -> Result<StretchStatus> {
        self.internal.stretch(context, width)
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
            let mut pos: Point2 = rect.point().into();
            pos.x += rect.w;
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
    fn draw(&self, context: &mut Context) -> GameResult {
        self.internal.draw(context)
    }

    fn click(&self, pos: Point2) {
        self.internal.click(pos);
    }

    fn move_mouse(&mut self, pos: Point2) {
        self.internal.move_mouse(pos);
    }

    fn rect(&self) -> Rect {
        self.internal.rect()
    }

    fn set_pos(&mut self, pos: Point2) {
        self.internal.set_pos(pos);
    }

    fn can_stretch(&self) -> bool {
        self.internal.can_stretch()
    }

    fn stretch(&mut self, context: &mut Context, width: f32) -> Result<StretchStatus> {
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
            let mut pos: Point2 = r.point().into();
            pos.x += diff_w;
            widget.set_pos(pos);
            if widget.can_stretch() {
                let new_w = r.w + additional_w_per_stretchable;
                widget.stretch(context, new_w)?;
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
    fn draw(&self, context: &mut Context) -> GameResult {
        self.internal.draw(context)
    }

    fn click(&self, pos: Point2) {
        self.internal.click(pos);
    }

    fn move_mouse(&mut self, pos: Point2) {
        self.internal.move_mouse(pos);
    }

    fn rect(&self) -> Rect {
        self.internal.rect()
    }

    fn set_pos(&mut self, pos: Point2) {
        self.internal.set_pos(pos);
    }

    fn can_stretch(&self) -> bool {
        self.internal.can_stretch()
    }

    fn stretch(&mut self, context: &mut Context, width: f32) -> Result<StretchStatus> {
        self.internal.stretch(context, width)
    }
}
