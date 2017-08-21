use core::command::{self, Command};
use core::{check, ObjId, PlayerId, State};
use core::movement::{self, path_cost, truncate_path, Pathfinder};
use core::map::{self, PosHex};

#[derive(Debug, Clone)]
pub struct Ai {
    id: PlayerId,
    pathfinder: Pathfinder,
}

impl Ai {
    // TODO: i32 -> ?
    pub fn new(id: PlayerId, map_radius: i32) -> Self {
        Self {
            id,
            pathfinder: Pathfinder::new(map_radius),
        }
    }

    fn get_best_path(&mut self, state: &State, unit_id: ObjId) -> Option<Vec<PosHex>> {
        let unit = state.unit(unit_id);
        self.pathfinder.fill_map(state, unit);
        let mut best_path = None;
        let mut best_cost = movement::max_cost();
        for target_id in state.obj_iter() {
            let target = state.unit(target_id);
            if target.player_id == self.id {
                continue;
            }
            for dir in map::dirs() {
                let pos = map::Dir::get_neighbor_pos(target.pos, dir);
                if !state.map().is_inboard(pos) {
                    continue;
                }
                let path = match self.pathfinder.path(pos) {
                    Some(path) => path,
                    None => continue,
                };
                let cost = path_cost(state, state.unit(unit_id), &path);
                if best_cost > cost {
                    best_cost = cost;
                    best_path = Some(path);
                }
            }
        }
        best_path
    }

    pub fn command(&mut self, state: &State) -> Option<Command> {
        for unit_id in state.obj_iter() {
            if state.unit(unit_id).player_id != self.id {
                continue;
            }

            // move to `try_to_attack` method
            {
                for target_id in state.obj_iter() {
                    let target = state.unit(target_id);
                    if target.player_id == self.id {
                        continue;
                    }
                    let command = command::Command::Attack(command::Attack {
                        attacker_id: unit_id,
                        target_id: target_id,
                    });
                    if check(state, &command).is_ok() {
                        return Some(command);
                    }
                }
            }

            // move to `try_to_move` method
            {
                let path = match self.get_best_path(state, unit_id) {
                    Some(path) => path,
                    None => continue,
                };
                let unit = state.unit(unit_id);
                let path = match truncate_path(state, &path, unit) {
                    Some(path) => path,
                    None => continue,
                };
                let cost = path_cost(state, unit, &path);
                if unit.unit_type.move_points < cost {
                    continue;
                }
                let command = command::Command::MoveTo(command::MoveTo { id: unit_id, path });
                if check(state, &command).is_ok() {
                    return Some(command);
                }
            }
        }
        Some(Command::EndTurn(command::EndTurn))
    }
}
