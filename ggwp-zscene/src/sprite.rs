use std::{cell::RefCell, path::Path, rc::Rc};

use ggez::{
    nalgebra::{Point2, Vector2},
    graphics::{self, Rect, Color},
    Context, GameResult,
};

#[derive(Debug, Clone)]
struct SpriteData {
    image: graphics::Image,
    basic_scale: f32,
    offset: Vector2<f32>,

    scale: f32,
    dest: Point2<f32>,
    color: Color,
}

#[derive(Debug, Clone)]
pub struct Sprite {
    data: Rc<RefCell<SpriteData>>,
}

impl Sprite {
    pub fn from_image(image: graphics::Image, height: f32) -> Self {
        let scale = height / f32::from(image.height());
        let data = SpriteData {
            image,
            scale,
            dest: Point2::new(0.0, 0.0),
            color: graphics::WHITE,
            basic_scale: scale,
            offset: Vector2::new(0.0, 0.0),
        };
        let data = Rc::new(RefCell::new(data));
        Self { data }
    }

    pub fn from_path<P: AsRef<Path>>(
        context: &mut Context,
        path: P,
        height: f32,
    ) -> GameResult<Self> {
        let image = graphics::Image::new(context, path)?;
        Ok(Self::from_image(image, height))
    }

    // TODO: some method to change the image.

    pub fn set_centered(&mut self, is_centered: bool) {
        let offset = if is_centered {
            Vector2::new(0.5, 0.5)
        } else {
            Vector2::new(0.0, 0.0)
        };
        self.set_offset(offset);
    }

    /// [0.0 .. 1.0]
    pub fn set_offset(&mut self, offset: Vector2<f32>) {
        let mut data = self.data.borrow_mut();
        let old_offset = data.offset;
        let mut dimensions = data.image.dimensions();
        dimensions.scale(data.scale, data.scale);
        data.offset.x = -dimensions.w * offset.x;
        data.offset.y = -dimensions.h * offset.y;
        let offset = data.offset;
        data.dest += offset - old_offset;
    }

    pub fn draw(&self, context: &mut Context) -> GameResult<()> {
        let data = self.data.borrow();
        let param = graphics::DrawParam::new()
            .dest(data.dest)
            .color(data.color)
            .scale([data.scale, data.scale]);
        graphics::draw(context, &data.image, param)
    }

    pub fn pos(&self) -> Point2<f32> {
        let data = self.data.borrow();
        data.dest - data.offset
    }

    pub fn rect(&self) -> Rect {
        let pos = self.pos();
        let data = self.data.borrow();
        let r = data.image.dimensions();
        // TODO: angle?
        Rect {
            x: pos.x,
            y: pos.y,
            w: r.w * data.scale,
            h: r.h * data.scale,
        }
    }

    pub fn color(&self) -> graphics::Color {
        self.data.borrow().color
    }

    pub fn scale(&self) -> f32 {
        let data = self.data.borrow();
        data.scale / data.basic_scale
    }

    pub fn set_pos(&mut self, pos: Point2<f32>) {
        let mut data = self.data.borrow_mut();
        data.dest = pos + data.offset;
    }

    pub fn set_color(&mut self, color: graphics::Color) {
        self.data.borrow_mut().color = color;
    }

    pub fn set_scale(&mut self, scale: f32) {
        let mut data = self.data.borrow_mut();
        data.scale = data.basic_scale * scale;
    }

    // TODO: unittest this?
    pub fn is_same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.data, &other.data)
    }
}
