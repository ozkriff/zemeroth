use std::default::Default;
use core::map::{HexMap, PosHex};
use core::movement::MovePoints;

pub use core::execute::execute;
pub use core::check::check;

pub mod command;
pub mod event;
pub mod movement;
pub mod effect;
pub mod map;
pub mod execute;
pub mod component;

mod check;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlayerId(pub i32); // TODO: make field private

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ObjId(i32);

impl Default for ObjId {
    fn default() -> Self {
        ObjId(0)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Strength(pub i32);

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Attacks(pub i32);

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Moves(pub i32);

/// Move or Attack
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Jokers(pub i32);

#[derive(Clone, Copy, Debug)]
pub enum TileType {
    Floor,
    Lava,
}

impl Default for TileType {
    fn default() -> Self {
        TileType::Floor
    }
}

rancor_storage!(Parts<ObjId>: {
    strength: component::Strength,
    pos: component::Pos,
    meta: component::Meta,
    belongs_to: component::BelongsTo,
    agent: component::Agent,
    blocker: component::Blocker,
});

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prototypes(pub HashMap<String, Vec<component::Component>>);

#[derive(Clone, Debug)]
pub struct State {
    parts: Parts,
    map: HexMap<TileType>,
    player_id: PlayerId,
    players_count: i32,
    prototypes: Prototypes,
}

impl State {
    pub fn new(prototypes: Prototypes) -> Self {
        let radius = map::Distance(5); // TODO: pass `Options` struct
        let mut map = HexMap::new(radius);
        {
            // TODO: load\generate maps
            map.set_tile(PosHex { q: 0, r: 0 }, TileType::Lava);
        }
        Self {
            map,
            player_id: PlayerId(0),
            players_count: 2, // TODO: Read from the `Options` struct
            parts: Parts::new(),
            prototypes,
        }
    }

    pub fn player_id(&self) -> PlayerId {
        self.player_id
    }

    pub fn parts(&self) -> &Parts {
        &self.parts
    }

    pub fn map(&self) -> &HexMap<TileType> {
        &self.map
    }
}

pub fn belongs_to(state: &State, player_id: PlayerId, id: ObjId) -> bool {
    state.parts.belongs_to.get(id).0 == player_id
}

pub fn object_ids_at(state: &State, pos: PosHex) -> Vec<ObjId> {
    let ids = state.parts().agent.ids();
    ids.filter(|&id| state.parts.pos.get(id).0 == pos).collect()
}

pub fn players_agent_ids(state: &State, player_id: PlayerId) -> Vec<ObjId> {
    let ids = state.parts().agent.ids();
    ids.filter(|&id| belongs_to(state, player_id, id)).collect()
}

pub fn enemy_agent_ids(state: &State, player_id: PlayerId) -> Vec<ObjId> {
    let ids = state.parts().agent.ids();
    ids.filter(|&id| !belongs_to(state, player_id, id))
        .collect()
}

pub fn is_tile_blocked(state: &State, pos: PosHex) -> bool {
    for id in state.parts.blocker.ids() {
        if state.parts.pos.get(id).0 == pos {
            return true;
        }
    }
    false
}
