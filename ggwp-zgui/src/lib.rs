#[cfg(not(target_arch = "wasm32"))]
extern crate ggez;
#[cfg(target_arch = "wasm32")]
extern crate good_web_game as ggez;

/// Tiny and opinionated GUI
use std::{
    cell::RefCell,
    fmt::{self, Debug},
    rc::Rc,
    sync::mpsc::{channel, Receiver, Sender},
};

use ggez::{
    graphics::{self, Color, Drawable, Image, Rect},
    Context, GameResult,
};
use log::{debug, info};
use nalgebra::Point2;

pub use error::Error;

pub type Result<T = ()> = std::result::Result<T, Error>;

const SPRITE_COLOR: Color = graphics::BLACK;
const SPRITE_COLOR_HIGHLIGHTED: Color = graphics::BLACK;
const SPRITE_COLOR_BG: Color = Color {
    r: 0.8,
    g: 0.8,
    b: 0.8,
    a: 0.5,
};
const SPRITE_COLOR_BG_HIGHLIGHTED: Color = Color {
    r: 0.9,
    g: 0.9,
    b: 0.9,
    a: 1.0,
};

// TODO: What should we do if some widget changes its size?

pub fn pack<W: Widget + 'static>(widget: W) -> RcWidget {
    Rc::new(RefCell::new(widget))
}

mod error {
    use std::{error::Error as StdError, fmt};

    use ggez::GameError;

    #[derive(Debug)]
    pub enum Error {
        GgezError(GameError),
        NoDimensions,
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                Error::GgezError(ref e) => write!(f, "GGEZ Error: {}", e),
                Error::NoDimensions => write!(f, "The drawable has no dimensions"),
            }
        }
    }

    impl StdError for Error {
        fn source(&self) -> Option<&(dyn StdError + 'static)> {
            match *self {
                Error::GgezError(ref e) => Some(e),
                Error::NoDimensions => None,
            }
        }
    }

    impl From<GameError> for Error {
        fn from(e: GameError) -> Self {
            Error::GgezError(e)
        }
    }
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

    fn clone_with_another_drawable(&self, drawable: Box<dyn Drawable>) -> Self {
        Self {
            drawable,
            dimensions: self.dimensions,
            param: self.param,
            basic_scale: self.basic_scale,
        }
    }

    // TODO: Add some method to change or switch the drawable.

    fn draw(&self, context: &mut Context) -> GameResult<()> {
        self.drawable.draw(context, self.param)
    }

    fn rect(&self) -> Rect {
        let w = self.dimensions.w;
        let h = self.dimensions.h;
        // TODO: Transform Drawable 's dimensions
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

    fn set_pos(&mut self, pos: Point2<f32>) {
        self.param.dest = pos.into();
    }
}

fn make_bg(context: &mut Context, sprite: &Sprite) -> Result<Sprite> {
    let h = sprite.dimensions.h.ceil();
    let w = sprite.dimensions.w.ceil();
    assert!(h > 0.0, "h = {}", h);
    assert!(w > 0.0, "w = {}", w);
    let count = w as usize * h as usize * 4;
    let data: Vec<u8> = [255, 255, 255, 255]
        .iter()
        .cloned()
        .cycle()
        .take(count)
        .collect();
    let image = Image::from_rgba8(context, w as _, h as _, &data)?;
    let mut bg = sprite.clone_with_another_drawable(Box::new(image));
    bg.set_color(SPRITE_COLOR_BG);
    Ok(bg)
}

