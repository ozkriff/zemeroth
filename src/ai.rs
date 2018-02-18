use core::command::{self, Command};
use core::{self, check, ObjId, PlayerId, State};
use core::movement::{self, Path, Pathfinder};
use core::state;
use core::map;
use core::utils::shuffle_vec;

#[derive(Debug, Clone)]
pub struct Ai {
    id: PlayerId,

    /// Each AI has its own Pathfinder because it's not a part of the game state.
    pathfinder: Pathfinder,
}

impl Ai {
    pub fn new(id: PlayerId, map_radius: map::Distance) -> Self {
        Self {
            id,
            pathfinder: Pathfinder::new(map_radius),
        }
    }

    fn get_best_path(&mut self, state: &State, unit_id: ObjId) -> Option<Path> {
        self.pathfinder.fill_map(state, unit_id);
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
                let cost = path.cost_for(state, unit_id);
                if best_cost > cost {
                    best_cost = cost;
                    best_path = Some(path);
                }
            }
        }
        best_path
    }

    fn try_throw_bomb(&self, state: &State, unit_id: ObjId) -> Option<Command> {
        // TODO: find ability in the parts and use it here:
        let ability = core::ability::Ability::Bomb(core::ability::Bomb(map::Distance(3)));
        for &target_id in &shuffle_vec(state::enemy_agent_ids(state, self.id)) {
            let target_pos = state.parts().pos.get(target_id).0;
            for dir in shuffle_vec(map::dirs().collect()) {
                let pos = map::Dir::get_neighbor_pos(target_pos, dir);
                if !state.map().is_inboard(pos) || state::is_tile_blocked(state, pos) {
                    continue;
                }
                let command = Command::UseAbility(command::UseAbility {
                    id: unit_id,
                    pos,
                    ability,
                });
                if check(state, &command).is_ok() {
                    return Some(command);
                }
            }
        }
        None
    }

    fn try_summon_imp(&self, state: &State, unit_id: ObjId) -> Option<Command> {
        // TODO: find ability in the parts and use it here:
        let ability = core::ability::Ability::Summon(core::ability::Summon(3));
        let target_pos = state.parts().pos.get(unit_id).0;
        let command = Command::UseAbility(command::UseAbility {
            id: unit_id,
            pos: target_pos,
            ability,
        });
        if check(state, &command).is_ok() {
            return Some(command);
        }
        None
    }

    fn try_to_attack(&self, state: &State, unit_id: ObjId) -> Option<Command> {
        for &target_id in &shuffle_vec(state::enemy_agent_ids(state, self.id)) {
            let command = Command::Attack(command::Attack {
                attacker_id: unit_id,
                target_id: target_id,
            });
            if check(state, &command).is_ok() {
                return Some(command);
            }
        }
        None
    }

    fn try_to_move(&mut self, state: &State, unit_id: ObjId) -> Option<Command> {
        let path = match self.get_best_path(state, unit_id) {
            Some(path) => path,
            None => return None,
        };
        let path = match path.truncate(state, unit_id) {
            Some(path) => path,
            None => return None,
        };
        let cost = path.cost_for(state, unit_id);
        let agent = state.parts().agent.get(unit_id);
        if agent.move_points < cost {
            return None;
        }
        let command = Command::MoveTo(command::MoveTo { id: unit_id, path });
        if check(state, &command).is_ok() {
            return Some(command);
        }
        None
    }

    pub fn command(&mut self, state: &State) -> Option<Command> {
        for unit_id in state::players_agent_ids(state, self.id) {
            if let Some(summon_command) = self.try_summon_imp(state, unit_id) {
                return Some(summon_command);
            }
            if let Some(bomb_command) = self.try_throw_bomb(state, unit_id) {
                return Some(bomb_command);
            }
            if let Some(attack_command) = self.try_to_attack(state, unit_id) {
                return Some(attack_command);
            }
            if let Some(move_command) = self.try_to_move(state, unit_id) {
                return Some(move_command);
            }
        }
        Some(Command::EndTurn(command::EndTurn))
    }
}
