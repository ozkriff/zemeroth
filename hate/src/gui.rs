use std::collections::HashMap;

use cgmath::Vector2;

use context::Context;
use geom::{Point, Size};
use mesh::RMesh;
use sprite::Sprite;
use texture;

// TODO: Make private? Move to other file?
pub fn text_sprite(context: &mut Context, label: &str, height: f32) -> Sprite {
    let mesh = text_mesh(context, label, height);
    let mut sprite = Sprite::from_mesh(mesh);
    sprite.set_color([0.0, 0.0, 0.0, 1.0]);
    sprite
}

fn text_mesh(context: &mut Context, label: &str, height: f32) -> RMesh {
    let texture = context.text_texture(label);
    let size = Size {
        w: height / (texture.size.h as f32 / texture.size.w as f32),
        h: height,
    };
    RMesh::new(context, texture, size)
}

/// Widget ID
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(i32);

#[derive(Clone, Debug)]
struct Clickable<Message: Clone> {
    message: Message,
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

#[derive(Clone, Copy, Debug)]
pub struct Anchor {
    pub vertical: VAnchor,
    pub horizontal: HAnchor,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    // Left,
    Right,
    Up,
    // Down,
}

#[derive(Clone, Debug)]
struct Layout {
    children: Vec<Id>,
    anchor: Anchor,
    direction: Direction,
}

const SPACING: f32 = 0.02;

#[derive(Clone, Debug)]
pub struct Gui<Message: Clone> {
    aspect_ratio: f32,
    next_id: Id, // TODO: store a list of freed ids
    message_queue: Vec<Message>,
    sprites: HashMap<Id, Sprite>,
    backgrounds: HashMap<Id, Sprite>,
    clickables: HashMap<Id, Clickable<Message>>,
    layouts: HashMap<Id, Layout>,
    bg_texture: texture::Texture,
}

impl<Message: Clone> Gui<Message> {
    pub fn new(context: &mut Context) -> Self {
        let bg_texture = {
            let data = include_bytes!("test_button_bg.png");
            texture::load(context, data)
        };
        Self {
            aspect_ratio: context.aspect_ratio(),
            next_id: Id(0),
            message_queue: Vec::new(),
            clickables: HashMap::new(),
            layouts: HashMap::new(),
            sprites: HashMap::new(),
            backgrounds: HashMap::new(),
            bg_texture,
        }
    }

    pub fn try_recv(&mut self) -> Option<Message> {
        if self.message_queue.is_empty() {
            None
        } else {
            Some(self.message_queue.remove(0))
        }
    }

    fn alloc_id(&mut self) -> Id {
        let id = self.next_id;
        self.next_id.0 += 1;
        id
    }

    fn calc_size(&self, id: Id) -> Size<f32> {
        if let Some(layout) = self.layouts.get(&id) {
            let mut size = Size { w: 0.0, h: 0.0 };
            for &child_id in &layout.children {
                let child_size = self.calc_size(child_id);
                match layout.direction {
                    Direction::Right => {
                        if size.h < child_size.h {
                            size.h = child_size.h;
                        }
                        size.w += child_size.w + SPACING;
                    }
                    Direction::Up => {
                        if size.w < child_size.w {
                            size.w = child_size.w;
                        }
                        size.h += child_size.h + SPACING;
                    }
                }
            }
            size.w += SPACING;
            size.h += SPACING;
            size
        } else if let Some(sprite) = self.sprites.get(&id) {
            sprite.size()
        } else {
            unreachable!(); // TODO:
        }
    }

    pub fn resize(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
        self.update_sprite_positions();
    }

