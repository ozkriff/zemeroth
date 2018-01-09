use core::{Attacks, Jokers, Moves, ObjId, State};
use core::component::Component;
use core::effect::{self, Effect};
use core::event::{self, ActiveEvent, Event};

pub fn apply(state: &mut State, event: &Event) {
    debug!("apply: {:?}", event);
    apply_event(state, event);
    for (&obj_id, effects) in &event.effects {
        for effect in effects {
            apply_effect(state, obj_id, effect);
        }
    }
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

fn apply_event_create(state: &mut State, event: &event::Create) {
    let parts = state.parts_mut();
    let id = event.id;
    for component in &event.components {
        match component.clone() {
            Component::Pos(c) => parts.pos.insert(id, c),
            Component::Strength(c) => parts.strength.insert(id, c),
            Component::Meta(c) => parts.meta.insert(id, c),
            Component::BelongsTo(c) => parts.belongs_to.insert(id, c),
            Component::Agent(c) => parts.agent.insert(id, c),
            Component::Blocker(c) => parts.blocker.insert(id, c),
        }
    }
}

fn apply_event_move_to(state: &mut State, event: &event::MoveTo) {
    let parts = state.parts_mut();
    let agent = parts.agent.get_mut(event.id);
    let pos = parts.pos.get_mut(event.id);
    pos.0 = *event.path.tiles().last().unwrap();
    if agent.moves.0 > 0 {
        agent.moves.0 -= event.cost.0;
    } else {
        agent.jokers.0 -= event.cost.0;
    }
    assert!(agent.moves >= Moves(0));
    assert!(agent.jokers >= Jokers(0));
}

fn apply_event_attack(state: &mut State, event: &event::Attack) {
    let parts = state.parts_mut();
    let agent = parts.agent.get_mut(event.attacker_id);
    if agent.attacks.0 > 0 {
        agent.attacks.0 -= 1;
    } else {
        agent.jokers.0 -= 1;
    }
    assert!(agent.attacks >= Attacks(0));
    assert!(agent.jokers >= Jokers(0));
}

fn apply_event_end_turn(state: &mut State, event: &event::EndTurn) {
    let parts = state.parts_mut();
    let ids: Vec<_> = parts.agent.ids().collect();
    for id in ids {
        let agent = parts.agent.get_mut(id);
        let player_id = parts.belongs_to.get(id).0;
        if player_id == event.player_id {
            agent.attacks.0 += agent.reactive_attacks.0;
        }
    }
}

fn apply_event_begin_turn(state: &mut State, event: &event::BeginTurn) {
    state.set_player_id(event.player_id);
    let parts = state.parts_mut();
    let ids: Vec<_> = parts.agent.ids().collect();
    for id in ids {
        let agent = parts.agent.get_mut(id);
        let player_id = parts.belongs_to.get(id).0;
        if player_id == event.player_id {
            agent.moves = agent.base_moves;
            agent.attacks = agent.base_attacks;
            agent.jokers = agent.base_jokers;
        }
    }
}

fn apply_effect(state: &mut State, id: ObjId, effect: &Effect) {
    debug!("apply_effect: {:?}", effect);
    match *effect {
        Effect::Kill => apply_effect_kill(state, id),
        Effect::Wound(ref effect) => apply_effect_wound(state, id, effect),
        Effect::Miss => apply_effect_miss(state, id),
    }
}

fn apply_effect_kill(state: &mut State, id: ObjId) {
    state.parts_mut().remove(id);
}

fn apply_effect_wound(state: &mut State, id: ObjId, effect: &effect::Wound) {
    let damage = effect.0;
    let strength = state.parts_mut().strength.get_mut(id);
    strength.strength.0 -= damage.0;
    assert!(strength.strength.0 > 0);
}

fn apply_effect_miss(_: &mut State, _: ObjId) {}
