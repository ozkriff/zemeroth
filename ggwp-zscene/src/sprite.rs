use std::{cell::RefCell, collections::HashMap, fmt, path::Path, rc::Rc, hash::Hash};

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
    drawable: Option<Box<dyn Drawable>>,
    drawables: HashMap<String, Option<Box<dyn Drawable>>>,
    current_frame_name: String,
    dimensions: Rect,
    basic_scale: f32,
    param: graphics::DrawParam,
    offset: Vector2<f32>,
    facing: Facing,
}

impl fmt::Debug for SpriteData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SpriteData")
            .field("drawable", &self.drawable.as_ref().map(|d| d as *const _))
            .field("drawables", &format_args!("{:?}", self.drawables.keys()))
            .field("current_frame_name", &self.current_frame_name)
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
        let mut drawables = HashMap::new();
        drawables.insert("".into(), None);
        let data = SpriteData {
            drawable: Some(drawable),
            drawables,
            current_frame_name: "".into(),
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

    pub fn add_frame(&mut self, frame_name: String, drawable: Box<dyn Drawable>) {
        let mut data = self.data.borrow_mut();
        data.drawables.insert(frame_name, Some(drawable));
    }

    pub fn from_paths<S: Eq + Hash + ::std::borrow::Borrow<str>, P: AsRef<Path>>(
        context: &mut Context,
        paths: &HashMap<S, P>,
        height: f32,
    ) -> Result<Self> {
        let path = paths.get(&"").expect("missing default path");
        let mut this = Self::from_path(context, path.as_ref(), height)?;
        for (frame_name, frame_path) in paths.into_iter() {
            let image = graphics::Image::new(context, frame_path)?;
            this.add_frame(frame_name.borrow().to_string(), Box::new(image));
        }
        Ok(this)
    }

    pub fn has_frame(&self, frame_name: &str) -> bool {
        let data = self.data.borrow();
        data.drawables.contains_key(frame_name)
    }

    // TODO: Add a usage example
    pub fn set_frame(&mut self, frame_name: &str) {
        assert!(self.has_frame(frame_name));
        let mut data = self.data.borrow_mut();
        let previous_frame_name = data.current_frame_name.clone();
        let previous_drawable = data.drawable.take().expect("no active drawable");
        let previous_slot = data
            .drawables
            .get_mut(&previous_frame_name)
            .expect("bad frame name");
        *previous_slot = Some(previous_drawable);
        data.drawable = data
            .drawables
            .get_mut(frame_name)
            .expect("bad frame name")
            .take();
        assert!(data.drawable.is_some());
        data.current_frame_name = frame_name.into();
    }

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
        let drawable = data.drawable.as_ref().expect("no active drawable");
        drawable.draw(context, data.param)
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
