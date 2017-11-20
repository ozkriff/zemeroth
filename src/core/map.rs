use std::iter::repeat;
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Distance(pub i32);

/// Cube coordinates
/// <http://www.redblobgames.com/grids/hexagons/#coordinates-cube>
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PosCube<T: Debug + Copy = i32> {
    pub x: T,
    pub y: T,
    pub z: T,
}

/// Axial coordinates
/// <http://www.redblobgames.com/grids/hexagons/#coordinates-axial>
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PosHex<T: Debug + Copy = i32> {
    /// column
    pub q: T,

    /// row
    pub r: T,
}

// TODO: merge hex_to_cube_f and hex_to_cube
pub fn hex_to_cube_f(hex: PosHex<f32>) -> PosCube<f32> {
    PosCube {
        x: hex.q,
        y: -hex.q - hex.r,
        z: hex.r,
    }
}

pub fn hex_to_cube(hex: PosHex) -> PosCube {
    PosCube {
        x: hex.q,
        y: -hex.q - hex.r,
        z: hex.r,
    }
}

pub fn cube_to_hex<T: Debug + Copy>(cube: PosCube<T>) -> PosHex<T> {
    PosHex {
        q: cube.x,
        r: cube.z,
    }
}

pub fn hex_round(hex: PosHex<f32>) -> PosHex {
    cube_to_hex(cube_round(hex_to_cube_f(hex)))
}

/// <http://www.redblobgames.com/grids/hexagons/#rounding>
pub fn cube_round(cube: PosCube<f32>) -> PosCube {
    let mut rx = cube.x.round();
    let mut ry = cube.y.round();
    let mut rz = cube.z.round();
    let x_diff = (rx - cube.x).abs();
    let y_diff = (ry - cube.y).abs();
    let z_diff = (rz - cube.z).abs();
    if x_diff > y_diff && x_diff > z_diff {
        rx = -ry - rz;
    } else if y_diff > z_diff {
        ry = -rx - rz;
    } else {
        rz = -rx - ry;
    }
    PosCube {
        x: rx as i32,
        y: ry as i32,
        z: rz as i32,
    }
}

pub fn distance_cube(a: PosCube, b: PosCube) -> Distance {
    let n = ((a.x - b.x).abs() + (a.y - b.y).abs() + (a.z - b.z).abs()) / 2;
    Distance(n)
}

pub fn distance_hex(a: PosHex, b: PosHex) -> Distance {
    distance_cube(hex_to_cube(a), hex_to_cube(b))
}

fn is_inboard(radius: Distance, pos: PosHex) -> bool {
    let origin = PosHex { q: 0, r: 0 };
    distance_hex(origin, pos) <= radius
}

#[derive(Clone, Debug)]
pub struct HexIter {
    cursor: PosHex,
    radius: Distance,
}

impl HexIter {
    fn new(radius: Distance) -> Self {
        let mut iter = Self {
            cursor: PosHex {
                q: -radius.0,
                r: -radius.0,
            },
            radius,
        };
        iter.inc_cursor_with_hex_bounds();
        iter
    }

    fn inc_cursor(&mut self) {
        self.cursor.q += 1;
        if self.cursor.q > self.radius.0 {
            self.cursor.q = -self.radius.0;
            self.cursor.r += 1;
        }
    }

    fn inc_cursor_with_hex_bounds(&mut self) {
        self.inc_cursor();
        while !is_inboard(self.radius, self.cursor) && self.cursor.r < self.radius.0 + 1 {
            self.inc_cursor();
        }
    }
}

impl Iterator for HexIter {
    type Item = PosHex;

    fn next(&mut self) -> Option<PosHex> {
        if self.cursor.r >= self.radius.0 + 1 {
            None
        } else {
            let current = self.cursor;
            self.inc_cursor_with_hex_bounds();
            Some(current)
        }
    }
}

///
///     [-1, 0]  [0, -1]
/// [-1, 1]  [0, 0]  [1, -1]
///     [ 0, 1]  [ 1, 0]
///
#[derive(Debug, Clone)]
pub struct HexMap<T: Copy + Debug> {
    tiles: Vec<T>,
    size: Distance,
    radius: Distance,
}

impl<T: Copy + Default + Debug> HexMap<T> {
    pub fn new(radius: Distance) -> Self {
        let size = Distance(radius.0 * 2 + 1);
        let tiles_count = (size.0 * size.0) as usize;
        let tiles = repeat(Default::default()).take(tiles_count).collect();
        Self {
            tiles,
            size,
            radius,
        }
    }

    pub fn radius(&self) -> Distance {
        self.radius
    }

    pub fn iter(&self) -> HexIter {
        HexIter::new(self.radius)
    }

    pub fn is_inboard(&self, pos: PosHex) -> bool {
        is_inboard(self.radius, pos)
    }

    fn hex_to_index(&self, hex: PosHex) -> usize {
        let i = (hex.r + self.radius.0) + (hex.q + self.radius.0) * self.size.0;
        i as usize
    }

    pub fn tile(&self, pos: PosHex) -> T {
        assert!(self.is_inboard(pos));
        self.tiles[self.hex_to_index(pos)]
    }

    pub fn set_tile(&mut self, pos: PosHex, tile: T) {
        assert!(self.is_inboard(pos));
        let index = self.hex_to_index(pos);
        self.tiles[index] = tile;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    SouthEast,
    East,
    NorthEast,
    NorthWest,
    West,
    SouthWest,
}

/// <http://www.redblobgames.com/grids/hexagons/#neighbors-axial>
const DIR_TO_POS_DIFF: [[i32; 2]; 6] = [[1, 0], [1, -1], [0, -1], [-1, 0], [-1, 1], [0, 1]];

impl Dir {
    pub fn from_int(n: i32) -> Self {
        assert!(n >= 0 && n < 6);
        let dirs = [
            Dir::SouthEast,
            Dir::East,
            Dir::NorthEast,
            Dir::NorthWest,
            Dir::West,
            Dir::SouthWest,
        ];
        dirs[n as usize]
    }

    pub fn to_int(&self) -> i32 {
        match *self {
            Dir::SouthEast => 0,
            Dir::East => 1,
            Dir::NorthEast => 2,
            Dir::NorthWest => 3,
            Dir::West => 4,
            Dir::SouthWest => 5,
        }
    }

    // TODO:
    /*
    fn hex_direction(direction: Dir) {
        return axial_directions[direction]
    }
    */

    pub fn get_dir_from_to(from: PosHex, to: PosHex) -> Dir {
        assert_eq!(distance_hex(from, to), Distance(1));
        let diff = [to.q - from.q, to.r - from.r];
        for dir in dirs() {
            if diff == DIR_TO_POS_DIFF[dir.to_int() as usize] {
                return dir;
            }
        }
        panic!("impossible positions: {:?}, {:?}", from, to); // TODO: implement Display for PosHex
    }

    pub fn get_neighbor_pos(pos: PosHex, dir: Dir) -> PosHex {
        let diff = DIR_TO_POS_DIFF[dir.to_int() as usize];
        PosHex {
            q: pos.q + diff[0],
            r: pos.r + diff[1],
        }
    }
}

#[derive(Clone, Debug)]
pub struct DirIter {
    index: i32,
}

pub fn dirs() -> DirIter {
    DirIter { index: 0 }
}

impl Iterator for DirIter {
    type Item = Dir;

    fn next(&mut self) -> Option<Dir> {
        let max = DIR_TO_POS_DIFF.len() as i32;
        let next_dir = if self.index >= max {
            None
        } else {
            Some(Dir::from_int(self.index))
        };
        self.index += 1;
        next_dir
    }
}
