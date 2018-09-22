use core::tactical_map::{
    ability::{self, Ability},
    component::{self, Component},
    effect::{self, Effect, LastingEffect},
    event::{self, ActiveEvent, Event},
    state, Attacks, Jokers, Moves, ObjId, PlayerId, State,
};

pub fn apply(state: &mut State, event: &Event) {
    debug!("event::apply: {:?}", event);
    apply_event(state, event);
    for (&obj_id, effects) in &event.instant_effects {
        for effect in effects {
            apply_effect_instant(state, obj_id, effect);
        }
    }
    for (&obj_id, effects) in &event.timed_effects {
        for effect in effects {
            apply_effect_timed(state, obj_id, effect);
        }
    }
}

fn apply_event(state: &mut State, event: &Event) {
    match event.active_event {
        ActiveEvent::Create => {}
        ActiveEvent::MoveTo(ref ev) => apply_event_move_to(state, ev),
        ActiveEvent::Attack(ref ev) => apply_event_attack(state, ev),
        ActiveEvent::EndTurn(ref ev) => apply_event_end_turn(state, ev),
        ActiveEvent::EndBattle(ref ev) => apply_event_end_battle(state, ev),
        ActiveEvent::BeginTurn(ref ev) => apply_event_begin_turn(state, ev),
        ActiveEvent::UseAbility(ref ev) => apply_event_use_ability(state, ev),
        ActiveEvent::UsePassiveAbility(ref ev) => apply_event_use_passive_ability(state, ev),
        ActiveEvent::EffectTick(ref ev) => apply_event_effect_tick(state, ev),
        ActiveEvent::EffectEnd(ref ev) => apply_event_effect_end(state, ev),
    }
}

fn add_components(state: &mut State, id: ObjId, components: &[Component]) {
    for component in components {
        add_component(state, id, component.clone());
    }
}

fn apply_event_move_to(state: &mut State, event: &event::MoveTo) {
    let parts = state.parts_mut();
    let agent = parts.agent.get_mut(event.id);
    let pos = parts.pos.get_mut(event.id);
    pos.0 = event.path.to();
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
    let ids = parts.agent.ids_collected();
    for id in ids {
        let agent = parts.agent.get_mut(id);
        let player_id = parts.belongs_to.get(id).0;
        if player_id == event.player_id {
            agent.attacks.0 += agent.reactive_attacks.0;
        }
        if let Some(effects) = parts.effects.get_opt(id) {
            for effect in &effects.0 {
                if let LastingEffect::Stun = effect.effect {
                    agent.attacks.0 = 0;
                }
            }
        }
    }
}

fn apply_lasting_effect_stun(state: &mut State, id: ObjId) {
    let parts = state.parts_mut();
    let agent = parts.agent.get_mut(id);
    agent.moves.0 = 0;
    agent.attacks.0 = 0;
    agent.jokers.0 = 0;
}

fn apply_lasting_effect(state: &mut State, id: ObjId, effect: &LastingEffect) {
    if let LastingEffect::Stun = *effect {
        apply_lasting_effect_stun(state, id);
    }
}

fn reset_moves_and_attacks(state: &mut State, player_id: PlayerId) {
    for id in state::players_agent_ids(state, player_id) {
        let agent = state.parts_mut().agent.get_mut(id);
        agent.moves = agent.base_moves;
        agent.attacks = agent.base_attacks;
        agent.jokers = agent.base_jokers;
    }
}

fn apply_event_end_battle(state: &mut State, event: &event::EndBattle) {
    state.set_battle_result(event.result.clone());
}

fn apply_event_begin_turn(state: &mut State, event: &event::BeginTurn) {
    state.set_player_id(event.player_id);
    reset_moves_and_attacks(state, event.player_id);
    for id in state::players_agent_ids(state, event.player_id) {
        if state.parts().effects.get_opt(id).is_some() {
            let effects = state.parts().effects.get(id).clone();
            for effect in &effects.0 {
                apply_lasting_effect(state, id, &effect.effect);
            }
        }
    }
    update_cooldowns(state, event.player_id);
}

fn apply_event_use_ability(state: &mut State, event: &event::UseAbility) {
    let parts = state.parts_mut();
    if let Some(abilities) = parts.abilities.get_opt_mut(event.id) {
        for ability in &mut abilities.0 {
            if ability.ability == event.ability {
                assert_eq!(ability.status, ability::Status::Ready);
                if ability.base_cooldown != 0 {
                    ability.status = ability::Status::Cooldown(ability.base_cooldown);
                }
            }
        }
    }
    if let Some(agent) = parts.agent.get_opt_mut(event.id) {
        if agent.attacks.0 > 0 {
            agent.attacks.0 -= 1;
        } else if agent.jokers.0 > 0 {
            agent.jokers.0 -= 1;
        } else {
            panic!("internal error: can't use ability if there're not attacks or jokers");
        }
    }
    match event.ability {
        Ability::Jump(_) | Ability::Dash => {
            parts.pos.get_mut(event.id).0 = event.pos;
        }
        Ability::Rage(ability) => {
            let component = parts.agent.get_mut(event.id);
            let attacks = ability.0;
            component.attacks.0 = attacks.0 + 1;
        }
        _ => {}
    }
}

fn apply_event_use_passive_ability(_: &mut State, _: &event::UsePassiveAbility) {}

fn apply_event_effect_tick(_: &mut State, _: &event::EffectTick) {}

fn apply_event_effect_end(_: &mut State, _: &event::EffectEnd) {}

