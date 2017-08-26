use std::collections::{HashMap, VecDeque};
use std::iter::FromIterator;
use rand::{thread_rng, Rng};
use core::map::PosHex;
use core::{self, Attacks, Jokers, Moves, ObjId, PlayerId, State, Strength, Unit, UnitType};
use core::command;
use core::command::Command;
use core::event::{self, ActiveEvent, Event};
use core::effect::{self, Effect};
use core::check::{check, check_attack_at};
use core::movement::{MovePoints, Path};

pub fn execute<F>(state: &mut State, command: &Command, cb: &mut F)
where
    F: FnMut(&mut State, &Event),
{
    debug!("Simulator: do_command: {:?}", command);
    if let Err(err) = check(state, command) {
        debug!("Error: {:?}", err);
        return;
    }
    match *command {
        Command::Create(ref command) => execute_create(state, cb, command),
        Command::MoveTo(ref command) => execute_move_to(state, cb, command),
        Command::Attack(ref command) => execute_attack(state, cb, command),
        Command::EndTurn(ref command) => execute_end_turn(state, cb, command),
    }
}

fn do_event<F>(state: &mut State, cb: &mut F, event: &Event)
where
    F: FnMut(&mut State, &Event),
{
    cb(state, event);
    event::apply(state, event);
}

fn execute_move_to<F>(state: &mut State, cb: &mut F, command: &command::MoveTo)
where
    F: FnMut(&mut State, &Event),
{
    let id = command.id;
    let mut cost = Some(Moves(1));
    let mut current_path = Vec::new();
    let mut remainder = VecDeque::from_iter(command.path.tiles().iter().cloned());
    while let Some(pos) = remainder.pop_front() {
        if check_reaction_attacks_at(state, id, pos) {
            current_path.push(pos);
            do_move(
                state,
                cb,
                id,
                cost.take(),
                Path::new(current_path.split_off(0)),
            );
            let attack_status = try_execute_reaction_attacks(state, cb, id);
            if attack_status == AttackStatus::Hit {
                return;
            }
        }
        current_path.push(pos);
    }
    do_move(state, cb, command.id, cost.take(), Path::new(current_path));
}

fn do_move<F>(state: &mut State, cb: &mut F, id: ObjId, cost: Option<Moves>, path: Path)
where
    F: FnMut(&mut State, &Event),
{
    let cost = cost.unwrap_or(Moves(0));
    let active_event = ActiveEvent::MoveTo(event::MoveTo { id, path, cost });
    let event = Event {
        active_event,
        effects: HashMap::new(),
    };
    do_event(state, cb, &event);
}

// TODO: try to remove code duplication with `try_execute_reaction_attacks`
fn check_reaction_attacks_at(state: &mut State, target_id: ObjId, pos: PosHex) -> bool {
    let initial_player_id = state.player_id;
    let ids: Vec<_> = state.obj_iter().collect();
    let mut result = false;
    for obj_id in ids {
        let unit_player_id = match state.unit_opt(obj_id) {
            Some(unit) => unit.player_id,
            None => continue,
        };
        if unit_player_id == initial_player_id {
            continue;
        }
        let command_attack = command::Attack {
            attacker_id: obj_id,
            target_id,
        };
        state.player_id = unit_player_id;
        if check_attack_at(state, &command_attack, pos).is_ok() {
            result = true;
            break;
        }
    }
    state.player_id = initial_player_id;
    result
}

fn execute_create<F>(state: &mut State, cb: &mut F, command: &command::Create)
where
    F: FnMut(&mut State, &Event),
{
    let active_event = ActiveEvent::Create(event::Create {
        id: command.id,
        unit: command.unit.clone(),
    });
    let event = Event {
        active_event,
        effects: HashMap::new(),
    };
    do_event(state, cb, &event);
}

#[derive(PartialEq, Clone, Debug)]
enum AttackStatus {
    Hit,
    Miss,
}

fn execute_attack_internal<F>(
    state: &mut State,
    cb: &mut F,
    command: &command::Attack,
    mode: event::AttackMode,
) -> AttackStatus
where
    F: FnMut(&mut State, &Event),
{
    let active_event = ActiveEvent::Attack(event::Attack {
        attacker_id: command.attacker_id,
        target_id: command.target_id,
        mode,
    });
    let mut effects = HashMap::new();
    let effect = if thread_rng().gen_range(0, 6) < 3 {
        if state.unit(command.target_id).strength.0 > 1 {
            Effect::Wound(effect::Wound(core::Strength(1)))
        } else {
            Effect::Kill
        }
    } else {
        Effect::Miss
    };
    let status = match effect {
        Effect::Kill | Effect::Wound(_) => AttackStatus::Hit,
        Effect::Miss => AttackStatus::Miss,
    };
    effects.insert(command.target_id, vec![effect.clone()]);
    let event = Event {
        active_event,
        effects,
    };
    do_event(state, cb, &event);
    status
}