pub fn window_to_screen(context: &Context, pos: Point2<f32>) -> Point2<f32> {
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

pub trait Widget: Debug {
    fn draw(&self, _: &mut Context) -> GameResult<()>;
    fn click(&self, _: Point2<f32>) {}
    fn move_mouse(&mut self, _: Point2<f32>) {}
    fn rect(&self) -> Rect;
    fn set_pos(&mut self, pos: Point2<f32>);
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
        debug!("Gui: aspect_ratio: {}", aspect_ratio);
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

    pub fn click(&mut self, pos: Point2<f32>) -> Option<Message> {
        for AnchoredWidget { widget, .. } in &self.anchored_widgets {
            widget.borrow_mut().click(pos);
        }
        self.receiver.try_recv().ok()
    }

    pub fn move_mouse(&mut self, pos: Point2<f32>) {
        for AnchoredWidget { widget, .. } in &self.anchored_widgets {
            widget.borrow_mut().move_mouse(pos);
        }
    }

    pub fn resize(&mut self, ratio: f32) {
        self.aspect_ratio = ratio;
        debug!("Gui::resize: {}", ratio);
        for AnchoredWidget { widget, anchor } in &mut self.anchored_widgets {
            let mut widget = widget.borrow_mut();
            let rect = widget.rect();
            let mut pos = rect.point();
            match anchor.0 {
                HAnchor::Left => pos.x = -ratio,
                HAnchor::Middle => pos.x = -rect.w / 2.0,
                HAnchor::Right => pos.x = ratio - rect.w,
            }
            match anchor.1 {
                VAnchor::Top => pos.y = -1.0,
                VAnchor::Middle => pos.y = -rect.h / 2.0,
                VAnchor::Bottom => pos.y = 1.0 - rect.h,
            }
            widget.set_pos(pos.into());
        }
    }
}

#[derive(Debug)]
pub struct Label {
    sprite: Sprite,
    bg: Sprite,
}

impl Label {
    pub fn new(context: &mut Context, drawable: Box<dyn Drawable>, height: f32) -> Result<Self> {
        let sprite = Sprite::new(context, drawable, height)?;
        let bg = make_bg(context, &sprite)?;
        Ok(Self { sprite, bg })
    }
}

impl Widget for Label {
    fn draw(&self, context: &mut Context) -> GameResult<()> {
        self.bg.draw(context)?;
        self.sprite.draw(context)
    }

    fn rect(&self) -> Rect {
        self.sprite.rect()
    }

    fn set_pos(&mut self, pos: Point2<f32>) {
        self.sprite.set_pos(pos);
        self.bg.set_pos(pos);
    }
}

#[derive(Debug)]
pub struct Spacer {
    rect: Rect,
}

impl Spacer {
    pub fn new(rect: Rect) -> Self {
        Self { rect }
    }
}

impl Widget for Spacer {
    fn draw(&self, _: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    fn set_pos(&mut self, pos: Point2<f32>) {
        self.rect.move_to(pos)
    }
}

#[derive(Debug)]
pub struct Button<Message: Clone> {
    sprite: Sprite,
    bg: Sprite,
    sender: Sender<Message>,
    message: Message,
}

impl<Message: Clone> Button<Message> {
    pub fn new(
        context: &mut Context,
        drawable: Box<dyn Drawable>,
        height: f32,
        sender: Sender<Message>,
        message: Message,
    ) -> Result<Self> {
        let sprite = Sprite::new(context, drawable, height)?;
        let bg = make_bg(context, &sprite)?;
        Ok(Self {
            sprite,
            bg,
            sender,
            message,
        })
    }
}

impl<Message: Clone + Debug> Widget for Button<Message> {
    fn draw(&self, context: &mut Context) -> GameResult<()> {
        self.bg.draw(context)?;
        self.sprite.draw(context)
    }

    fn click(&self, pos: Point2<f32>) {
        debug!("Label: rect={:?}, pos={:?}", self.sprite.rect(), pos);
        if self.sprite.rect().contains(pos) {
            let message = self.message.clone();
            self.sender.send(message).unwrap();
            return;
        }
    }

    fn move_mouse(&mut self, pos: Point2<f32>) {
        let highlighted = self.sprite.rect().contains(pos);
        if highlighted {
            self.sprite.param.color = SPRITE_COLOR_HIGHLIGHTED;
            self.bg.param.color = SPRITE_COLOR_BG_HIGHLIGHTED;
        } else {
            self.sprite.param.color = SPRITE_COLOR;
            self.bg.param.color = SPRITE_COLOR_BG;
        };
    }

    fn rect(&self) -> Rect {
        self.sprite.rect()
    }

    fn set_pos(&mut self, pos: Point2<f32>) {
        self.sprite.set_pos(pos);
        self.bg.set_pos(pos);
    }
}

#[derive(Debug, Default)]
pub struct VLayout {
    widgets: Vec<Box<dyn Widget>>,
    rect: Rect,
}

impl VLayout {
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
            rect: Rect::default(),
        }
    }

    pub fn add(&mut self, mut widget: Box<dyn Widget>) {
        let rect = widget.rect();
        if let Some(last) = self.widgets.last() {
            let rect = last.rect();
            let mut pos = rect.point();
            pos.y += rect.h;
            widget.set_pos(pos.into());
        } else {
            widget.set_pos(self.rect.point().into());
        }
        self.widgets.push(widget);
        self.rect.h += rect.h;
        if self.rect.w < rect.w {
            self.rect.w = rect.w;
        }
    }
}

impl Widget for VLayout {
    fn draw(&self, context: &mut Context) -> GameResult<()> {
        for widget in &self.widgets {
            widget.draw(context)?;
        }
        Ok(())
    }

    fn click(&self, pos: Point2<f32>) {
        for widget in &self.widgets {
            widget.click(pos);
        }
    }

    fn move_mouse(&mut self, pos: Point2<f32>) {
        for widget in &mut self.widgets {
            widget.move_mouse(pos);
        }
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    fn set_pos(&mut self, pos: Point2<f32>) {
        let point: Point2<f32> = self.rect.point().into();
        let diff = pos - point;
        for widget in &mut self.widgets {
            let pos: Point2<f32> = widget.rect().point().into();
            widget.set_pos(pos + diff);
        }
        self.rect.move_to(pos);
    }
}

#[derive(Debug, Default)]
pub struct HLayout {
    widgets: Vec<Box<dyn Widget>>,
    rect: Rect,
}

impl HLayout {
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
            rect: Rect::default(),
        }
    }

    pub fn add(&mut self, mut widget: Box<dyn Widget>) {
        let rect = widget.rect();
        if let Some(last) = self.widgets.last() {
            let rect = last.rect();
            let mut pos: Point2<f32> = rect.point().into();
            pos.x += rect.w;
            widget.set_pos(pos);
        } else {
            widget.set_pos(self.rect.point().into());
        }
        self.widgets.push(widget);
        self.rect.w += rect.w;
        if self.rect.h < rect.h {
            self.rect.h = rect.h;
        }
    }
}

impl Widget for HLayout {
    fn draw(&self, context: &mut Context) -> GameResult<()> {
        for widget in &self.widgets {
            widget.draw(context)?;
        }
        Ok(())
    }

    fn click(&self, pos: Point2<f32>) {
        for widget in &self.widgets {
            widget.click(pos);
        }
    }

    fn move_mouse(&mut self, pos: Point2<f32>) {
        for widget in &mut self.widgets {
            widget.move_mouse(pos);
        }
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    fn set_pos(&mut self, pos: Point2<f32>) {
        let point: Point2<f32> = self.rect.point().into();
        let diff = pos - point;
        for widget in &mut self.widgets {
            let pos: Point2<f32> = widget.rect().point().into();
            widget.set_pos(pos + diff);
        }
        self.rect.move_to(pos);
    }
}
