use core::map::{Distance, HexMap, PosHex};
use core::strategy_map::{component::Parts, Id, TileType};

#[derive(Clone, Debug)]
pub struct State {
    map: HexMap<TileType>,
    parts: Parts,
}

impl State {
    pub fn new() -> Self {
        let radius = Distance(3); // TODO: pass `Options` struct
        let mut map = HexMap::new(radius);
        map.set_tile(PosHex { q: 0, r: 0 }, TileType::Water);
        let parts = Parts::new();
        Self { map, parts }
    }

    pub fn map(&self) -> &HexMap<TileType> {
        &self.map
    }

    pub fn parts(&self) -> &Parts {
        &self.parts
    }
}

pub fn agent_id_at_opt(state: &State, pos: PosHex) -> Option<Id> {
    let ids = agent_ids_at(state, pos);
    if ids.len() == 1 {
        Some(ids[0])
    } else {
        None
    }
}

pub fn agent_ids_at(state: &State, pos: PosHex) -> Vec<Id> {
    let i = state.parts().agent.ids();
    i.filter(|&id| state.parts().pos.get(id).0 == pos).collect()
}
