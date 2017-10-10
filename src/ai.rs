use core::command::{self, Command};
use core::{belongs_to, check, ObjId, PlayerId, State};
use core::movement::{self, Path, Pathfinder};
use core::map;

#[derive(Debug, Clone)]
pub struct Ai {
    id: PlayerId,
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
        let ids = state.parts().agent.ids();
        for target_id in ids.filter(|&id| !belongs_to(state, self.id, id)) {
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

    pub fn command(&mut self, state: &State) -> Option<Command> {
        for unit_id in state.parts().agent.ids() {
            let agent = state.parts().agent.get(unit_id);
            let unit_player_id = state.parts().belongs_to.get(unit_id).0;
            if unit_player_id != self.id {
                continue;
            }
            // move to `try_to_attack` method
            {
                for target_id in state.parts().agent.ids() {
                    let target_player_id = state.parts().belongs_to.get(target_id).0;
                    if target_player_id == self.id {
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
                let path = match path.truncate(state, unit_id) {
                    Some(path) => path,
                    None => continue,
                };
                let cost = path.cost_for(state, unit_id);
                if agent.move_points < cost {
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
