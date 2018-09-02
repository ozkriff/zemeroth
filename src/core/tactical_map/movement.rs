use std::{collections::VecDeque, slice::Windows};

use core::map::{dirs, Dir, Distance, HexMap, PosHex};
use core::tactical_map::{ability::PassiveAbility, state, ObjId, State, TileType};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MovePoints(pub i32);

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    cost: MovePoints,
    parent_dir: Option<Dir>,
}

impl Tile {
    pub fn parent(self) -> Option<Dir> {
        self.parent_dir
    }

    pub fn cost(self) -> MovePoints {
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

pub fn tile_cost(state: &State, _: ObjId, _: PosHex, pos: PosHex) -> MovePoints {
    // taking other dangerous objects in the tile into account
    for id in state.parts().passive_abilities.ids() {
        if state.parts().pos.get(id).0 != pos {
            continue;
        }
        for &ability in &state.parts().passive_abilities.get(id).0 {
            match ability {
                PassiveAbility::SpikeTrap | PassiveAbility::Burn | PassiveAbility::Poison => {
                    return MovePoints(4)
                }
                _ => {}
            }
        }
    }
    // just tile's cost
    match state.map().tile(pos) {
        TileType::Plain => MovePoints(1),
        TileType::Rocks => MovePoints(3),
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

    pub fn from(&self) -> PosHex {
        self.tiles[0]
    }

    pub fn to(&self) -> PosHex {
        *self.tiles().last().unwrap()
    }

    pub fn truncate(&self, state: &State, id: ObjId) -> Option<Self> {
        let agent = state.parts().agent.get(id);
        let mut new_path = Vec::new();
        let mut cost = MovePoints(0);
        new_path.push(self.tiles[0]);
        let move_points = agent.move_points;
        for Step { from, to } in self.steps() {
            cost.0 += tile_cost(state, id, from, to).0;
            if cost > move_points {
                break;
            }
            new_path.push(to);
        }
        if new_path.len() >= 2 {
            Some(Self::new(new_path))
        } else {
            None
        }
    }

    pub fn cost_for(&self, state: &State, id: ObjId) -> MovePoints {
        let mut cost = MovePoints(0);
        for step in self.steps() {
            cost.0 += tile_cost(state, id, step.from, step.to).0;
        }
        cost
    }

    pub fn steps(&self) -> Steps {
        Steps {
            windows: self.tiles.windows(2),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
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
        if let Some([from, to]) = self.windows.next() {
            Some(Step {
                from: *from,
                to: *to,
            })
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
    pub fn new(map_radius: Distance) -> Self {
        Self {
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
        id: ObjId,
        original_pos: PosHex,
        neighbor_pos: PosHex,
    ) {
        let old_cost = self.map.tile(original_pos).cost;
        let tile_cost = tile_cost(state, id, original_pos, neighbor_pos);
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

    fn try_to_push_neighbors(&mut self, state: &State, id: ObjId, pos: PosHex) {
        assert!(self.map.is_inboard(pos));
        for dir in dirs() {
            let neighbor_pos = Dir::get_neighbor_pos(pos, dir);
            if self.map.is_inboard(neighbor_pos) && !state::is_tile_blocked(state, neighbor_pos) {
                self.process_neighbor_pos(state, id, pos, neighbor_pos);
            }
        }
    }

    fn push_start_pos_to_queue(&mut self, start_pos: PosHex) {
        let start_tile = Tile::default();
        self.map.set_tile(start_pos, start_tile);
        self.queue.push_back(start_pos);
    }

    pub fn fill_map(&mut self, state: &State, id: ObjId) {
        let agent_pos = state.parts().pos.get(id).0;
        assert!(self.queue.is_empty());
        self.clean_map();
        self.push_start_pos_to_queue(agent_pos);
        while let Some(pos) = self.queue.pop_front() {
            self.try_to_push_neighbors(state, id, pos);
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

#[cfg(test)]
mod tests {
    use core::tactical_map::{
        movement::{Path, Step},
        PosHex,
    };

    const NODE_0: PosHex = PosHex { q: 0, r: 1 };
    const NODE_1: PosHex = PosHex { q: 1, r: 0 };
    const NODE_2: PosHex = PosHex { q: 2, r: 0 };

    #[test]
    fn path_from_to() {
        let nodes = vec![NODE_0, NODE_1, NODE_2];
        let path = Path::new(nodes);
        assert_eq!(path.from(), NODE_0);
        assert_eq!(path.to(), NODE_2);
    }

    #[test]
    fn path_tiles() {
        let nodes = vec![NODE_0, NODE_1, NODE_2];
        let path = Path::new(nodes);
        let tiles = path.tiles();
        assert_eq!(tiles.len(), 3);
        assert_eq!(tiles[0], NODE_0);
        assert_eq!(tiles[1], NODE_1);
        assert_eq!(tiles[2], NODE_2);
    }

    #[test]
    fn path_steps() {
        let nodes = vec![NODE_0, NODE_1, NODE_2];
        let path = Path::new(nodes);
        let mut steps = path.steps();
        assert_eq!(
            steps.next(),
            Some(Step {
                from: NODE_0,
                to: NODE_1,
            })
        );
        assert_eq!(
            steps.next(),
            Some(Step {
                from: NODE_1,
                to: NODE_2,
            })
        );
        assert_eq!(steps.next(), None);
    }
}
