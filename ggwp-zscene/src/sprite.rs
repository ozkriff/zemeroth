use std::{cell::RefCell, fmt, path::Path, rc::Rc};

use ggez::{
    graphics::{self, Drawable, Rect},
    nalgebra::{Point2, Vector2},
    Context, GameResult,
};

struct SpriteData {
    drawable: Box<dyn Drawable>,
    dimensions: Rect,
    basic_scale: f32,
    param: graphics::DrawParam,
    offset: Vector2<f32>,
}

impl fmt::Debug for SpriteData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SpriteData")
            .field("drawable", &format_args!("{:p}", self.drawable))
            .field("dimensions", &self.dimensions)
            .field("basic_scale", &self.basic_scale)
            .field("param", &self.param)
            .field("offset", &self.offset)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct Sprite {
    data: Rc<RefCell<SpriteData>>,
}

impl Sprite {
    pub fn from_drawable(context: &mut Context, drawable: Box<dyn Drawable>, height: f32) -> Self {
        let dimensions = drawable
            .dimensions(context)
            .expect("Can't get the dimensions"); // TODO: convert to Result
        let scale = height / dimensions.h;
        let param = graphics::DrawParam {
            scale: Vector2::new(scale, scale),
            ..Default::default()
        };
        let data = SpriteData {
            drawable,
            dimensions,
            basic_scale: scale,
            param,
            offset: Vector2::new(0.0, 0.0),
        };
        let data = Rc::new(RefCell::new(data));
        Self { data }
    }

    pub fn from_image(context: &mut Context, image: graphics::Image, height: f32) -> Self {
        Self::from_drawable(context, Box::new(image), height)
    }

    pub fn from_path<P: AsRef<Path>>(
        context: &mut Context,
        path: P,
        height: f32,
    ) -> GameResult<Self> {
        let image = graphics::Image::new(context, path)?;
        Ok(Self::from_image(context, image, height))
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
        let mut dimensions = data.dimensions;
        dimensions.scale(data.param.scale.x, data.param.scale.y);
        data.offset.x = -dimensions.w * offset.x;
        data.offset.y = -dimensions.h * offset.y;
        let offset = data.offset;
        data.param.dest += offset - old_offset;
    }

    pub fn draw(&self, context: &mut Context) -> GameResult<()> {
        let data = self.data.borrow();
        data.drawable.draw(context, data.param)
    }

    pub fn pos(&self) -> Point2<f32> {
        let data = self.data.borrow();
        data.param.dest - data.offset
    }

    pub fn rect(&self) -> Rect {
        // TODO: `self.dimensions` + `graphics::transform_rect(param)` ?
        let pos = self.pos();
        let data = self.data.borrow();
        let r = data.dimensions;
        // TODO: angle?
        Rect {
            x: pos.x,
            y: pos.y,
            w: r.w * data.param.scale.x,
            h: r.h * data.param.scale.y,
        }
    }

    pub fn color(&self) -> graphics::Color {
        self.data.borrow().param.color
    }

    pub fn scale(&self) -> f32 {
        let data = self.data.borrow();
        data.param.scale.y / data.basic_scale
    }

    pub fn set_pos(&mut self, pos: Point2<f32>) {
        let mut data = self.data.borrow_mut();
        data.param.dest = pos + data.offset;
    }

    pub fn set_color(&mut self, color: graphics::Color) {
        self.data.borrow_mut().param.color = color;
    }

    pub fn set_scale(&mut self, scale: f32) {
        let mut data = self.data.borrow_mut();
        let s = data.basic_scale * scale;
        let scale = Vector2::new(s, s);
        data.param.scale = scale;
    }

    // TODO: unittest this?
    pub fn is_same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.data, &other.data)
    }
}
