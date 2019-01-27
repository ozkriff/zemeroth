#![warn(bare_trait_objects)]

/// Tiny and opinionated GUI

use std::{
    fmt,
    cell::RefCell,
    fmt::Debug,
    rc::Rc,
    sync::mpsc::{channel, Receiver, Sender},
};

use ggez::{
    graphics::{self, Color, Image, Text, DrawParam, Drawable, Rect},
    nalgebra::Point2,
    Context, GameResult,
};
use log::{debug, info};

// TODO: What should we do if some widget changes its size?

pub fn pack<W: Widget + 'static>(widget: W) -> RcWidget {
    Rc::new(RefCell::new(widget))
}

// TODO: make a note in the blog that I wasn't able to use a trait
// because ggez::Drawable is not object safe (because of the `Into` arg)

// TODO: rustfmt

// // TODO: try to find a better name
// #[derive(Debug, Clone)]
// pub enum DrawableSized {
//     Image(Image),
//     Text(Text, (f32, f32)),
// }

// impl DrawableSized {
//     pub fn from_image(image: Image) -> Self {
//         DrawableSized::Image(image)
//     }

//     pub fn from_text(text: Text, context: &mut Context) -> Self {
//         let (w, h) = text.dimensions(context);
//         DrawableSized::Text(text, (w as _, h as _))
//     }

//     pub fn dimensions(&self) -> (f32, f32) {
//         match self {
//             DrawableSized::Image(image) => {
//                 let d = image.dimensions();
//                 (d.w, d.h)
//             },
//             DrawableSized::Text(_, dimensions) => *dimensions,
//         }
//     }

//     pub fn draw(&self, context: &mut Context, param: DrawParam) -> GameResult {
//         match self {
//             DrawableSized::Image(image) => image.draw(context, param),
//             DrawableSized::Text(text, _) => text.draw(context, param),
//         }
//     }
// }

// #[derive(Debug, Clone)]
struct Sprite {
    // image: graphics::Image,
    // image: DrawableSized, // TODO: rename Image to ?

    // TODO: try to require `Debug` also! (using helper trait I guess)
    drawable: Box<dyn Drawable>,
    dimensions: Rect,

    basic_scale: f32, // TODO: Do I really need it now? check

    // param: graphics::DrawParam,
    scale: f32,
    color: Color,
    dest: Point2<f32>
}

impl Debug for Sprite {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // write!(f, "Point {{ x: {}, y: {} }}", self.x, self+.y)
        unimplemented!() // TODO: show some fields
    }
}

impl Sprite {
    // TODO: Document what `height` argument is.
    fn new(context: &mut Context, drawable: Box<dyn Drawable>, height: f32) -> Self {
        // TODO: remove expect, return Result from this function.
        let dimensions = drawable.dimensions(context).expect("Can't get dimensions");
        let basic_scale = height / dimensions.h;
        Self {
            drawable,
            dimensions,
            scale: basic_scale,
            color: graphics::BLACK,
            dest: [0.0, 0.0].into(),
            basic_scale,
        }
    }

    // TODO: name me
    fn xxx(&self, drawable: Box<dyn Drawable>) -> Self {
        Self {
            drawable,
            dimensions: self.dimensions,
            scale: self.scale,
            color: self.color,
            dest: self.dest,
            basic_scale: self.basic_scale,
        }
    }

    // fn from_image(image: Image, height: f32) -> Self {
    //     let basic_scale = height / image.height() as f32;
    //     Self {
    //         drawable: DrawableSized::from_image(image),
    //         scale: basic_scale,
    //         color: graphics::BLACK,
    //         dest: [0.0, 0.0].into(),
    //         basic_scale,
    //     }
    // }

    // fn from_text(text: Text, context: &mut Context, height: f32) -> Self {
    //     let image = DrawableSized::from_text(text, context);
    //     let (_, image_height) = image.dimensions();
    //     let basic_scale = height / image_height; // TODO: ?
    //     Self {
    //         image,
    //         scale: basic_scale,
    //         color: graphics::BLACK,
    //         dest: [0.0, 0.0].into(),
    //         basic_scale,
    //     }
    // }

    // TODO: Add some method to change or switch the drawable (TODO: github issue?).

    fn draw(&self, context: &mut Context) -> GameResult<()> {
        let param = graphics::DrawParam::new()
            .scale([self.scale, self.scale])
            .color(self.color)
            .dest(self.dest);
        // graphics::draw(context, &self.image, param)
        self.drawable.draw(context, param)
    }

    fn rect(&self) -> Rect {
        // let r = self.image.dimensions();
        // let dimensions = self.dimensions;
        let w = self.dimensions.w;
        let h = self.dimensions.h;
        // TODO: Transform Drawable 's dimensions
        Rect {
            x: self.dest.x,
            y: self.dest.y,
            // w: r.w * self.scale,
            // h: r.h * self.scale,
            w: w * self.scale,
            h: h * self.scale,
        }
    }

    fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    fn set_pos(&mut self, pos: Point2<f32>) {
        self.dest = pos;
    }
}

fn make_bg(context: &mut Context, sprite: &Sprite) -> Sprite {
    // TODO: clean this up
    // let width = sprite.image.width();
    // let height = sprite.image.height();
    let h = sprite.dimensions.h;
    let w = sprite.dimensions.w;
    let count = w as usize * h as usize * 4; // TODO: Are this conversions make sense?
    let data: Vec<u8> = [255, 255, 255, 255]
        .iter()
        .cloned()
        .cycle()
        .take(count)
        .collect();
    let image = Image::from_rgba8(context, w as _, h as _, &data)
        .expect("zgui: Can't create bg image");
    // let mut bg = Sprite {
    //     image: DrawableSized::Image(image),
    //     ..sprite.clone()
    // };
    let mut bg = sprite.xxx(Box::new(image));
    // let mut bg: Sprite = unimplemented!(); // TODO: !!!
    // let mut bg = Sprite::new(context, Box::new(image), sprite.);
    // bg.scale = sprite.scale;
    // bg.dest = sprite.dest;
    bg.set_color([0.8, 0.8, 0.8, 0.5].into());
    bg
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
    bg: Sprite,
}

impl Label {
    pub fn new(context: &mut Context, drawable: Box<dyn Drawable>, height: f32) -> Self {
        let sprite = Sprite::new(context, drawable, height);
        let bg = make_bg(context, &sprite);
        Self { sprite, bg }
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
pub struct Button<Message: Clone> {
    sprite: Sprite,
    bg: Sprite,
    sender: Sender<Message>,
    message: Message,
}

impl<Message: Clone> Button<Message> {
    pub fn new(
        context: &mut Context,
        // image: Image, // TODO: !
        drawable: Box<dyn Drawable>,
        height: f32,
        sender: Sender<Message>,
        message: Message,
    ) -> Self {
        let sprite = Sprite::new(context, drawable, height);
        let bg = make_bg(context, &sprite);
        Self {
            sprite,
            bg,
            sender,
            message,
        }
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

    fn click(&self, pos: Point2<f32>) {
        for widget in &self.widgets {
            widget.click(pos);
        }
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    fn set_pos(&mut self, pos: Point2<f32>) {
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

    fn click(&self, pos: Point2<f32>) {
        for widget in &self.widgets {
            widget.click(pos);
        }
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    fn set_pos(&mut self, pos: Point2<f32>) {
        let diff = pos - self.rect.point();
        for widget in &mut self.widgets {
            let pos = widget.rect().point();
            widget.set_pos(pos + diff);
        }
        self.rect.move_to(pos);
    }
}
