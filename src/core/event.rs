use std::collections::HashMap;
use core::{Attacks, Moves, ObjId, PlayerId, State, Unit};
use core::effect::{self, Effect};
use core::map::PosHex;

#[derive(Clone, Debug)]
pub struct Event {
    pub active_event: ActiveEvent,
    pub effects: HashMap<ObjId, Vec<Effect>>,
}

#[derive(Debug, Clone)]
pub enum ActiveEvent {
    Create(Create),
    MoveTo(MoveTo),
    Attack(Attack),
    EndTurn(EndTurn),
    BeginTurn(BeginTurn),
}

#[derive(Debug, Clone)]
pub struct Create {
    // pos: PosHex,
    pub unit: Unit,
    pub id: ObjId,
}

#[derive(Debug, Clone)]
pub struct MoveTo {
    pub path: Vec<PosHex>,
    pub cost: Moves,
    pub id: ObjId,
}

#[derive(PartialEq, Clone, Debug)]
pub enum AttackMode {
    Active,
    Reactive,
}

#[derive(Debug, Clone)]
pub struct Attack {
    pub attacker_id: ObjId,
    pub target_id: ObjId,
    pub mode: AttackMode,
}

#[derive(Debug, Clone)]
pub struct EndTurn {
    pub player_id: PlayerId,
}

#[derive(Debug, Clone)]
pub struct BeginTurn {
    pub player_id: PlayerId,
}

pub fn apply(state: &mut State, event: &Event) {
    println!("event::apply: {:?}", event);
    for (&obj_id, effects) in &event.effects {
        for effect in effects {
            effect::apply(state, obj_id, effect);
        }
    }
    apply_event(state, event);
}

pub fn apply_event(state: &mut State, event: &Event) {
    match event.active_event {
        ActiveEvent::Create(ref event) => apply_event_create(state, event),
        ActiveEvent::MoveTo(ref event) => apply_event_move_to(state, event),
        ActiveEvent::Attack(ref event) => apply_event_attack(state, event),
        ActiveEvent::EndTurn(ref event) => apply_event_end_turn(state, event),
        ActiveEvent::BeginTurn(ref event) => apply_event_begin_turn(state, event),
    }
}

fn apply_event_create(state: &mut State, event: &Create) {
    let unit = event.unit.clone();
    state.units.insert(event.id, unit);
}

fn apply_event_move_to(state: &mut State, event: &MoveTo) {
    let unit = state.units.get_mut(&event.id).unwrap();
    unit.pos = *event.path.last().unwrap();
    unit.moves.0 -= event.cost.0;
    assert!(unit.moves >= Moves(0));
}

fn apply_event_attack(state: &mut State, event: &Attack) {
    let attacker = state.units.get_mut(&event.attacker_id).unwrap();
    attacker.attacks.0 -= 1;
    assert!(attacker.attacks >= Attacks(0));
}

fn apply_event_end_turn(state: &mut State, event: &EndTurn) {
    for unit in state.units.values_mut() {
        if unit.player_id == event.player_id {
            unit.attacks.0 += 1; // TODO: get inc value from unit's type
        }
    }
}

fn apply_event_begin_turn(state: &mut State, event: &BeginTurn) {
    state.player_id = event.player_id;
    for unit in state.units.values_mut() {
        if unit.player_id == event.player_id {
            // TODO: get values from the real unit's type
            unit.moves = unit.unit_type.moves;
            unit.attacks = unit.unit_type.attacks;
        }
    }
}
