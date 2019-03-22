use std::collections::HashMap;

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::core::{
    battle::{
        component::ObjType,
        state::{self, State},
        PlayerId, TileType,
    },
    map::{self, PosHex},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectsGroup {
    pub owner: Option<PlayerId>,
    pub typename: ObjType,
    pub line: Line,
    pub count: i32,
}

// TODO: rename to just `Object`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExactObject {
    pub owner: Option<PlayerId>,
    pub typename: ObjType,
    pub pos: PosHex,
}

// TODO: Split into `Scenario` (exact info) and `ScenarioTemplate`?
//  Rename  `exact_*` fields to just `*`.
#[serde(default = "default")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub map_radius: map::Distance,
    pub players_count: i32,

    // TODO: rename it to `randomized_tiles` later (not only `TileType::Rocks`)
    pub rocky_tiles_count: i32,

    pub exact_tiles: HashMap<PosHex, TileType>,

    // TODO: rename to `randomized_objects`
    pub objects: Vec<ObjectsGroup>,

    pub exact_objects: Vec<ExactObject>,
}

#[derive(Clone, Debug, derive_more::From)]
pub enum Error {
    MapIsTooSmall,
    PosOutsideOfMap(PosHex),
    NoPlayerAgents,
    NoEnemyAgents,
    UnsupportedPlayersCount(i32),
}

impl Scenario {
    pub fn check(&self) -> Result<(), Error> {
        if self.players_count != 2 {
            return Err(Error::UnsupportedPlayersCount(self.players_count));
        }
        if self.map_radius.0 < 3 {
            return Err(Error::MapIsTooSmall);
        }
        let origin = PosHex { q: 0, r: 0 };
        for obj in &self.exact_objects {
            let dist = map::distance_hex(origin, obj.pos);
            if dist > self.map_radius {
                return Err(Error::PosOutsideOfMap(obj.pos));
            }
        }
        let any_exact_player_agents = self
            .exact_objects
            .iter()
            .any(|obj| obj.owner == Some(PlayerId(0)));
        let any_random_player_agents = self
            .objects
            .iter()
            .any(|obj| obj.owner == Some(PlayerId(0)));
        if !any_exact_player_agents && !any_random_player_agents {
            return Err(Error::NoPlayerAgents);
        }
        let any_exact_enemy_agents = self
            .exact_objects
            .iter()
            .any(|obj| obj.owner == Some(PlayerId(1)));
        let any_random_enemy_agents = self
            .objects
            .iter()
            .any(|obj| obj.owner == Some(PlayerId(1)));
        if !any_exact_enemy_agents && !any_random_enemy_agents {
            return Err(Error::NoEnemyAgents);
        }
        Ok(())
    }
}

pub fn random_free_pos(state: &State) -> Option<PosHex> {
    assert!(!state.deterministic_mode());
    let attempts = 30;
    let radius = state.map().radius();
    for _ in 0..attempts {
        let pos = PosHex {
            q: thread_rng().gen_range(-radius.0, radius.0),
            r: thread_rng().gen_range(-radius.0, radius.0),
        };
        if state::is_tile_plain_and_completely_free(state, pos) {
            return Some(pos);
        }
    }
    None
}

fn middle_range(min: i32, max: i32) -> (i32, i32) {
    assert!(min <= max);
    let size = max - min;
    let half = size / 2;
    let forth = size / 4;
    let min = half - forth;
    let mut max = half + forth;
    if min == max {
        max += 1;
    }
    (min, max)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Line {
    Any,
    Front,
    Middle,
    Back,
}

impl Line {
    pub fn to_range(self, radius: map::Distance) -> (i32, i32) {
        let radius = radius.0;
        match self {
            Line::Front => (radius / 2, radius + 1),
            Line::Middle => middle_range(0, radius),
            Line::Back => (0, radius / 2),
            Line::Any => (0, radius + 1),
        }
    }
}

fn random_free_sector_pos(state: &State, player_id: PlayerId, line: Line) -> Option<PosHex> {
    assert!(!state.deterministic_mode());
    let attempts = 30;
    let radius = state.map().radius();
    let (min, max) = line.to_range(radius);
    for _ in 0..attempts {
        let q = radius.0 - thread_rng().gen_range(min, max);
        let pos = PosHex {
            q: match player_id.0 {
                0 => -q,
                1 => q,
                _ => unimplemented!(),
            },
            r: thread_rng().gen_range(-radius.0, radius.0 + 1),
        };
        let no_enemies_around = !state::check_enemies_around(state, pos, player_id);
        if state::is_tile_completely_free(state, pos) && no_enemies_around {
            return Some(pos);
        }
    }
    None
}

pub fn random_pos(state: &State, owner: Option<PlayerId>, line: Line) -> Option<PosHex> {
    match owner {
        Some(player_id) => random_free_sector_pos(state, player_id, line),
        None => random_free_pos(state),
    }
}

pub fn default() -> Scenario {
    Scenario {
        map_radius: map::Distance(5),
        players_count: 2,
        rocky_tiles_count: 0,
        exact_tiles: HashMap::new(),
        objects: Vec::new(),
        exact_objects: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::middle_range;

    #[test]
    fn test_middle_range() {
        assert_eq!(middle_range(0, 3), (1, 2));
        assert_eq!(middle_range(0, 4), (1, 3));
        assert_eq!(middle_range(0, 5), (1, 3));
        assert_eq!(middle_range(0, 6), (2, 4));
        assert_eq!(middle_range(0, 7), (2, 4));
        assert_eq!(middle_range(0, 8), (2, 6));
        assert_eq!(middle_range(0, 9), (2, 6));
        assert_eq!(middle_range(0, 10), (3, 7));
    }
}
