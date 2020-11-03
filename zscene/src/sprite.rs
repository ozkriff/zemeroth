use std::{cell::RefCell, collections::HashMap, rc::Rc};

use mq::{
    prelude::{Color, Rect, Vec2},
    text::{self, Font},
    texture::{self, DrawTextureParams, Texture2D},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Facing {
    Left,
    Right,
}

#[derive(Clone, Debug)]
enum Drawable {
    Texture(Texture2D),
    Text {
        label: String,
        font: Font,
        font_size: u16,
    },
}

impl Drawable {
    fn dimensions(&self) -> Rect {
        match *self {
            Drawable::Texture(texture) => Rect::new(0.0, 0.0, texture.width(), texture.height()),
            Drawable::Text {
                ref label,
                font,
                font_size,
            } => {
                let (w, _) = text::measure_text(&label, Some(font), font_size, 1.0);
                // TODO: A hack to have a fixed height for text.
                // TODO: Keep this in sync with the same hack in zscene until fixed.
                let h = font_size as f32 * 1.4;
                Rect::new(-w / 1.0, -h / 1.0, w / 1.0, h / 1.0)
            }
        }
    }
}

#[derive(Debug)]
struct SpriteData {
    drawable: Option<Drawable>,
    drawables: HashMap<String, Option<Drawable>>,
    current_frame_name: String,
    dimensions: Rect,
    basic_scale: f32,
    pos: Vec2,
    scale: Vec2,
    color: Color,
    offset: Vec2,
    facing: Facing,
}

#[derive(Debug, Clone)]
pub struct Sprite {
    data: Rc<RefCell<SpriteData>>,
}

impl Sprite {
    pub fn deep_clone(&self) -> Self {
        let data = self.data.borrow();
        let cloned_data = SpriteData {
            drawable: data.drawable.clone(),
            drawables: data.drawables.clone(),
            current_frame_name: data.current_frame_name.clone(),
            dimensions: data.dimensions,
            basic_scale: data.basic_scale,
            pos: data.pos,
            scale: data.scale,
            color: data.color,
            offset: data.offset,
            facing: data.facing,
        };
        Sprite {
            data: Rc::new(RefCell::new(cloned_data)),
        }
    }

    fn from_drawable(drawable: Drawable, height: f32) -> Self {
        let dimensions = drawable.dimensions();
        let scale = height / dimensions.h;
        let mut drawables = HashMap::new();
        drawables.insert("".into(), None);
        let data = SpriteData {
            drawable: Some(drawable),
            drawables,
            current_frame_name: "".into(),
            dimensions,
            basic_scale: scale,
            scale: Vec2::new(scale, scale),
            offset: Vec2::new(0.0, 0.0),
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            pos: Vec2::new(0.0, 0.0),
            facing: Facing::Right,
        };
        let data = Rc::new(RefCell::new(data));
        Self { data }
    }

    pub fn from_texture(texture: Texture2D, height: f32) -> Self {
        Self::from_drawable(Drawable::Texture(texture), height)
    }

    pub fn from_text((label, font, font_size): (&str, Font, u16), height: f32) -> Self {
        Self::from_drawable(
            Drawable::Text {
                label: label.to_string(),
                font,
                font_size,
            },
            height,
        )
    }

    fn add_frame(&mut self, frame_name: String, drawable: Drawable) {
        let mut data = self.data.borrow_mut();
        data.drawables.insert(frame_name, Some(drawable));
    }

    pub fn from_textures(frames: &HashMap<String, Texture2D>, height: f32) -> Self {
        let tex = *frames.get("").expect("missing default path");
        let mut this = Self::from_texture(tex, height);
        for (frame_name, &tex) in frames.iter() {
            this.add_frame(frame_name.clone(), Drawable::Texture(tex));
        }
        this
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
            *data.scale.x_mut() *= -1.0;
            let mut dimensions = data.dimensions;
            dimensions.scale(data.scale.x(), data.scale.y());
            let off_x = -data.offset.x() / dimensions.w;
            let off_y = -data.offset.y() / dimensions.h;
            offset = Vec2::new(-off_x, off_y);
        }
        self.set_offset(offset);
    }

    pub fn set_centered(&mut self, is_centered: bool) {
        let offset = if is_centered {
            Vec2::new(0.5, 0.5)
        } else {
            Vec2::new(0.0, 0.0)
        };
        self.set_offset(offset);
    }

    /// [0.0 .. 1.0]
    pub fn set_offset(&mut self, offset: Vec2) {
        let mut data = self.data.borrow_mut();
        let old_offset = data.offset;
        let off_x = -data.dimensions.w * data.scale.x() * offset.x();
        let off_y = -data.dimensions.h * data.scale.y() * offset.y();
        data.offset = Vec2::new(off_x, off_y);
        data.pos = data.pos + data.offset - old_offset;
    }

    pub fn draw(&self) {
        let data = self.data.borrow();
        let drawable = data.drawable.as_ref().expect("no active drawable");
        match drawable {
            Drawable::Texture(texture) => {
                texture::draw_texture_ex(
                    *texture,
                    data.pos.x(),
                    data.pos.y(),
                    data.color,
                    DrawTextureParams {
                        dest_size: Some(data.scale * Vec2::new(texture.width(), texture.height())),
                        ..Default::default()
                    },
                );
            }
            Drawable::Text {
                label,
                font,
                font_size,
            } => {
                text::draw_text_ex(
                    label,
                    data.pos.x(),
                    data.pos.y(),
                    text::TextParams {
                        font_size: *font_size,
                        font: *font,
                        font_scale: data.scale.x(),
                        color: data.color,
                    },
                );
            }
        }
    }

    pub fn pos(&self) -> Vec2 {
        let data = self.data.borrow();
        data.pos - data.offset
    }

    pub fn rect(&self) -> Rect {
        // TODO: `self.dimensions` + `graphics::transform_rect(param)` ?
        let pos = self.pos();
        let data = self.data.borrow();
        let r = data.dimensions;
        // TODO: angle?
        Rect {
            x: pos.x(),
            y: pos.y(),
            w: r.w * data.scale.x(),
            h: r.h * data.scale.y(),
        }
    }

    pub fn color(&self) -> Color {
        self.data.borrow().color
    }

    pub fn scale(&self) -> f32 {
        let data = self.data.borrow();
        data.scale.x() / data.basic_scale
    }

    pub fn set_pos(&mut self, pos: Vec2) {
        let mut data = self.data.borrow_mut();
        data.pos = pos + data.offset;
    }

    pub fn set_color(&mut self, color: Color) {
        self.data.borrow_mut().color = color;
    }

    pub fn set_scale(&mut self, scale: f32) {
        let mut data = self.data.borrow_mut();
        let s = data.basic_scale * scale;
        data.scale = Vec2::new(s, s);
    }

    // TODO: unittest this?
    pub fn is_same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.data, &other.data)
    }
}
