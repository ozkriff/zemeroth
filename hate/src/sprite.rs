use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;
use cgmath::{Matrix4, Vector2, Zero};
use ::Context;
use mesh::RMesh;
use geom::{self, Size, Point};

#[derive(Debug, Clone)]
pub struct Sprite {
    data: Rc<RefCell<SpriteData>>,
}

#[derive(Debug, Clone)]
struct SpriteData {
    mesh: RMesh,
    pos: Point,
    color: [f32; 4],
}

impl Sprite {
    pub fn from_path<P: AsRef<Path>>(context: &mut Context, path: P, size: f32) -> Self {
        let size = Size { w: size, h: size };
        Self::from_path_rect(context, path, size)
    }

    pub fn from_path_rect<P: AsRef<Path>>(context: &mut Context, path: P, size: Size<f32>) -> Self {
        let texture = context.load_texture(path);
        let mesh = RMesh::new(context, texture, size);
        Self::from_mesh(mesh)
    }

    pub fn from_mesh(mesh: RMesh) -> Self {
        let pos = Point(Vector2::zero());
        let color = [1.0, 1.0, 1.0, 1.0];
        let data = SpriteData { mesh, pos, color };
        Self {
            data: Rc::new(RefCell::new(data)),
        }
    }

    pub fn is_same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.data, &other.data)
    }

    pub fn pos(&self) -> Point {
        self.data.borrow().pos
    }

    pub fn set_pos(&mut self, pos: Point) {
        self.data.borrow_mut().pos = pos;
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.data.borrow_mut().color = color;
    }

    pub fn color(&self) -> [f32; 4] {
        self.data.borrow().color
    }

    pub fn size(&self) -> Size<f32> {
        self.data.borrow().mesh.size()
    }

    pub fn draw(&self, context: &mut Context, parent_matrix: Matrix4<f32>) {
        let data = self.data.borrow();
        let model_matrix = geom::pos_to_matrix(data.pos);
        context.set_color(data.color);
        context.draw_mesh(parent_matrix * model_matrix, data.mesh.mesh());
    }
}
