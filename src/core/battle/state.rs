use crate::core::{
    battle::{
        self,
        ability::{self, Ability, PassiveAbility},
        component::ObjType,
        effect, Id, PlayerId, Strength, TileType,
    },
    map::{self, PosHex},
    utils,
};

pub use self::{
    apply::apply,
    private::{BattleResult, State},
};

mod apply;
mod private;

pub fn is_agent_belong_to(state: &State, player_id: PlayerId, id: Id) -> bool {
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

pub fn is_tile_completely_free(state: &State, pos: PosHex) -> bool {
    if !state.map().is_inboard(pos) {
        return false;
    }
    for id in state.parts().pos.ids() {
        if state.parts().pos.get(id).0 == pos {
            return false;
        }
    }
    true
}

pub fn is_lasting_effect_over(state: &State, id: Id, timed_effect: &effect::Timed) -> bool {
    if let effect::Lasting::Poison = timed_effect.effect {
        let strength = state.parts().strength.get(id).strength;
        if strength <= Strength(1) {
            return true;
        }
    }
    timed_effect.duration.is_over()
}

/// Are there any enemy agents on the adjacent tiles?
pub fn check_enemies_around(state: &State, pos: PosHex, player_id: PlayerId) -> bool {
    for dir in map::dirs() {
        let neighbor_pos = map::Dir::get_neighbor_pos(pos, dir);
        if let Some(id) = agent_id_at_opt(state, neighbor_pos) {
            let neighbor_player_id = state.parts().belongs_to.get(id).0;
            if neighbor_player_id != player_id {
                return true;
            }
        }
    }
    false
}

pub fn ids_at(state: &State, pos: PosHex) -> Vec<Id> {
    let i = state.parts().pos.ids();
    i.filter(|&id| state.parts().pos.get(id).0 == pos).collect()
}

pub fn obj_with_passive_ability_at(
    state: &State,
    pos: PosHex,
    ability: PassiveAbility,
) -> Option<Id> {
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

pub fn blocker_id_at(state: &State, pos: PosHex) -> Id {
    blocker_id_at_opt(state, pos).unwrap()
}

pub fn blocker_id_at_opt(state: &State, pos: PosHex) -> Option<Id> {
    let ids = blocker_ids_at(state, pos);
    if ids.len() == 1 {
        Some(ids[0])
    } else {
        None
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

pub fn blocker_ids_at(state: &State, pos: PosHex) -> Vec<Id> {
    let i = state.parts().blocker.ids();
    i.filter(|&id| state.parts().pos.get(id).0 == pos).collect()
}

pub fn players_agent_ids(state: &State, player_id: PlayerId) -> Vec<Id> {
    let i = state.parts().agent.ids();
    i.filter(|&id| is_agent_belong_to(state, player_id, id))
        .collect()
}

pub fn enemy_agent_ids(state: &State, player_id: PlayerId) -> Vec<Id> {
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
            if positions.len() == count as usize {
                break;
            }
        }
    }
    positions
}

pub fn sort_agent_ids_by_distance_to_enemies(state: &State, ids: &mut [Id]) {
    ids.sort_unstable_by_key(|&id| {
        let agent_player_id = state.parts().belongs_to.get(id).0;
        let agent_pos = state.parts().pos.get(id).0;
        let mut min_distance = state.map().height();
        for enemy_id in enemy_agent_ids(state, agent_player_id) {
            let enemy_pos = state.parts().pos.get(enemy_id).0;
            let distance = map::distance_hex(agent_pos, enemy_pos);
            if distance < min_distance {
                min_distance = distance;
            }
        }
        min_distance
    });
}

pub fn get_armor(state: &State, id: Id) -> Strength {
    let parts = state.parts();
    let default = Strength(0);
    parts.armor.get_opt(id).map(|v| v.armor).unwrap_or(default)
}

pub fn players_agent_types(state: &State, player_id: PlayerId) -> Vec<ObjType> {
    players_agent_ids(state, player_id)
        .into_iter()
        .map(|id| state.parts().meta.get(id).name.clone())
        .collect()
}

pub fn can_agent_use_ability(state: &State, id: Id, ability: &Ability) -> bool {
    let parts = state.parts();
    let agent_player_id = parts.belongs_to.get(id).0;
    let agent = parts.agent.get(id);
    let has_actions = agent.attacks > battle::Attacks(0) || agent.jokers > battle::Jokers(0);
    let is_player_agent = agent_player_id == state.player_id();
    let abilities = &parts.abilities.get(id).0;
    let r_ability = abilities.iter().find(|r| &r.ability == ability).unwrap();
    let is_ready = r_ability.status == ability::Status::Ready;
    is_player_agent && is_ready && has_actions
}
