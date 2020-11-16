use log::trace;

use crate::core::battle::{
    ability::{self, Ability},
    component::{self, Component, Parts, PlannedAbility},
    effect::{self, Duration, Effect},
    event::{self, ActiveEvent, Event},
    state, Attacks, Id, Jokers, Moves, Phase, PlayerId, State, Strength,
};

pub fn apply(state: &mut State, event: &Event) {
    trace!("event::apply: {:?}", event);
    apply_event(state, event);
    for &(obj_id, ref effects) in &event.instant_effects {
        for effect in effects {
            apply_effect_instant(state, obj_id, effect);
        }
    }
    for &(obj_id, ref effects) in &event.timed_effects {
        for effect in effects {
            apply_effect_timed(state, obj_id, effect);
        }
    }
    for &(id, ref abilities) in &event.scheduled_abilities {
        for planned_ability in abilities {
            apply_scheduled_ability(state, id, planned_ability);
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

fn add_components(state: &mut State, id: Id, components: &[Component]) {
    let parts = state.parts_mut();
    for component in components {
        add_component(parts, id, component.clone());
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
    // Update attacks
    {
        let parts = state.parts_mut();
        for id in parts.agent.ids_collected() {
            let agent = parts.agent.get_mut(id);
            let player_id = parts.belongs_to.get(id).0;
            if player_id == event.player_id {
                agent.attacks.0 += agent.reactive_attacks.0;
            }
            if let Some(effects) = parts.effects.get_opt(id) {
                for effect in &effects.0 {
                    if let effect::Lasting::Stun = effect.effect {
                        agent.attacks.0 = 0;
                    }
                }
            }
        }
    }
    // Remove outdated planned abilities
    for id in state.parts().schedule.ids_collected() {
        state
            .parts_mut()
            .schedule
            .get_mut(id)
            .planned
            .retain(|p| p.rounds.0 > 0);
    }
    // Remove outdated lasting effect
    for id in state.parts().effects.ids_collected() {
        let mut effects = state.parts().effects.get(id).0.clone();
        effects.retain(|effect| !state::is_lasting_effect_over(state, id, effect));
        state.parts_mut().effects.get_mut(id).0 = effects;
    }
}

fn apply_lasting_effect_stun(state: &mut State, id: Id) {
    let parts = state.parts_mut();
    let agent = parts.agent.get_mut(id);
    agent.moves.0 = 0;
    agent.attacks.0 = 0;
    agent.jokers.0 = 0;
}

fn apply_lasting_effect(state: &mut State, id: Id, effect: &effect::Lasting) {
    if let effect::Lasting::Stun = *effect {
        apply_lasting_effect_stun(state, id);
    }
}

fn update_lasting_effects_duration(state: &mut State) {
    let phase = Phase::from_player_id(state.player_id());
    for id in state.parts().effects.ids_collected() {
        for effect in &mut state.parts_mut().effects.get_mut(id).0 {
            if effect.phase == phase {
                if let Duration::Rounds(ref mut rounds) = effect.duration {
                    assert!(rounds.0 > 0);
                    rounds.decrease();
                }
            }
        }
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
    update_lasting_effects_duration(state);
    reset_moves_and_attacks(state, event.player_id);
    apply_lasting_effects(state);
    update_cooldowns(state, event.player_id);
    tick_planned_abilities(state);
}

fn apply_event_use_ability(state: &mut State, event: &event::UseAbility) {
    let id = event.id;
    let parts = state.parts_mut();
    if let Some(abilities) = parts.abilities.get_opt_mut(id) {
        for r_ability in &mut abilities.0 {
            if r_ability.ability == event.ability {
                let cooldown = r_ability.ability.base_cooldown();
                assert_eq!(r_ability.status, ability::Status::Ready);
                if !cooldown.is_zero() {
                    r_ability.status = ability::Status::Cooldown(cooldown);
                }
            }
        }
    }
    if let Some(agent) = parts.agent.get_opt_mut(id) {
        if agent.attacks.0 > 0 {
            agent.attacks.0 -= 1;
        } else if agent.jokers.0 > 0 {
            agent.jokers.0 -= 1;
        } else {
            panic!("internal error: can't use ability if there're not attacks or jokers");
        }
    }
    match event.ability {
        Ability::Jump | Ability::LongJump | Ability::Dash => {
            parts.pos.get_mut(id).0 = event.pos;
        }
        Ability::Rage => {
            let component = parts.agent.get_mut(id);
            component.attacks.0 += 3;
        }
        Ability::Summon => {
            assert!(parts.summoner.get_opt(id).is_some());
            let mut summoner = parts.summoner.get_mut(id);
            summoner.count += 1;
        }
        _ => {}
    }
}

fn apply_event_use_passive_ability(_: &mut State, _: &event::UsePassiveAbility) {}

fn apply_event_effect_tick(_: &mut State, _: &event::EffectTick) {}

fn apply_event_effect_end(_: &mut State, _: &event::EffectEnd) {}

fn add_component(parts: &mut Parts, id: Id, component: Component) {
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
        Component::Summoner(c) => parts.summoner.insert(id, c),
    }
}

fn apply_scheduled_ability(state: &mut State, id: Id, planned_ability: &PlannedAbility) {
    trace!("effect::apply_scheduled_ability: {:?}", planned_ability);
    let schedule = &mut state.parts_mut().schedule;
    if schedule.get_opt(id).is_none() {
        schedule.insert(id, component::Schedule::default());
    }
    let planned = &mut schedule.get_mut(id).planned;
    if let Some(i) = planned
        .iter()
        .position(|e| e.ability == planned_ability.ability)
    {
        planned[i] = planned_ability.clone();
    } else {
        planned.push(planned_ability.clone());
    }
}

fn apply_effect_timed(state: &mut State, id: Id, timed_effect: &effect::Timed) {
    trace!("effect::apply_timed: {:?}", timed_effect);
    let effects = &mut state.parts_mut().effects;
    if effects.get_opt(id).is_none() {
        effects.insert(id, component::Effects(Vec::new()));
    }
    let effects = &mut effects.get_mut(id).0;
    if let Some(i) = effects.iter().position(|e| e.effect == timed_effect.effect) {
        effects[i] = timed_effect.clone();
    } else {
        effects.push(timed_effect.clone());
    }
}

fn apply_effect_instant(state: &mut State, id: Id, effect: &Effect) {
    trace!("effect::apply_instant: {:?} ({})", effect, effect.to_str());
    match *effect {
        Effect::Create(ref effect) => apply_effect_create(state, id, effect),
        Effect::Kill(ref effect) => apply_effect_kill(state, id, effect),
        Effect::Vanish => apply_effect_vanish(state, id),
        Effect::Stun => apply_effect_stun(state, id),
        Effect::Heal(ref effect) => apply_effect_heal(state, id, effect),
        Effect::Wound(ref effect) => apply_effect_wound(state, id, effect),
        Effect::Knockback(ref effect) => apply_effect_knockback(state, id, effect),
        Effect::FlyOff(ref effect) => apply_effect_fly_off(state, id, effect),
        Effect::Throw(ref effect) => apply_effect_throw(state, id, effect),
        Effect::Dodge(_) => {}
        Effect::Bloodlust => apply_effect_bloodlust(state, id),
    }
}

fn apply_effect_create(state: &mut State, id: Id, effect: &effect::Create) {
    add_components(state, id, &effect.components);
}

fn apply_effect_kill(state: &mut State, id: Id, _: &effect::Kill) {
    let parts = state.parts_mut();
    parts.remove(id);
}

fn apply_effect_vanish(state: &mut State, id: Id) {
    let parts = state.parts_mut();
    parts.remove(id);
}

fn apply_effect_stun(state: &mut State, id: Id) {
    let parts = state.parts_mut();
    let agent = parts.agent.get_mut(id);
    agent.moves.0 = 0;
    agent.attacks.0 = 0;
    agent.jokers.0 = 0;
}

// TODO: split `Heal` effect into two? `Heal` + `RemoveLastingEffects`?
fn apply_effect_heal(state: &mut State, id: Id, effect: &effect::Heal) {
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

fn apply_effect_wound(state: &mut State, id: Id, effect: &effect::Wound) {
    let parts = state.parts_mut();
    let damage = effect.damage.0;
    assert!(damage >= 0);
    if effect.armor_break > Strength(0) {
        assert!(parts.armor.get_opt(id).is_some());
    }
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

fn apply_effect_knockback(state: &mut State, id: Id, effect: &effect::Knockback) {
    assert!(state.map().is_inboard(effect.from));
    if effect.to == effect.from {
        return;
    }
    assert!(state.map().is_inboard(effect.to));
    assert!(!state::is_tile_blocked(state, effect.to));
    let parts = state.parts_mut();
    parts.pos.get_mut(id).0 = effect.to;
}

fn apply_effect_fly_off(state: &mut State, id: Id, effect: &effect::FlyOff) {
    assert!(state.map().is_inboard(effect.from));
    if effect.to == effect.from {
        return;
    }
    assert!(state.map().is_inboard(effect.to));
    assert!(!state::is_tile_blocked(state, effect.to));
    let parts = state.parts_mut();
    parts.pos.get_mut(id).0 = effect.to;
}

fn apply_effect_throw(state: &mut State, id: Id, effect: &effect::Throw) {
    assert!(state.map().is_inboard(effect.from));
    assert!(state.map().is_inboard(effect.to));
    assert!(!state::is_tile_blocked(state, effect.to));
    let parts = state.parts_mut();
    parts.pos.get_mut(id).0 = effect.to;
}

fn apply_effect_bloodlust(state: &mut State, id: Id) {
    let parts = state.parts_mut();
    let agent = parts.agent.get_mut(id);
    agent.jokers.0 += 3;
}

fn update_cooldowns_for_object(state: &mut State, id: Id) {
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

fn apply_lasting_effects(state: &mut State) {
    for id in state::players_agent_ids(state, state.player_id()) {
        if state.parts().effects.get_opt(id).is_some() {
            let effects = state.parts().effects.get(id).clone();
            for effect in &effects.0 {
                apply_lasting_effect(state, id, &effect.effect);
            }
        }
    }
}

fn tick_planned_abilities(state: &mut State) {
    let phase = Phase::from_player_id(state.player_id());
    let ids = state.parts().schedule.ids_collected();
    for obj_id in ids {
        let schedule = state.parts_mut().schedule.get_mut(obj_id);
        for planned in &mut schedule.planned {
            if planned.phase == phase {
                planned.rounds.decrease();
            }
        }
    }
}
