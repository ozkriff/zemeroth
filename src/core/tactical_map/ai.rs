use log::info;

use crate::core::{
    map::{self, Distance, HexMap},
    tactical_map::{
        ability::{self, Ability},
        check,
        command::{self, Command},
        movement::{self, Path, Pathfinder},
        state,
        utils::shuffle_vec,
        ObjId, PlayerId, State,
    },
};

fn does_agent_have_ability_summon(state: &State, id: ObjId) -> bool {
    if let Some(abilities) = state.parts().abilities.get_opt(id) {
        for ability in &abilities.0 {
            if let Ability::Summon = ability.ability {
                return true;
            }
        }
    }
    false
}

fn does_agent_have_ability_bomb(state: &State, id: ObjId) -> bool {
    if let Some(abilities) = state.parts().abilities.get_opt(id) {
        for ability in &abilities.0 {
            if let Ability::BombDemonic(_) = ability.ability {
                return true;
            }
        }
    }
    false
}

fn check_path_is_ok(state: &State, id: ObjId, path: &Path) -> bool {
    let path = path.clone();
    let command = command::MoveTo { id, path }.into();
    check(state, &command).is_ok()
}

#[derive(Clone, Debug)]
enum PathfindingResult {
    Path(Path),
    CantFindPath,
    DontNeedToMove,
}

#[derive(Clone, Copy, Debug)]
struct DistanceRange {
    min: Distance,
    max: Distance,
}

#[derive(Debug, Clone)]
pub struct Ai {
    id: PlayerId,

    distance_map: HexMap<bool>,

    /// Each AI has its own Pathfinder because it's not a part of the game state.
    pathfinder: Pathfinder,
}

impl Ai {
    pub fn new(id: PlayerId, map_radius: Distance) -> Self {
        Self {
            id,
            pathfinder: Pathfinder::new(map_radius),
            distance_map: HexMap::new(map_radius),
        }
    }

    /// Finds shortest path to some enemy.
    fn find_path_to_nearest_enemy(&mut self, state: &State, agent_id: ObjId) -> Option<Path> {
        self.pathfinder.fill_map(state, agent_id);
        let mut best_path = None;
        let mut best_cost = movement::max_cost();
        for &target_id in &shuffle_vec(state::enemy_agent_ids(state, self.id)) {
            let target_pos = state.parts().pos.get(target_id).0;
            for dir in map::dirs() {
                let pos = map::Dir::get_neighbor_pos(target_pos, dir);
                if !state.map().is_inboard(pos) {
                    continue;
                }
                let path = match self.pathfinder.path(pos) {
                    Some(path) => path,
                    None => continue,
                };
                let cost = path.cost_for(state, agent_id);
                if best_cost > cost {
                    best_cost = cost;
                    best_path = Some(path);
                }
            }
        }
        best_path
    }

    fn find_path_to_preserve_distance(
        &mut self,
        state: &State,
        agent_id: ObjId,
        distance_range: DistanceRange,
    ) -> Option<Path> {
        // clean the map
        for pos in self.distance_map.iter() {
            self.distance_map.set_tile(pos, false);
        }

        for pos in self.distance_map.iter() {
            for &enemy_id in &state::enemy_agent_ids(state, self.id) {
                let enemy_pos = state.parts().pos.get(enemy_id).0;
                if map::distance_hex(pos, enemy_pos) <= distance_range.max {
                    self.distance_map.set_tile(pos, true);
                }
            }
            for &enemy_id in &state::enemy_agent_ids(state, self.id) {
                let enemy_pos = state.parts().pos.get(enemy_id).0;
                if map::distance_hex(pos, enemy_pos) <= distance_range.min {
                    self.distance_map.set_tile(pos, false);
                }
            }
        }

        self.pathfinder.fill_map(state, agent_id);
        // TODO: remove code duplication
        let mut best_path = None;
        let mut best_cost = movement::max_cost();
        for pos in self.distance_map.iter() {
            if !self.distance_map.tile(pos) {
                continue;
            }
            let path = match self.pathfinder.path(pos) {
                Some(path) => path,
                None => continue,
            };
            let cost = path.cost_for(state, agent_id);
            if best_cost > cost {
                best_cost = cost;
                best_path = Some(path);
            }
        }
        best_path
    }

    fn find_any_path(&mut self, state: &State, agent_id: ObjId) -> Option<Path> {
        self.pathfinder.fill_map(state, agent_id);
        let mut best_path = None;
        let mut best_distance = state.map().radius();
        for pos in self.distance_map.iter() {
            let path = match self.pathfinder.path(pos) {
                Some(path) => path,
                None => continue,
            };
            for &enemy_id in &state::enemy_agent_ids(state, self.id) {
                let enemy_pos = state.parts().pos.get(enemy_id).0;
                let distance = map::distance_hex(pos, enemy_pos);
                // TODO: compare path costs
                if distance <= best_distance {
                    best_path = Some(path.clone());
                    best_distance = distance;
                }
            }
        }
        best_path
    }

    fn try_throw_bomb(&self, state: &State, agent_id: ObjId) -> Option<Command> {
        // TODO: find ability in the parts and use it here:
        let ability = Ability::BombDemonic(ability::BombDemonic(Distance(3)));
        for &target_id in &shuffle_vec(state::enemy_agent_ids(state, self.id)) {
            let target_pos = state.parts().pos.get(target_id).0;
            for dir in shuffle_vec(map::dirs().collect()) {
                let pos = map::Dir::get_neighbor_pos(target_pos, dir);
                if !state.map().is_inboard(pos) || state::is_tile_blocked(state, pos) {
                    continue;
                }
                let command = Command::UseAbility(command::UseAbility {
                    id: agent_id,
                    pos,
                    ability: ability.clone(),
                });
                if check(state, &command).is_ok() {
                    return Some(command);
                }
            }
        }
        None
    }

