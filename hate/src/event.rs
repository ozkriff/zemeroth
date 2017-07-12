use geom::Point;

#[derive(Clone, Debug)]
pub enum Event {
    Click { pos: Point },
    Resize { aspect_ratio: f32 },
}
