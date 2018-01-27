use core::{ObjId, Parts, PlayerId, Prototypes, TileType};
use core::{component, map};

#[derive(Clone, Debug)]
pub struct State {
    parts: Parts,
    map: map::HexMap<TileType>,
    player_id: PlayerId,
    players_count: i32,
    prototypes: Prototypes,
}

impl State {
    pub fn new(prototypes: Prototypes) -> Self {
        let radius = map::Distance(5); // TODO: pass `Options` struct
        Self {
            map: map::HexMap::new(radius),
            player_id: PlayerId(0),
            players_count: 2, // TODO: Read from the `Options` struct
            parts: Parts::new(),
            prototypes,
        }
    }

    pub fn player_id(&self) -> PlayerId {
        self.player_id
    }

    pub fn next_player_id(&self) -> PlayerId {
        let current_player_id = PlayerId(self.player_id().0 + 1);
        if current_player_id.0 < self.players_count {
            current_player_id
        } else {
            PlayerId(0)
        }
    }

    pub fn parts(&self) -> &Parts {
        &self.parts
    }

    pub fn map(&self) -> &map::HexMap<TileType> {
        &self.map
    }

    pub(super) fn prototype_for(&self, name: &str) -> Vec<component::Component> {
        let prototypes = &self.prototypes.0;
        prototypes[name].clone()
    }
}

/// Mutators. Be carefull with them!
impl State {
    pub(super) fn parts_mut(&mut self) -> &mut Parts {
        &mut self.parts
    }

    pub(super) fn map_mut(&mut self) -> &mut map::HexMap<TileType> {
        &mut self.map
    }

    pub(super) fn set_player_id(&mut self, new_value: PlayerId) {
        self.player_id = new_value;
    }

    pub(super) fn alloc_id(&mut self) -> ObjId {
        self.parts.alloc_id()
    }
}
