use std::{cell::RefCell, fmt, path::Path, rc::Rc};

use ggez::{
    graphics::{self, Drawable, Rect},
    nalgebra::{Point2, Vector2},
    Context, GameResult,
};

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Facing {
    Left,
    Right,
}

struct SpriteData {
    drawable: Box<dyn Drawable>,
    dimensions: Rect,
    basic_scale: f32,
    param: graphics::DrawParam,
    offset: Vector2<f32>,
    facing: Facing,
}

impl fmt::Debug for SpriteData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SpriteData")
            .field("drawable", &format_args!("{:p}", self.drawable))
            .field("dimensions", &self.dimensions)
            .field("basic_scale", &self.basic_scale)
            .field("param", &self.param)
            .field("offset", &self.offset)
            .field("facing", &self.facing)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct Sprite {
    data: Rc<RefCell<SpriteData>>,
}

impl Sprite {
    pub fn from_drawable(
        context: &mut Context,
        drawable: Box<dyn Drawable>,
        height: f32,
    ) -> Result<Self> {
        let dimensions = match drawable.dimensions(context) {
            Some(dimensions) => dimensions,
            None => return Err(Error::NoDimensions),
        };
        let scale = height / dimensions.h;
        let param = graphics::DrawParam {
            scale: [scale, scale].into(),
            ..Default::default()
        };
        let data = SpriteData {
            drawable,
            dimensions,
            basic_scale: scale,
            param,
            offset: Vector2::new(0.0, 0.0),
            facing: Facing::Right,
        };
        let data = Rc::new(RefCell::new(data));
        Ok(Self { data })
    }

    pub fn from_image(context: &mut Context, image: graphics::Image, height: f32) -> Result<Self> {
        Self::from_drawable(context, Box::new(image), height)
    }

    pub fn from_path<P: AsRef<Path>>(context: &mut Context, path: P, height: f32) -> Result<Self> {
        let image = graphics::Image::new(context, path)?;
        Self::from_image(context, image, height)
    }

    // TODO: some method to change the image.

    pub fn set_facing(&mut self, facing: Facing) {
        if facing == self.data.borrow().facing {
            return;
        }
        let offset;
        {
            let mut data = self.data.borrow_mut();
            data.facing = facing;
            data.param.scale.x *= -1.0;
            let mut dimensions = data.dimensions;
            dimensions.scale(data.param.scale.x, data.param.scale.y);
            let off_x = -data.offset.x / dimensions.w;
            let off_y = -data.offset.y / dimensions.h;
            offset = Vector2::new(-off_x, off_y);
        }
        self.set_offset(offset);
    }

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
        let mut new_dest: Point2<f32> = data.param.dest.into();
        new_dest += data.offset - old_offset;
        data.param.dest = new_dest.into();
    }

    pub fn draw(&self, context: &mut Context) -> GameResult<()> {
        let data = self.data.borrow();
        data.drawable.draw(context, data.param)
    }

    pub fn pos(&self) -> Point2<f32> {
        let data = self.data.borrow();
        let dest: Point2<f32> = data.param.dest.into();
        dest - data.offset
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
        data.param.dest = (pos + data.offset).into();
    }

    pub fn set_color(&mut self, color: graphics::Color) {
        self.data.borrow_mut().param.color = color;
    }

    pub fn set_scale(&mut self, scale: f32) {
        let mut data = self.data.borrow_mut();
        let s = data.basic_scale * scale;
        let scale = [s, s].into();
        data.param.scale = scale;
    }

    // TODO: unittest this?
    pub fn is_same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.data, &other.data)
    }
}
