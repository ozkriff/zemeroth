use std::collections::HashMap;
use rand::{thread_rng, Rng};
use core::map::PosHex;
use core::{Attacks, Moves, ObjId, PlayerId, State, Unit};
use core::command;
use core::command::Command;
use core::event;
use core::event::Event;
use core::effect::Effect;
use core::check::check;
use core::movement::MovePoints;

pub fn execute<F>(state: &mut State, command: &Command, cb: &mut F)
where
    F: FnMut(&mut State, &Event),
{
    println!("Simulator: do_command: {:?}", command);
    if let Err(err) = check(state, command) {
        println!("Error: {:?}", err);
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
    let active_event = event::ActiveEvent::MoveTo(event::MoveTo {
        id: command.id,
        path: command.path.clone(),
    });
    let event = event::Event {
        active_event,
        effects: HashMap::new(),
    };
    do_event(state, cb, &event);
}

fn execute_create<F>(state: &mut State, cb: &mut F, command: &command::Create)
where
    F: FnMut(&mut State, &Event),
{
    let active_event = event::ActiveEvent::Create(event::Create {
        id: command.id,
        unit: command.unit.clone(),
    });
    let event = event::Event {
        active_event,
        effects: HashMap::new(),
    };
    do_event(state, cb, &event);
}

fn execute_attack_internal<F>(
    state: &mut State,
    cb: &mut F,
    command: &command::Attack,
    mode: event::AttackMode,
) where
    F: FnMut(&mut State, &Event),
{
    let active_event = event::ActiveEvent::Attack(event::Attack {
        attacker_id: command.attacker_id,
        target_id: command.target_id,
        mode,
    });
    let mut effects = HashMap::new();
    let effect = if thread_rng().gen_range(0, 6) < 3 {
        Effect::Kill
    } else {
        Effect::Miss
    };
    effects.insert(command.target_id, vec![effect.clone()]);
    let event = event::Event {
        active_event,
        effects,
    };
    do_event(state, cb, &event);
}

fn try_execute_reaction_attacks<F>(state: &mut State, cb: &mut F, target_id: ObjId)
where
    F: FnMut(&mut State, &Event),
{
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
        execute_attack_internal(state, cb, &command_attack, event::AttackMode::Reactive);
    }
    state.player_id = initial_player_id;
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
    let event_end_turn = event::ActiveEvent::EndTurn(event::EndTurn {
        player_id: player_id_old,
    });
    let event_end_turn = event::Event {
        active_event: event_end_turn,
        effects: HashMap::new(),
    };
    do_event(state, cb, &event_end_turn);
    let event_begin_turn = event::ActiveEvent::BeginTurn(event::BeginTurn {
        player_id: player_id_new,
    });
    let event_begin_turn = event::Event {
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

// TODO: improve the API
pub fn create_objects<F>(state: &mut State, cb: &mut F)
where
    F: FnMut(&mut State, &Event),
{
    for &player_index in &[0, 1] {
        for i in 0..6 {
            let id = state.alloc_id();
            let active_event = event::ActiveEvent::Create(event::Create {
                id,
                unit: Unit {
                    // TODO: really random positions
                    pos: PosHex {
                        q: match player_index {
                            0 => -2,
                            _ => 2,
                        },
                        r: -3 + i,
                    },
                    player_id: PlayerId(player_index),

                    // TODO: remove code duplication:
                    move_points: MovePoints(3),
                    attacks: Attacks(2),
                    moves: Moves(2),
                },
            });
            let event = event::Event {
                active_event,
                effects: HashMap::new(),
            };
            do_event(state, cb, &event);
        }
    }
}
