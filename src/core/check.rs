use core::State;
use core::command::{self, Command};
use core::map::{self, PosHex};
use core::{Attacks, Jokers, Moves};

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
    ObjectAlreadyExists,
    TileIsOccupied,
    DistanceIsTooBig,
    CanNotCommandEnemyUnits,
    NotEnoughMoves,
    NotEnoughAttacks,
    BadPos,
}

fn check_move_to(state: &State, command: &command::MoveTo) -> Result<(), Error> {
    let unit = match state.unit_opt(command.id) {
        Some(unit) => unit,
        None => return Err(Error::BadActorId),
    };
    if unit.player_id != state.player_id() {
        return Err(Error::CanNotCommandEnemyUnits);
    }
    if unit.moves == Moves(0) && unit.jokers == Jokers(0) {
        return Err(Error::NotEnoughMoves);
    }
    for &pos in command.path.tiles() {
        if !state.map().is_inboard(pos) {
            return Err(Error::BadPos);
        }
    }
    let cost = command.path.cost_for(state, unit);
    if cost > unit.unit_type.move_points {
        return Err(Error::NotEnoughMovePoints);
    }
    Ok(())
}

fn check_create(state: &State, command: &command::Create) -> Result<(), Error> {
    if state.unit_opt(command.id).is_some() {
        return Err(Error::ObjectAlreadyExists);
    }
    if command.unit.player_id != state.player_id() {
        return Err(Error::CanNotCommandEnemyUnits);
    }
    if !state.map().is_inboard(command.unit.pos) {
        return Err(Error::BadPos);
    }
    if !state.units_at(command.unit.pos).is_empty() {
        return Err(Error::TileIsOccupied);
    }
    Ok(())
}

fn check_attack(state: &State, command: &command::Attack) -> Result<(), Error> {
    let target = match state.unit_opt(command.target_id) {
        Some(unit) => unit,
        None => return Err(Error::BadTargetId),
    };
    check_attack_at(state, command, target.pos)
}

pub fn check_attack_at(state: &State, command: &command::Attack, at: PosHex) -> Result<(), Error> {
    let attacker = match state.unit_opt(command.attacker_id) {
        Some(unit) => unit,
        None => return Err(Error::BadActorId),
    };
    if attacker.player_id != state.player_id() {
        return Err(Error::CanNotCommandEnemyUnits);
    }
    if state.unit_opt(command.target_id).is_none() {
        return Err(Error::BadTargetId);
    };
    if !state.map().is_inboard(at) {
        return Err(Error::BadPos);
    }
    if attacker.attacks == Attacks(0) && attacker.jokers == Jokers(0) {
        return Err(Error::NotEnoughAttacks);
    }
    let dist = map::distance_hex(attacker.pos, at);
    let max_dist = attacker.unit_type.attack_distance;
    if dist > max_dist {
        return Err(Error::DistanceIsTooBig);
    }
    Ok(())
}

fn check_end_turn(_: &State, _: &command::EndTurn) -> Result<(), Error> {
    Ok(())
}