    fn update_sprite_positions(&mut self) {
        let aspect_ratio = self.aspect_ratio;
        for (&layout_id, layout) in &self.layouts {
            let size = self.calc_size(layout_id);
            let y = match layout.direction {
                Direction::Right => match layout.anchor.vertical {
                    VAnchor::Top => 1.0 - size.h / 2.0,
                    VAnchor::Middle => 0.0,
                    VAnchor::Bottom => -1.0,
                },
                Direction::Up => match layout.anchor.vertical {
                    VAnchor::Top => 1.0 - size.h / 2.0,
                    VAnchor::Middle => -size.h / 2.0,
                    VAnchor::Bottom => -1.0,
                },
            };
            let x = match layout.direction {
                Direction::Right => match layout.anchor.horizontal {
                    HAnchor::Left => -aspect_ratio,
                    HAnchor::Middle => -size.w / 2.0,
                    HAnchor::Right => aspect_ratio - size.w,
                },
                Direction::Up => match layout.anchor.horizontal {
                    HAnchor::Left => size.w / 2.0 - aspect_ratio,
                    HAnchor::Middle => 0.0,
                    HAnchor::Right => aspect_ratio - size.w / 2.0,
                },
            };
            let mut cursor = Point(Vector2 { x, y });
            for id in &layout.children {
                // TODO: this code is not ready for nested containers
                //
                // TODO: And, by the way, how can I understand what layouts are roots?
                // Add a `root` component? :-\
                //
                let sprite = self.sprites.get_mut(id).expect("Can't access children");
                match layout.direction {
                    Direction::Right => {
                        cursor.0.x += sprite.size().w / 2.0;
                        sprite.set_pos(cursor);
                        if let Some(bg) = self.backgrounds.get_mut(id) {
                            bg.set_pos(cursor);
                        }
                        cursor.0.x += sprite.size().w / 2.0 + SPACING;
                    }
                    Direction::Up => {
                        cursor.0.y += sprite.size().h / 2.0;
                        sprite.set_pos(cursor);
                        if let Some(bg) = self.backgrounds.get_mut(id) {
                            bg.set_pos(cursor);
                        }
                        cursor.0.y += sprite.size().h / 2.0 + SPACING;
                    }
                }
            }
        }
    }

    pub fn draw(&self, context: &mut Context) {
        let projection_matrix = context.projection_matrix();
        for bg in self.backgrounds.values() {
            bg.draw(context, projection_matrix);
        }
        for sprite in self.sprites.values() {
            sprite.draw(context, projection_matrix);
        }
    }

    pub fn click(&mut self, pos: Point) {
        for (id, clickable) in &self.clickables {
            let sprite = self.sprites.get(id).expect("Clickable depends on Sprite");
            let size = sprite.size();
            if size.is_pos_inside(Point(pos.0 - sprite.pos().0)) {
                self.message_queue.push(clickable.message.clone());
            }
        }
    }

    pub fn add_button(&mut self, context: &mut Context, sprite: Sprite, message: Message) -> Id {
        let id = self.alloc_id();
        self.sprites.insert(id, sprite);
        self.update_bg(context, id);
        self.clickables.insert(id, Clickable { message });
        id
    }

    pub fn add_sprite(&mut self, sprite: Sprite) -> Id {
        let id = self.alloc_id();
        self.sprites.insert(id, sprite);
        id
    }

    // TODO: is it a good idea?
    pub fn update_sprite(&mut self, context: &mut Context, id: Id, sprite: Sprite) {
        self.sprites.insert(id, sprite);
        self.update_bg(context, id);
        self.update_sprite_positions();
    }

    fn update_bg(&mut self, context: &mut Context, id: Id) {
        let sprite = self.sprites.get(&id).expect("Can't add bg without Sprite");
        let mut sprite_size = sprite.size();
        sprite_size.w += SPACING / 2.0;
        let mesh = RMesh::new(context, self.bg_texture.clone(), sprite_size);
        let mut bg = Sprite::from_mesh(mesh);
        bg.set_color([0.0, 0.0, 0.0, 0.5]);
        self.backgrounds.insert(id, bg);
    }

    pub fn remove(&mut self, id: Id) -> Result<(), ()> {
        let mut other_things_to_remove = Vec::new();
        self.sprites.remove(&id);
        self.backgrounds.remove(&id);
        self.clickables.remove(&id);
        if let Some(layout) = self.layouts.get(&id) {
            other_things_to_remove.extend(layout.children.clone());
        }
        self.layouts.remove(&id);
        for layout in self.layouts.values_mut() {
            layout.children.retain(|&e| e != id);
        }
        self.update_sprite_positions();
        for id in other_things_to_remove {
            self.remove(id).unwrap();
        }
        Ok(())
    }

    pub fn add_layout(&mut self, anchor: Anchor, direction: Direction, children: Vec<Id>) -> Id {
        let id = self.alloc_id();
        let layout = Layout {
            anchor,
            direction,
            children,
        };
        self.layouts.insert(id, layout);
        self.update_sprite_positions();
        id
    }
}