    fn try_summon_imp(&self, state: &State, agent_id: ObjId) -> Option<Command> {
        // TODO: find ability in the parts and use it here:
        let ability = ability::Ability::Summon;
        let target_pos = state.parts().pos.get(agent_id).0;
        let command = Command::UseAbility(command::UseAbility {
            id: agent_id,
            pos: target_pos,
            ability,
        });
        if check(state, &command).is_ok() {
            return Some(command);
        }
        None
    }

    fn try_to_attack(&self, state: &State, agent_id: ObjId) -> Option<Command> {
        for &target_id in &shuffle_vec(state::enemy_agent_ids(state, self.id)) {
            let attacker_id = agent_id;
            let command = command::Attack {
                attacker_id,
                target_id,
            }
            .into();
            if check(state, &command).is_ok() {
                return Some(command);
            }
        }
        None
    }

    fn try_to_move_closer(&mut self, state: &State, id: ObjId) -> PathfindingResult {
        let path = match self.find_path_to_nearest_enemy(state, id) {
            Some(path) => path,
            None => return PathfindingResult::CantFindPath,
        };
        if path.tiles().len() == 1 {
            return PathfindingResult::DontNeedToMove;
        }
        let path = match path.truncate(state, id) {
            Some(path) => path,
            None => return PathfindingResult::CantFindPath,
        };
        let cost = path.cost_for(state, id);
        let agent = state.parts().agent.get(id);
        if agent.move_points < cost {
            return PathfindingResult::CantFindPath;
        }
        if check_path_is_ok(state, id, &path) {
            return PathfindingResult::Path(path);
        }
        PathfindingResult::CantFindPath
    }

    fn try_to_keep_distance(
        &mut self,
        state: &State,
        agent_id: ObjId,
        distance_range: DistanceRange,
    ) -> PathfindingResult {
        let path = match self.find_path_to_preserve_distance(state, agent_id, distance_range) {
            Some(path) => path,
            None => return PathfindingResult::CantFindPath,
        };
        if path.tiles().len() == 1 {
            return PathfindingResult::DontNeedToMove;
        }
        let path = match path.truncate(state, agent_id) {
            Some(path) => path,
            None => return PathfindingResult::CantFindPath,
        };
        let cost = path.cost_for(state, agent_id);
        let agent = state.parts().agent.get(agent_id);
        if agent.move_points < cost {
            return PathfindingResult::CantFindPath;
        }
        if check_path_is_ok(state, agent_id, &path) {
            return PathfindingResult::Path(path);
        }
        PathfindingResult::CantFindPath
    }

    fn try_to_find_bad_path(&mut self, state: &State, agent_id: ObjId) -> Option<Command> {
        let path = match self.find_any_path(state, agent_id) {
            Some(path) => path,
            None => return None,
        };
        let path = match path.truncate(state, agent_id) {
            Some(path) => path,
            None => return None,
        };
        let cost = path.cost_for(state, agent_id);
        let agent = state.parts().agent.get(agent_id);
        if agent.move_points < cost {
            return None;
        }
        let command = command::MoveTo { id: agent_id, path }.into();
        if check(state, &command).is_ok() {
            return Some(command);
        }
        None
    }

    fn try_to_move(&mut self, state: &State, agent_id: ObjId) -> Option<Command> {
        let path_result = if does_agent_have_ability_summon(state, agent_id) {
            let range = DistanceRange {
                min: Distance(4),
                max: Distance(6),
            };
            self.try_to_keep_distance(state, agent_id, range)
        } else if does_agent_have_ability_bomb(state, agent_id) {
            let range = DistanceRange {
                min: Distance(2),
                max: Distance(4),
            };
            self.try_to_keep_distance(state, agent_id, range)
        } else {
            self.try_to_move_closer(state, agent_id)
        };
        match path_result {
            PathfindingResult::Path(path) => {
                let command = command::MoveTo { id: agent_id, path }.into();
                if check(state, &command).is_ok() {
                    Some(command)
                } else {
                    None
                }
            }
            PathfindingResult::CantFindPath => self.try_to_find_bad_path(state, agent_id),
            PathfindingResult::DontNeedToMove => None,
        }
    }

    pub fn command(&mut self, state: &State) -> Option<Command> {
        if state.battle_result().is_some() {
            info!("AI: The battle has ended, can't create new commands.");
            return None;
        }
        let mut ids = state::players_agent_ids(state, self.id);
        state::sort_agent_ids_by_distance_to_enemies(state, &mut ids);
        for agent_id in ids {
            if let Some(summon_command) = self.try_summon_imp(state, agent_id) {
                return Some(summon_command);
            }
            if let Some(bomb_command) = self.try_throw_bomb(state, agent_id) {
                return Some(bomb_command);
            }
            if let Some(attack_command) = self.try_to_attack(state, agent_id) {
                return Some(attack_command);
            }
            if let Some(move_command) = self.try_to_move(state, agent_id) {
                return Some(move_command);
            }
        }
        Some(Command::EndTurn(command::EndTurn))
    }
}
