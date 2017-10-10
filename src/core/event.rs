use std::collections::HashMap;
use core::{Attacks, Jokers, Moves, ObjId, PlayerId, PosHex, State};
use core::component::Component;
use core::effect::{self, Effect};
use core::movement::Path;

#[derive(Clone, Debug)]
pub struct Event {
    pub active_event: ActiveEvent,
    pub actor_ids: Vec<ObjId>,
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
    pub id: ObjId,
    pub pos: PosHex,
    pub prototype: String,
    pub components: Vec<Component>,
}

#[derive(Debug, Clone)]
pub struct MoveTo {
    pub path: Path,
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
    debug!("event::apply: {:?}", event);
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
    let id = event.id;
    for component in &event.components {
        match component.clone() {
            Component::Pos(c) => state.parts.pos.insert(id, c),
            Component::Strength(c) => state.parts.strength.insert(id, c),
            Component::Meta(c) => state.parts.meta.insert(id, c),
            Component::BelongsTo(c) => state.parts.belongs_to.insert(id, c),
            Component::Agent(c) => state.parts.agent.insert(id, c),
        }
    }
}

fn apply_event_move_to(state: &mut State, event: &MoveTo) {
    let agent = state.parts.agent.get_mut(event.id);
    let pos = state.parts.pos.get_mut(event.id);
    pos.0 = *event.path.tiles().last().unwrap();
    if agent.moves.0 > 0 {
        agent.moves.0 -= event.cost.0;
    } else {
        agent.jokers.0 -= event.cost.0;
    }
    assert!(agent.moves >= Moves(0));
    assert!(agent.jokers >= Jokers(0));
}

fn apply_event_attack(state: &mut State, event: &Attack) {
    let agent = state.parts.agent.get_mut(event.attacker_id);
    if agent.attacks.0 > 0 {
        agent.attacks.0 -= 1;
    } else {
        agent.jokers.0 -= 1;
    }
    assert!(agent.attacks >= Attacks(0));
    assert!(agent.jokers >= Jokers(0));
}

fn apply_event_end_turn(state: &mut State, event: &EndTurn) {
    let ids: Vec<_> = state.parts.agent.ids().collect();
    for id in ids {
        let agent = state.parts.agent.get_mut(id);
        let player_id = state.parts.belongs_to.get(id).0;
        if player_id == event.player_id {
            agent.attacks.0 += agent.reactive_attacks.0;
        }
    }
}

fn apply_event_begin_turn(state: &mut State, event: &BeginTurn) {
    state.player_id = event.player_id;
    let ids: Vec<_> = state.parts.agent.ids().collect();
    for id in ids {
        let agent = state.parts.agent.get_mut(id);
        let player_id = state.parts.belongs_to.get(id).0;
        if player_id == event.player_id {
            agent.moves = agent.base_moves;
            agent.attacks = agent.base_attacks;
            agent.jokers = agent.base_jokers;
        }
    }
}