fn add_component(state: &mut State, id: ObjId, component: Component) {
    let parts = state.parts_mut();
    match component {
        Component::Pos(c) => parts.pos.insert(id, c),
        Component::Strength(c) => parts.strength.insert(id, c),
        Component::Armor(c) => parts.armor.insert(id, c),
        Component::Meta(c) => parts.meta.insert(id, c),
        Component::BelongsTo(c) => parts.belongs_to.insert(id, c),
        Component::Agent(c) => parts.agent.insert(id, c),
        Component::Blocker(c) => parts.blocker.insert(id, c),
        Component::Abilities(c) => parts.abilities.insert(id, c),
        Component::PassiveAbilities(c) => parts.passive_abilities.insert(id, c),
        Component::Effects(c) => parts.effects.insert(id, c),
        Component::Schedule(c) => parts.schedule.insert(id, c),
    }
}

fn apply_effect_timed(state: &mut State, id: ObjId, timed_effect: &effect::TimedEffect) {
    let parts = state.parts_mut();
    debug!("effect::apply_timed: {:?}", timed_effect);
    let effects = &mut parts.effects;
    if effects.get_opt(id).is_none() {
        effects.insert(id, component::Effects(Vec::new()))
    }
    effects.get_mut(id).0.push(timed_effect.clone());
}

fn apply_effect_instant(state: &mut State, id: ObjId, effect: &Effect) {
    debug!("effect::apply_instant: {:?} ({})", effect, effect.to_str());
    match *effect {
        Effect::Create(ref effect) => apply_effect_create(state, id, effect),
        Effect::Kill => apply_effect_kill(state, id),
        Effect::Vanish => apply_effect_vanish(state, id),
        Effect::Stun => apply_effect_stun(state, id),
        Effect::Heal(ref effect) => apply_effect_heal(state, id, effect),
        Effect::Wound(ref effect) => apply_effect_wound(state, id, effect),
        Effect::Knockback(ref effect) => apply_effect_knockback(state, id, effect),
        Effect::FlyOff(ref effect) => apply_effect_fly_off(state, id, effect),
        Effect::Throw(ref effect) => apply_effect_throw(state, id, effect),
        Effect::Miss => {}
    }
}

fn apply_effect_create(state: &mut State, id: ObjId, effect: &effect::Create) {
    add_components(state, id, &effect.components);
}

fn apply_effect_kill(state: &mut State, id: ObjId) {
    let parts = state.parts_mut();
    parts.remove(id);
}

fn apply_effect_vanish(state: &mut State, id: ObjId) {
    let parts = state.parts_mut();
    parts.remove(id);
}

fn apply_effect_stun(state: &mut State, id: ObjId) {
    let parts = state.parts_mut();
    let agent = parts.agent.get_mut(id);
    agent.moves.0 = 0;
    agent.attacks.0 = 0;
    agent.jokers.0 = 0;
}

// TODO: split `Heal` effect into two? `Heal` + `RemoveLastingEffects`?
fn apply_effect_heal(state: &mut State, id: ObjId, effect: &effect::Heal) {
    let parts = state.parts_mut();
    {
        let component = parts.strength.get_mut(id);
        component.strength.0 += effect.strength.0;
        if component.strength > component.base_strength {
            component.strength = component.base_strength;
        }
    }
    if let Some(effects) = parts.effects.get_opt_mut(id) {
        effects.0.clear();
    }
}

fn apply_effect_wound(state: &mut State, id: ObjId, effect: &effect::Wound) {
    let parts = state.parts_mut();
    let damage = effect.damage.0;
    assert!(damage >= 0);
    if let Some(armor) = parts.armor.get_opt_mut(id) {
        let armor_break = effect.armor_break;
        armor.armor.0 -= armor_break.0;
        if armor.armor.0 < 0 {
            armor.armor.0 = 0;
        }
        assert!(armor.armor.0 >= 0);
    }
    {
        let strength = parts.strength.get_mut(id);
        strength.strength.0 -= damage;
        assert!(strength.strength.0 > 0);
    }
    {
        let agent = parts.agent.get_mut(id);
        agent.attacks.0 -= 1;
        if agent.attacks.0 < 0 {
            agent.attacks.0 = 0;
        }
    }
}

fn apply_effect_knockback(state: &mut State, id: ObjId, effect: &effect::Knockback) {
    assert!(state.map().is_inboard(effect.from));
    assert!(state.map().is_inboard(effect.to));
    assert!(!state::is_tile_blocked(state, effect.to));
    let parts = state.parts_mut();
    parts.pos.get_mut(id).0 = effect.to;
    // TODO: push anyone who's in the way aside
}

fn apply_effect_fly_off(state: &mut State, id: ObjId, effect: &effect::FlyOff) {
    assert!(state.map().is_inboard(effect.from));
    assert!(state.map().is_inboard(effect.to));
    assert!(!state::is_tile_blocked(state, effect.to));
    let parts = state.parts_mut();
    parts.pos.get_mut(id).0 = effect.to;
}

fn apply_effect_throw(state: &mut State, id: ObjId, effect: &effect::Throw) {
    assert!(state.map().is_inboard(effect.from));
    assert!(state.map().is_inboard(effect.to));
    assert!(!state::is_tile_blocked(state, effect.to));
    let parts = state.parts_mut();
    parts.pos.get_mut(id).0 = effect.to;
}

fn update_cooldowns_for_object(state: &mut State, id: ObjId) {
    let parts = state.parts_mut();
    if let Some(abilities) = parts.abilities.get_opt_mut(id) {
        for ability in &mut abilities.0 {
            ability.status.update();
        }
    }
}

fn update_cooldowns(state: &mut State, player_id: PlayerId) {
    for id in state::players_agent_ids(state, player_id) {
        update_cooldowns_for_object(state, id);
    }
}
