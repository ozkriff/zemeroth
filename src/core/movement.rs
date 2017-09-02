use std::collections::VecDeque;
use std::slice::Windows;
use core::map::{dirs, Dir, Distance, HexMap, PosHex};
use core::{State, TileType, Unit};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MovePoints(pub i32);

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    cost: MovePoints,
    parent_dir: Option<Dir>,
}

impl Tile {
    pub fn parent(&self) -> Option<Dir> {
        self.parent_dir
    }

    pub fn cost(&self) -> MovePoints {
        self.cost
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            cost: MovePoints(0),
            parent_dir: None,
        }
    }
}

// TODO: const (see https://github.com/rust-lang/rust/issues/24111 )
pub fn max_cost() -> MovePoints {
    MovePoints(i32::max_value())
}

pub fn tile_cost(state: &State, _: &Unit, _: PosHex, pos: PosHex) -> MovePoints {
    match state.map().tile(pos) {
        TileType::Floor => MovePoints(1),
        TileType::Lava => MovePoints(3),
    }
}

#[derive(Clone, Debug)]
pub struct Path {
    tiles: Vec<PosHex>,
}

impl Path {
    pub fn new(tiles: Vec<PosHex>) -> Self {
        Self { tiles }
    }

    pub fn tiles(&self) -> &[PosHex] {
        &self.tiles
    }

    pub fn truncate(&self, state: &State, unit: &Unit) -> Option<Path> {
        let mut new_path = Vec::new();
        let mut cost = MovePoints(0);
        new_path.push(self.tiles[0]);
        let move_points = unit.unit_type.move_points;
        for Step { from, to } in self.steps() {
            cost.0 += tile_cost(state, unit, from, to).0;
            if cost > move_points {
                break;
            }
            new_path.push(to);
        }
        if new_path.len() >= 2 {
            Some(Path::new(new_path))
        } else {
            None
        }
    }

    pub fn cost_for(&self, state: &State, unit: &Unit) -> MovePoints {
        let mut cost = MovePoints(0);
        for step in self.steps() {
            cost.0 += tile_cost(state, unit, step.from, step.to).0;
        }
        cost
    }

    pub fn steps(&self) -> Steps {
        Steps {
            windows: self.tiles.windows(2),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Step {
    pub from: PosHex,
    pub to: PosHex,
}

#[derive(Clone, Debug)]
pub struct Steps<'a> {
    windows: Windows<'a, PosHex>,
}

impl<'a> Iterator for Steps<'a> {
    type Item = Step;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(window) = self.windows.next() {
            let from = window[0];
            let to = window[1];
            Some(Step { from, to })
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct Pathfinder {
    queue: VecDeque<PosHex>,
    map: HexMap<Tile>,
}

impl Pathfinder {
    pub fn new(map_radius: Distance) -> Pathfinder {
        Pathfinder {
            queue: VecDeque::new(),
            map: HexMap::new(map_radius),
        }
    }

    pub fn map(&self) -> &HexMap<Tile> {
        &self.map
    }

    fn process_neighbor_pos(
        &mut self,
        state: &State,
        unit: &Unit,
        original_pos: PosHex,
        neighbor_pos: PosHex,
    ) {
        let old_cost = self.map.tile(original_pos).cost;
        let tile_cost = tile_cost(state, unit, original_pos, neighbor_pos);
        let new_cost = MovePoints(old_cost.0 + tile_cost.0);
        let tile = self.map.tile(neighbor_pos);
        if tile.cost > new_cost {
            let parent_dir = Dir::get_dir_from_to(neighbor_pos, original_pos);
            let updated_tile = Tile {
                cost: new_cost,
                parent_dir: Some(parent_dir),
            };
            self.map.set_tile(neighbor_pos, updated_tile);
            self.queue.push_back(neighbor_pos);
        }
    }

    fn clean_map(&mut self) {
        for pos in self.map.iter() {
            let tile = Tile {
                cost: max_cost(),
                parent_dir: None,
            };
            self.map.set_tile(pos, tile);
        }
    }

    fn try_to_push_neighbors(&mut self, state: &State, unit: &Unit, pos: PosHex) {
        assert!(self.map.is_inboard(pos));
        for dir in dirs() {
            let neighbor_pos = Dir::get_neighbor_pos(pos, dir);
            if !state.units_at(neighbor_pos).is_empty() {
                continue;
            }
            if self.map.is_inboard(neighbor_pos) {
                self.process_neighbor_pos(state, unit, pos, neighbor_pos);
            }
        }
    }

    fn push_start_pos_to_queue(&mut self, start_pos: PosHex) {
        let start_tile = Tile::default();
        self.map.set_tile(start_pos, start_tile);
        self.queue.push_back(start_pos);
    }

    pub fn fill_map(&mut self, state: &State, unit: &Unit) {
        assert!(self.queue.is_empty());
        self.clean_map();
        self.push_start_pos_to_queue(unit.pos);
        while let Some(pos) = self.queue.pop_front() {
            self.try_to_push_neighbors(state, unit, pos);
        }
    }

    pub fn path(&self, destination: PosHex) -> Option<Path> {
        if self.map.tile(destination).cost == max_cost() {
            return None;
        }
        let mut path = vec![destination];
        let mut pos = destination;
        while self.map.tile(pos).cost != MovePoints(0) {
            assert!(self.map.is_inboard(pos));
            let parent_dir = match self.map.tile(pos).parent() {
                Some(dir) => dir,
                None => return None,
            };
            pos = Dir::get_neighbor_pos(pos, parent_dir);
            path.push(pos);
        }
        path.reverse();
        if path.is_empty() {
            None
        } else {
            Some(Path::new(path))
        }
    }
}
