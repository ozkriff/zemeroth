use core::State;
use core::command::{self, Command};
use core::map::{self, PosHex};
use core::{self, Attacks, Jokers, Moves};

pub fn check(state: &State, command: &Command) -> Result<(), Error> {
    match *command {
        Command::Create(ref command) => check_create(state, command),
        Command::MoveTo(ref command) => check_move_to(state, command),
        Command::Attack(ref command) => check_attack(state, command),
        Command::EndTurn(ref command) => check_end_turn(state, command),
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Error {
    NotEnoughMovePoints,
    BadActorId,
    BadTargetId,
    TileIsOccupied,
    DistanceIsTooBig,
    CanNotCommandEnemyUnits,
    NotEnoughMoves,
    NotEnoughAttacks,
    BadPos,
}

fn check_move_to(state: &State, command: &command::MoveTo) -> Result<(), Error> {
    let agent = match state.parts.agent.get_opt(command.id) {
        Some(agent) => agent,
        None => return Err(Error::BadActorId),
    };
    let unit_player_id = state.parts().belongs_to.get(command.id).0;
    if unit_player_id != state.player_id() {
        return Err(Error::CanNotCommandEnemyUnits);
    }
    if agent.moves == Moves(0) && agent.jokers == Jokers(0) {
        return Err(Error::NotEnoughMoves);
    }
    for &pos in command.path.tiles() {
        if !state.map().is_inboard(pos) {
            return Err(Error::BadPos);
        }
    }
    for step in command.path.steps() {
        if !core::object_ids_at(state, step.to).is_empty() {
            return Err(Error::TileIsOccupied);
        }
    }
    let cost = command.path.cost_for(state, command.id);
    if cost > agent.move_points {
        return Err(Error::NotEnoughMovePoints);
    }
    Ok(())
}

fn check_create(state: &State, command: &command::Create) -> Result<(), Error> {
    if !state.map().is_inboard(command.pos) {
        return Err(Error::BadPos);
    }
    if !core::object_ids_at(state, command.pos).is_empty() {
        return Err(Error::TileIsOccupied);
    }
    Ok(())
}

fn check_attack(state: &State, command: &command::Attack) -> Result<(), Error> {
    let target_pos = match state.parts.pos.get_opt(command.target_id) {
        Some(pos) => pos.0,
        None => return Err(Error::BadTargetId),
    };
    check_attack_at(state, command, target_pos)
}

pub fn check_attack_at(state: &State, command: &command::Attack, at: PosHex) -> Result<(), Error> {
    let parts = state.parts();
    let attacker_agent = match parts.agent.get_opt(command.attacker_id) {
        Some(agent) => agent,
        None => return Err(Error::BadActorId),
    };
    let attacker_pos = parts.pos.get(command.attacker_id).0;
    let attacker_player_id = parts.belongs_to.get(command.attacker_id).0;
    if attacker_player_id != state.player_id() {
        return Err(Error::CanNotCommandEnemyUnits);
    }
    if parts.agent.get_opt(command.target_id).is_none() {
        return Err(Error::BadTargetId);
    };
    if !state.map().is_inboard(at) {
        return Err(Error::BadPos);
    }
    if attacker_agent.attacks == Attacks(0) && attacker_agent.jokers == Jokers(0) {
        return Err(Error::NotEnoughAttacks);
    }
    let dist = map::distance_hex(attacker_pos, at);
    if dist > attacker_agent.attack_distance {
        return Err(Error::DistanceIsTooBig);
    }
    Ok(())
}

fn check_end_turn(_: &State, _: &command::EndTurn) -> Result<(), Error> {
    Ok(())
}
