use std::fmt::Debug;

use cgmath::{Matrix4, Vector2};

#[derive(Clone, Copy, Debug)]
pub struct Size<T: Copy + Debug = i32> {
    pub w: T,
    pub h: T,
}

impl Size<f32> {
    pub fn is_pos_inside(&self, pos: Point) -> bool {
        let w_2 = self.w / 2.0;
        let h_2 = self.h / 2.0;
        let x = pos.0.x;
        let y = pos.0.y;
        x >= -w_2 && x <= w_2 && y >= -h_2 && y <= h_2
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Point(pub Vector2<f32>);

pub fn pos_to_matrix(pos: Point) -> Matrix4<f32> {
    Matrix4::from_translation(pos.0.extend(0.0))
}
