#![warn(bare_trait_objects)]

/// Tiny and opinionated GUI
///

#[macro_use]
extern crate log;

extern crate ggez;

use std::cell::RefCell;
use std::rc::Rc;

use ggez::graphics::{self, Image, Point2, Rect};
use ggez::{Context, GameResult};
use std::fmt::Debug;
use std::sync::mpsc::{channel, Receiver, Sender};

// TODO: What should we do if some widget changes its size?

pub fn pack<W: Widget + 'static>(widget: W) -> RcWidget {
    Rc::new(RefCell::new(widget))
}

#[derive(Debug, Clone)]
struct Sprite {
    image: graphics::Image,
    basic_scale: f32,
    param: graphics::DrawParam,
}

impl Sprite {
    pub fn from_image(image: graphics::Image, height: f32) -> Self {
        let basic_scale = height / image.height() as f32;
        let param = graphics::DrawParam {
            scale: Point2::new(basic_scale, basic_scale),
            color: Some([0.0, 0.0, 0.0, 1.0].into()),
            ..Default::default()
        };
        Self {
            image,
            param,
            basic_scale,
        }
    }

    // TODO: some method to change the image.

    pub fn draw(&self, context: &mut Context) -> GameResult<()> {
        graphics::draw_ex(context, &self.image, self.param)
    }

    pub fn rect(&self) -> Rect {
        let r = self.image.get_dimensions();
        Rect {
            x: self.param.dest.x,
            y: self.param.dest.y,
            w: r.w * self.param.scale.x,
            h: r.h * self.param.scale.y,
        }
    }

    pub fn set_pos(&mut self, pos: Point2) {
        self.param.dest = pos;
    }
}

pub fn window_to_screen(context: &Context, pos: Point2) -> Point2 {
    let (w, h) = graphics::get_drawable_size(context);
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
    fn click(&self, _: Point2) {}
    fn rect(&self) -> Rect;
    fn set_pos(&mut self, pos: Point2);
}

pub type RcWidget = Rc<RefCell<dyn Widget>>;

#[derive(Debug)]
pub struct AnchoredWidget {
    widget: Rc<RefCell<dyn Widget>>,
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
        let (w, h) = graphics::get_drawable_size(context);
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

    pub fn add(&mut self, widget: &Rc<RefCell<dyn Widget>>, anchor: Anchor) {
        let widget = widget.clone();
        let anchored_widget = AnchoredWidget { widget, anchor };
        self.anchored_widgets.push(anchored_widget);
        let ratio = self.aspect_ratio;
        self.resize(ratio);
    }

    pub fn remove(&mut self, widget: &Rc<RefCell<dyn Widget>>) -> GameResult<()> {
        let len_before = self.anchored_widgets.len();
        self.anchored_widgets
            .retain(|w| !Rc::ptr_eq(&w.widget, widget));
        let len_after = self.anchored_widgets.len();
        info!("len_before={}, len_after={}", len_before, len_after);
        if len_after != len_before - 1 {
            Err("Can't remove the widget".to_string())?
        }
        Ok(())
    }

    pub fn draw(&self, context: &mut Context) -> GameResult<()> {
        let old_coordinates = graphics::get_screen_coordinates(context);
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
            widget.set_pos(pos);
        }
    }
}

#[derive(Debug)]
pub struct Label {
    sprite: Sprite,
}

impl Label {
    pub fn new(image: Image, height: f32) -> Self {
        let sprite = Sprite::from_image(image, height);
        Self { sprite }
    }
}

impl Widget for Label {
    fn draw(&self, context: &mut Context) -> GameResult<()> {
        self.sprite.draw(context)
    }

    fn rect(&self) -> Rect {
        self.sprite.rect()
    }

    fn set_pos(&mut self, pos: Point2) {
        self.sprite.set_pos(pos);
    }
}

// TODO: add a semi-transparent background
#[derive(Debug)]
pub struct Button<Message: Clone> {
    sprite: Sprite,
    sender: Sender<Message>,
    message: Message,
}

impl<Message: Clone> Button<Message> {
    pub fn new(image: Image, height: f32, sender: Sender<Message>, message: Message) -> Self {
        let sprite = Sprite::from_image(image, height);
        Self {
            sprite,
            sender,
            message,
        }
    }
}

impl<Message: Clone + Debug> Widget for Button<Message> {
    fn draw(&self, context: &mut Context) -> GameResult<()> {
        self.sprite.draw(context)
    }

    fn click(&self, pos: Point2) {
        debug!("Label: rect={:?}, pos={:?}", self.sprite.rect(), pos);
        if self.sprite.rect().contains(pos) {
            let message = self.message.clone();
            self.sender.send(message).unwrap();
            return;
        }
    }

    fn rect(&self) -> Rect {
        self.sprite.rect()
    }

    fn set_pos(&mut self, pos: Point2) {
        self.sprite.set_pos(pos);
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
            widget.set_pos(pos);
        } else {
            widget.set_pos(self.rect.point());
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

    fn click(&self, pos: Point2) {
        for widget in &self.widgets {
            widget.click(pos);
        }
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    fn set_pos(&mut self, pos: Point2) {
        let diff = pos - self.rect.point();
        for widget in &mut self.widgets {
            let pos = widget.rect().point();
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
            let mut pos = rect.point();
            pos.x += rect.w;
            widget.set_pos(pos);
        } else {
            widget.set_pos(self.rect.point());
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

    fn click(&self, pos: Point2) {
        for widget in &self.widgets {
            widget.click(pos);
        }
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    fn set_pos(&mut self, pos: Point2) {
        let diff = pos - self.rect.point();
        for widget in &mut self.widgets {
            let pos = widget.rect().point();
            widget.set_pos(pos + diff);
        }
        self.rect.move_to(pos);
    }
}
