use core::ability::PassiveAbility;
use core::map::{self, PosHex};
use core::utils;
use core::{ObjId, PlayerId, TileType};

pub use self::private::State;

mod private {
    use core::component::{Component, Parts, Prototypes};
    use core::map::{self, HexMap};
    use core::{ObjId, PlayerId, TileType};

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

        // TODO: make visible only for `apply`
        pub(in core) fn prototype_for(&self, name: &str) -> Vec<Component> {
            let prototypes = &self.prototypes.0;
            prototypes[name].clone()
        }
    }

    /// Mutators. Be carefull with them!
    impl State {
        // TODO: check that it's called only from apply.rs!
        pub(in core) fn parts_mut(&mut self) -> &mut Parts {
            &mut self.parts
        }

        pub(in core) fn map_mut(&mut self) -> &mut HexMap<TileType> {
            &mut self.map
        }

        pub(in core) fn set_player_id(&mut self, new_value: PlayerId) {
            self.player_id = new_value;
        }

        pub(in core) fn alloc_id(&mut self) -> ObjId {
            self.parts.alloc_id()
        }
    }
}

pub fn is_agent_belong_to(state: &State, player_id: PlayerId, id: ObjId) -> bool {
    state.parts().belongs_to.get(id).0 == player_id
}

pub fn is_tile_blocked(state: &State, pos: PosHex) -> bool {
    assert!(state.map().is_inboard(pos));
    for id in state.parts().blocker.ids() {
        if state.parts().pos.get(id).0 == pos {
            return true;
        }
    }
    false
}

pub fn is_tile_plain_and_completely_free(state: &State, pos: PosHex) -> bool {
    if !state.map().is_inboard(pos) || state.map().tile(pos) != TileType::Plain {
        return false;
    }
    for id in state.parts().pos.ids() {
        if state.parts().pos.get(id).0 == pos {
            return false;
        }
    }
    true
}

pub fn ids_at(state: &State, pos: PosHex) -> Vec<ObjId> {
    let i = state.parts().pos.ids();
    i.filter(|&id| state.parts().pos.get(id).0 == pos).collect()
}

pub fn obj_with_passive_ability_at(
    state: &State,
    pos: PosHex,
    ability: PassiveAbility,
) -> Option<ObjId> {
    for id in ids_at(state, pos) {
        if let Some(abilities) = state.parts().passive_abilities.get_opt(id) {
            for &current_ability in &abilities.0 {
                if current_ability == ability {
                    return Some(id);
                }
            }
        }
    }
    None
}

pub fn blocker_id_at(state: &State, pos: PosHex) -> ObjId {
    blocker_id_at_opt(state, pos).unwrap()
}

pub fn blocker_id_at_opt(state: &State, pos: PosHex) -> Option<ObjId> {
    let ids = blocker_ids_at(state, pos);
    if ids.len() == 1 {
        Some(ids[0])
    } else {
        None
    }
}

pub fn agent_id_at_opt(state: &State, pos: PosHex) -> Option<ObjId> {
    let ids = agent_ids_at(state, pos);
    if ids.len() == 1 {
        Some(ids[0])
    } else {
        None
    }
}

pub fn agent_ids_at(state: &State, pos: PosHex) -> Vec<ObjId> {
    let i = state.parts().agent.ids();
    i.filter(|&id| state.parts().pos.get(id).0 == pos).collect()
}

pub fn blocker_ids_at(state: &State, pos: PosHex) -> Vec<ObjId> {
    let i = state.parts().blocker.ids();
    i.filter(|&id| state.parts().pos.get(id).0 == pos).collect()
}

pub fn players_agent_ids(state: &State, player_id: PlayerId) -> Vec<ObjId> {
    let i = state.parts().agent.ids();
    i.filter(|&id| is_agent_belong_to(state, player_id, id))
        .collect()
}

pub fn enemy_agent_ids(state: &State, player_id: PlayerId) -> Vec<ObjId> {
    let i = state.parts().agent.ids();
    i.filter(|&id| !is_agent_belong_to(state, player_id, id))
        .collect()
}

pub fn free_neighbor_positions(state: &State, origin: PosHex, count: i32) -> Vec<PosHex> {
    let mut positions = Vec::new();
    for dir in utils::shuffle_vec(map::dirs().collect()) {
        let pos = map::Dir::get_neighbor_pos(origin, dir);
        if state.map().is_inboard(pos) && !is_tile_blocked(state, pos) {
            positions.push(pos);
            if positions.len() == count as _ {
                break;
            }
        }
    }
    positions
}
