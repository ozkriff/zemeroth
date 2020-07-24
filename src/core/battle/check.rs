use log::trace;

use crate::core::{
    battle::{
        self,
        ability::{self, Ability},
        command::{self, Command},
        state, Attacks, Id, Jokers, Moves, State,
    },
    map::{self, Distance, PosHex},
};

pub fn check(state: &State, command: &Command) -> Result<(), Error> {
    trace!("check: {:?}", command);
    if state.battle_result().is_some() {
        return Err(Error::BattleEnded);
    }
    match *command {
        Command::Create(ref command) => check_command_create(state, command),
        Command::MoveTo(ref command) => check_command_move_to(state, command),
        Command::Attack(ref command) => check_command_attack(state, command),
        Command::EndTurn(ref command) => check_command_end_turn(state, command),
        Command::UseAbility(ref command) => check_command_use_ability(state, command),
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Error {
    NotEnoughMovePoints,
    BadActorId,
    BadTargetId,
    BadTargeType,
    TileIsBlocked,
    DistanceIsTooBig,
    DistanceIsTooSmall,
    CanNotCommandEnemyAgents,
    NotEnoughMoves,
    NotEnoughAttacks,
    AbilityIsNotReady,
    NoSuchAbility,
    NoTarget,
    BadPos,
    BadActorType,
    BattleEnded,
}

fn check_command_move_to(state: &State, command: &command::MoveTo) -> Result<(), Error> {
    let agent = try_get_actor(state, command.id)?;
    let agent_player_id = state.parts().belongs_to.get(command.id).0;
    if agent_player_id != state.player_id() {
        return Err(Error::CanNotCommandEnemyAgents);
    }
    check_agent_can_move(state, command.id)?;
    for step in command.path.steps() {
        check_not_blocked_and_is_inboard(state, step.to)?;
    }
    let cost = command.path.cost_for(state, command.id);
    if cost > agent.move_points {
        return Err(Error::NotEnoughMovePoints);
    }
    Ok(())
}

fn check_command_create(state: &State, command: &command::Create) -> Result<(), Error> {
    check_not_blocked_and_is_inboard(state, command.pos)?;
    Ok(())
}

fn check_command_attack(state: &State, command: &command::Attack) -> Result<(), Error> {
    if command.attacker_id == command.target_id {
        return Err(Error::BadTargetId);
    }
    let target_pos = match state.parts().pos.get_opt(command.target_id) {
        Some(pos) => pos.0,
        None => return Err(Error::BadTargetId),
    };
    let parts = state.parts();
    let attacker_agent = try_get_actor(state, command.attacker_id)?;
    let attacker_pos = parts.pos.get(command.attacker_id).0;
    let attacker_player_id = parts.belongs_to.get(command.attacker_id).0;
    if attacker_player_id != state.player_id() {
        return Err(Error::CanNotCommandEnemyAgents);
    }
    if parts.agent.get_opt(command.target_id).is_none() {
        return Err(Error::BadTargetId);
    };
    check_is_inboard(state, target_pos)?;
    check_agent_can_attack(state, command.attacker_id)?;
    check_max_distance(attacker_pos, target_pos, attacker_agent.attack_distance)?;
    Ok(())
}

fn check_command_end_turn(_: &State, _: &command::EndTurn) -> Result<(), Error> {
    Ok(())
}

fn check_command_use_ability(state: &State, command: &command::UseAbility) -> Result<(), Error> {
    check_agent_belongs_to_correct_player(state, command.id)?;
    check_agent_can_attack(state, command.id)?;
    check_agent_ability_ready(state, command.id, &command.ability)?;
    match command.ability {
        Ability::Knockback(a) => check_ability_knockback(state, command.id, command.pos, a),
        Ability::Club => check_ability_club(state, command.id, command.pos),
        Ability::Jump(a) => check_ability_jump(state, command.id, command.pos, a),
        Ability::Poison => check_ability_poison(state, command.id, command.pos),
        Ability::Bomb(a) => check_ability_bomb(state, command.id, command.pos, a),
        Ability::BombPush(a) => check_ability_bomb_push(state, command.id, command.pos, a),
        Ability::BombFire(a) => check_ability_bomb_fire(state, command.id, command.pos, a),
        Ability::BombPoison(a) => check_ability_bomb_poison(state, command.id, command.pos, a),
        Ability::BombDemonic(a) => check_ability_bomb_demonic(state, command.id, command.pos, a),
        Ability::Summon => check_ability_summon(state, command.id, command.pos),
        Ability::Vanish => check_ability_vanish(state, command.id, command.pos),
        Ability::Dash => check_ability_dash(state, command.id, command.pos),
        Ability::Rage(a) => check_ability_rage(state, command.id, command.pos, a),
        Ability::Heal(a) => check_ability_heal(state, command.id, command.pos, a),
        Ability::Bloodlust => check_ability_bloodlust(state, command.id, command.pos),
        Ability::ExplodePush
        | Ability::ExplodeDamage
        | Ability::ExplodeFire
        | Ability::ExplodePoison => check_ability_explode(state, command.id, command.pos),
    }
}

fn check_ability_knockback(
    state: &State,
    id: Id,
    pos: PosHex,
    _: ability::Knockback,
) -> Result<(), Error> {
    let selected_pos = state.parts().pos.get(id).0;
    check_min_distance(selected_pos, pos, Distance(1))?;
    check_max_distance(selected_pos, pos, Distance(1))?;
    if state::blocker_id_at_opt(state, pos).is_none() {
        return Err(Error::NoTarget);
    }
    Ok(())
}

fn check_ability_jump(
    state: &State,
    id: Id,
    pos: PosHex,
    ability: ability::Jump,
) -> Result<(), Error> {
    let parts = state.parts();
    let agent_pos = parts.pos.get(id).0;
    check_min_distance(agent_pos, pos, Distance(2))?;
    check_max_distance(agent_pos, pos, ability.0)?;
    check_not_blocked_and_is_inboard(state, pos)?;
    Ok(())
}

fn check_ability_club(state: &State, id: Id, pos: PosHex) -> Result<(), Error> {
    let selected_pos = state.parts().pos.get(id).0;
    check_min_distance(selected_pos, pos, Distance(1))?;
    check_max_distance(selected_pos, pos, Distance(1))?;
    if state::agent_id_at_opt(state, pos).is_none() {
        return Err(Error::NoTarget);
    }
    Ok(())
}

fn check_ability_poison(state: &State, id: Id, pos: PosHex) -> Result<(), Error> {
    let selected_pos = state.parts().pos.get(id).0;
    check_min_distance(selected_pos, pos, Distance(1))?;
    check_max_distance(selected_pos, pos, Distance(3))?;
    if state::blocker_id_at_opt(state, pos).is_none() {
        return Err(Error::NoTarget);
    }
    Ok(())
}

fn check_ability_explode(state: &State, id: Id, pos: PosHex) -> Result<(), Error> {
    check_object_pos(state, id, pos)
}

fn check_ability_bomb(
    state: &State,
    id: Id,
    pos: PosHex,
    ability: ability::Bomb,
) -> Result<(), Error> {
    let agent_pos = state.parts().pos.get(id).0;
    check_max_distance(agent_pos, pos, ability.0)?;
    check_not_blocked_and_is_inboard(state, pos)?;
    Ok(())
}

fn check_ability_bomb_push(
    state: &State,
    id: Id,
    pos: PosHex,
    ability: ability::BombPush,
) -> Result<(), Error> {
    let agent_pos = state.parts().pos.get(id).0;
    check_max_distance(agent_pos, pos, ability.throw_distance)?;
    check_not_blocked_and_is_inboard(state, pos)?;
    Ok(())
}

fn check_ability_bomb_fire(
    state: &State,
    id: Id,
    pos: PosHex,
    ability: ability::BombFire,
) -> Result<(), Error> {
    let agent_pos = state.parts().pos.get(id).0;
    check_max_distance(agent_pos, pos, ability.0)?;
    check_not_blocked_and_is_inboard(state, pos)?;
    Ok(())
}

fn check_ability_bomb_poison(
    state: &State,
    id: Id,
    pos: PosHex,
    ability: ability::BombPoison,
) -> Result<(), Error> {
    let agent_pos = state.parts().pos.get(id).0;
    check_max_distance(agent_pos, pos, ability.0)?;
    check_not_blocked_and_is_inboard(state, pos)?;
    Ok(())
}

fn check_ability_bomb_demonic(
    state: &State,
    id: Id,
    pos: PosHex,
    ability: ability::BombDemonic,
) -> Result<(), Error> {
    let agent_pos = state.parts().pos.get(id).0;
    check_max_distance(agent_pos, pos, ability.0)?;
    check_not_blocked_and_is_inboard(state, pos)?;
    Ok(())
}

fn check_ability_summon(state: &State, id: Id, pos: PosHex) -> Result<(), Error> {
    check_object_pos(state, id, pos)
}

fn check_ability_vanish(state: &State, id: Id, pos: PosHex) -> Result<(), Error> {
    if state.parts().agent.get_opt(id).is_some() {
        return Err(Error::BadActorType);
    }
    let actor_pos = match state.parts().pos.get_opt(id) {
        Some(pos) => pos.0,
        None => return Err(Error::BadActorType),
    };
    if pos != actor_pos {
        return Err(Error::BadPos);
    }
    Ok(())
}

fn check_ability_dash(state: &State, id: Id, pos: PosHex) -> Result<(), Error> {
    let agent_pos = state.parts().pos.get(id).0;
    check_max_distance(agent_pos, pos, Distance(1))?;
    check_not_blocked_and_is_inboard(state, pos)?;
    Ok(())
}

fn check_ability_rage(state: &State, id: Id, pos: PosHex, _: ability::Rage) -> Result<(), Error> {
    check_object_pos(state, id, pos)
}

fn check_ability_heal(state: &State, id: Id, pos: PosHex, _: ability::Heal) -> Result<(), Error> {
    let agent_pos = state.parts().pos.get(id).0;
    check_max_distance(agent_pos, pos, Distance(1))?;
    let target_id = match state::agent_id_at_opt(state, pos) {
        Some(id) => id,
        None => return Err(Error::NoTarget),
    };
    match state.parts().strength.get_opt(target_id) {
        Some(strength) => {
            if strength.strength == strength.base_strength {
                return Err(Error::BadTargeType);
            }
        }
        None => return Err(Error::BadActorId),
    }
    Ok(())
}

fn check_ability_bloodlust(state: &State, _id: Id, pos: PosHex) -> Result<(), Error> {
    // TODO: check that the target belongs to the same player
    if state::agent_id_at_opt(state, pos).is_none() {
        return Err(Error::NoTarget);
    }
    Ok(())
}

fn try_get_actor(state: &State, id: Id) -> Result<&battle::component::Agent, Error> {
    match state.parts().agent.get_opt(id) {
        Some(agent) => Ok(agent),
        None => Err(Error::BadActorId),
    }
}

fn check_agent_ability_ready(
    state: &State,
    id: Id,
    expected_ability: &Ability,
) -> Result<(), Error> {
    let mut found = false;
    let abilities = match state.parts().abilities.get_opt(id) {
        Some(abilities) => &abilities.0,
        None => return Err(Error::BadActorType),
    };
    for ability in abilities {
        if ability.ability == *expected_ability {
            found = true;
            if ability.status != ability::Status::Ready {
                return Err(Error::AbilityIsNotReady);
            }
        }
    }
    if !found {
        return Err(Error::NoSuchAbility);
    }
    Ok(())
}

fn check_agent_belongs_to_correct_player(state: &State, id: Id) -> Result<(), Error> {
    let agent_player_id = state.parts().belongs_to.get(id).0;
    if agent_player_id != state.player_id() {
        return Err(Error::CanNotCommandEnemyAgents);
    }
    Ok(())
}

fn check_agent_can_attack(state: &State, id: Id) -> Result<(), Error> {
    let agent = try_get_actor(state, id)?;
    if agent.attacks == Attacks(0) && agent.jokers == Jokers(0) {
        return Err(Error::NotEnoughAttacks);
    }
    Ok(())
}

fn check_agent_can_move(state: &State, id: Id) -> Result<(), Error> {
    let agent = try_get_actor(state, id)?;
    if agent.moves == Moves(0) && agent.jokers == Jokers(0) {
        return Err(Error::NotEnoughMoves);
    }
    Ok(())
}

fn check_min_distance(from: PosHex, to: PosHex, min: Distance) -> Result<(), Error> {
    let dist = map::distance_hex(from, to);
    if dist < min {
        return Err(Error::DistanceIsTooSmall);
    }
    Ok(())
}

fn check_max_distance(from: PosHex, to: PosHex, max: Distance) -> Result<(), Error> {
    let dist = map::distance_hex(from, to);
    if dist > max {
        return Err(Error::DistanceIsTooBig);
    }
    Ok(())
}

fn check_not_blocked_and_is_inboard(state: &State, pos: PosHex) -> Result<(), Error> {
    check_is_inboard(state, pos)?;
    if state::is_tile_blocked(state, pos) {
        return Err(Error::TileIsBlocked);
    }
    Ok(())
}

fn check_is_inboard(state: &State, pos: PosHex) -> Result<(), Error> {
    if !state.map().is_inboard(pos) {
        return Err(Error::BadPos);
    }
    Ok(())
}

fn check_object_pos(state: &State, id: Id, expected_pos: PosHex) -> Result<(), Error> {
    let real_pos = state.parts().pos.get(id).0;
    if real_pos != expected_pos {
        return Err(Error::BadPos);
    }
    Ok(())
}