fn try_execute_reaction_attacks<F>(state: &mut State, cb: &mut F, target_id: ObjId) -> AttackStatus
where
    F: FnMut(&mut State, &Event),
{
    let mut status = AttackStatus::Miss;
    let initial_player_id = state.player_id;
    let ids: Vec<_> = state.obj_iter().collect();
    for obj_id in ids {
        let unit_player_id = match state.unit_opt(obj_id) {
            Some(unit) => unit.player_id,
            None => continue,
        };
        if unit_player_id == initial_player_id {
            continue;
        }
        let command_attack = command::Attack {
            attacker_id: obj_id,
            target_id,
        };
        let command = command::Command::Attack(command_attack.clone());
        state.player_id = unit_player_id;
        if check(state, &command).is_err() {
            continue;
        }
        let mode = event::AttackMode::Reactive;
        let this_attack_status = execute_attack_internal(state, cb, &command_attack, mode);
        if this_attack_status != AttackStatus::Miss {
            status = this_attack_status;
        }
    }
    state.player_id = initial_player_id;
    status
}

fn execute_attack<F>(state: &mut State, cb: &mut F, command: &command::Attack)
where
    F: FnMut(&mut State, &Event),
{
    execute_attack_internal(state, cb, command, event::AttackMode::Active);
    try_execute_reaction_attacks(state, cb, command.attacker_id);
}

fn execute_end_turn<F>(state: &mut State, cb: &mut F, _: &command::EndTurn)
where
    F: FnMut(&mut State, &Event),
{
    let player_id_old = state.player_id();
    let player_id_new = next_player_id(state);
    let event_end_turn = ActiveEvent::EndTurn(event::EndTurn {
        player_id: player_id_old,
    });
    let event_end_turn = Event {
        active_event: event_end_turn,
        effects: HashMap::new(),
    };
    do_event(state, cb, &event_end_turn);
    let event_begin_turn = ActiveEvent::BeginTurn(event::BeginTurn {
        player_id: player_id_new,
    });
    let event_begin_turn = Event {
        active_event: event_begin_turn,
        effects: HashMap::new(),
    };
    do_event(state, cb, &event_begin_turn);
}

fn next_player_id(state: &State) -> PlayerId {
    let current_player_id = PlayerId(state.player_id().0 + 1);
    if current_player_id.0 < state.players_count {
        current_player_id
    } else {
        PlayerId(0)
    }
}

pub fn make_unit(player_id: PlayerId, pos: PosHex, type_name: &str) -> Unit {
    let unit_type = match type_name {
        "swordsman" => UnitType {
            name: type_name.into(),
            moves: Moves(1),
            attacks: Attacks(1),
            jokers: Jokers(1),
            reactive_attacks: Attacks(1),
            attack_distance: 1, // TODO: Distance(1)
            move_points: MovePoints(3),
            strength: Strength(4),
        },
        "spearman" => UnitType {
            name: type_name.into(),
            moves: Moves(0),
            attacks: Attacks(0),
            jokers: Jokers(1),
            reactive_attacks: Attacks(2),
            attack_distance: 2,
            move_points: MovePoints(3),
            strength: Strength(4),
        },
        "imp" => UnitType {
            name: type_name.into(),
            moves: Moves(1),
            attacks: Attacks(1),
            jokers: Jokers(1),
            reactive_attacks: Attacks(0),
            attack_distance: 1,
            move_points: MovePoints(3),
            strength: Strength(2),
        },
        _ => unimplemented!(),
    };
    Unit {
        pos,
        player_id,
        attacks: unit_type.attacks,
        moves: unit_type.moves,
        jokers: unit_type.jokers,
        strength: unit_type.strength,
        unit_type,
    }
}

// TODO: improve the API
pub fn create_objects<F>(state: &mut State, cb: &mut F)
where
    F: FnMut(&mut State, &Event),
{
    for &(player_index, (q, r), typename) in &[
        // player 0
        (0, (-3, 2), "swordsman"),
        (0, (-3, 1), "spearman"),
        (0, (-3, 0), "swordsman"),
        (0, (-3, -1), "spearman"),
        // player 1
        (1, (2, -1), "imp"),
        (1, (2, 0), "imp"),
        (1, (2, 1), "imp"),
        (1, (2, 2), "imp"),
        (1, (1, -1), "imp"),
        (1, (1, 0), "imp"),
        (1, (1, 1), "imp"),
    ] {
        let pos = PosHex {
            q: q + thread_rng().gen_range(-1, 2),
            r,
        };
        let player_id = PlayerId(player_index);
        let unit = make_unit(player_id, pos, typename);
        let id = state.alloc_id();
        let active_event = ActiveEvent::Create(event::Create { id, unit });
        let event = Event {
            active_event,
            effects: HashMap::new(),
        };
        do_event(state, cb, &event);
    }
}
