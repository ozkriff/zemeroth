use std::collections::HashMap;

use rand::{thread_rng, Rng};

use core::ability::{self, Ability, PassiveAbility};
use core::apply::apply;
use core::check::{check, Error};
use core::command::{self, Command};
use core::component::{self, Component};
use core::effect::{self, Duration, Effect, LastingEffect, TimedEffect};
use core::event::{self, ActiveEvent, Event};
use core::map::{self, Dir, PosHex};
use core::movement::Path;
use core::state;
use core::{self, Moves, ObjId, Phase, PlayerId, State, TileType};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ApplyPhase {
    Pre,
    Post,
}

/// A callback for visualization of the events/effects with the correct state.
type Cb<'c> = &'c mut FnMut(&State, &Event, ApplyPhase);

pub fn execute(state: &mut State, command: &Command, cb: Cb) -> Result<(), Error> {
    debug!("Simulator: do_command: {:?}", command);
    if let Err(err) = check(state, command) {
        error!("Check failed: {:?}", err);
        return Err(err);
    }
    match *command {
        Command::Create(ref command) => execute_create(state, cb, command),
        Command::MoveTo(ref command) => execute_move_to(state, cb, command),
        Command::Attack(ref command) => execute_attack(state, cb, command),
        Command::EndTurn(ref command) => execute_end_turn(state, cb, command),
        Command::UseAbility(ref command) => execute_use_ability(state, cb, command),
    }
    Ok(())
}

fn do_event(state: &mut State, cb: Cb, event: &Event) {
    cb(state, event, ApplyPhase::Pre);
    apply(state, event);
    cb(state, event, ApplyPhase::Post);
}

fn execute_move_to(state: &mut State, cb: Cb, command: &command::MoveTo) {
    let mut cost = Some(Moves(1));
    let id = command.id;
    // prevent enemy from escaping
    let tie_up_attack_status = try_execute_reaction_attacks(state, cb, id);
    if tie_up_attack_status == AttackStatus::Hit && state.parts().agent.get_opt(id).is_some() {
        // A degenerate move event just to spend agent's move point
        let current_pos = state.parts().pos.get(id).0;
        let path = Path::new(vec![current_pos]);
        do_move(state, cb, id, cost.take(), path);
        return;
    }
    if state.parts().agent.get_opt(id).is_none() {
        return;
    }
    for step in command.path.steps() {
        assert!(state.parts().agent.get_opt(id).is_some());
        let path = Path::new(vec![step.from, step.to]);
        do_move(state, cb, id, cost.take(), path);
        try_execute_passive_abilities_on_move(state, cb, id);
        let attack_status = try_execute_reaction_attacks(state, cb, id);
        let is_alive = state.parts().agent.get_opt(id).is_some();
        if attack_status == AttackStatus::Hit || !is_alive {
            break;
        }
    }
}

fn do_move(state: &mut State, cb: Cb, id: ObjId, cost: Option<Moves>, path: Path) {
    let cost = cost.unwrap_or(Moves(0));
    let active_event = ActiveEvent::MoveTo(event::MoveTo { id, path, cost });
    let event = Event {
        active_event,
        actor_ids: vec![id],
        instant_effects: HashMap::new(),
        timed_effects: HashMap::new(),
    };
    do_event(state, cb, &event);
}

fn execute_create(state: &mut State, cb: Cb, command: &command::Create) {
    let mut components = state.prototype_for(&command.prototype);
    if let Some(player_id) = command.owner {
        components.push(Component::BelongsTo(component::BelongsTo(player_id)));
    }
    let name = command.prototype.clone();
    components.extend_from_slice(&[
        Component::Pos(component::Pos(command.pos)),
        Component::Meta(component::Meta { name }),
    ]);
    let id = state.alloc_id();

    let mut instant_effects = HashMap::new();
    let effect_create = Effect::Create(effect::Create {
        pos: command.pos,
        prototype: command.prototype.clone(),
        components,
    });
    instant_effects.insert(id, vec![effect_create]);

    let event = Event {
        active_event: ActiveEvent::Create,
        actor_ids: vec![id],
        instant_effects,
        timed_effects: HashMap::new(),
    };
    do_event(state, cb, &event);
}

#[derive(PartialEq, Clone, Debug)]
enum AttackStatus {
    Hit,
    Miss,
}

fn execute_attack_internal(
    state: &mut State,
    cb: Cb,
    command: &command::Attack,
    mode: event::AttackMode,
) -> AttackStatus {
    let active_event = ActiveEvent::Attack(event::Attack {
        attacker_id: command.attacker_id,
        target_id: command.target_id,
        mode,
    });
    let mut target_effects = Vec::new();
    // TODO: WE NEED SOME ACTUAL MATH HERE
    let mut is_kill = false;
    if thread_rng().gen_range(0, 6) < 4 {
        let damage = state.parts().agent.get(command.attacker_id).attack_strength;
        let effect = wound_or_kill(state, command.target_id, damage);
        if let Effect::Kill = effect {
            is_kill = true;
        }
        target_effects.push(effect);
    }
    let mut timed_effects = HashMap::new();
    let status = if target_effects.is_empty() {
        target_effects.push(Effect::Miss);
        AttackStatus::Miss
    } else {
        if !is_kill {
            let (mut effects_instant, effects_timed) = try_execute_passive_abilities_on_attack(
                state,
                command.attacker_id,
                command.target_id,
            );
            target_effects.append(&mut effects_instant);
            timed_effects.insert(command.target_id, effects_timed);
        }
        AttackStatus::Hit
    };
    let mut effects = HashMap::new();
    effects.insert(command.target_id, target_effects);
    let event = Event {
        active_event,
        actor_ids: vec![command.attacker_id],
        instant_effects: effects,
        timed_effects,
    };
    do_event(state, cb, &event);
    status
}

fn try_execute_passive_ability_burn(state: &mut State, target_id: ObjId) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let damage = core::Strength(1);
    let target_effects = vec![wound_or_kill(state, target_id, damage)];
    context.instant_effects.insert(target_id, target_effects);
    context
}

fn try_execute_passive_ability_spike_trap(state: &mut State, target_id: ObjId) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let damage = core::Strength(1);
    let target_effects = vec![wound_or_kill(state, target_id, damage)];
    context.instant_effects.insert(target_id, target_effects);
    context
}

fn try_execute_passive_ability_poison(state: &mut State, target_id: ObjId) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    if let Some(effects) = state.parts_mut().effects.get_opt_mut(target_id) {
        for effect in &mut effects.0 {
            if let LastingEffect::Poison = effect.effect {
                // this agent is already poisoned, so just resetting the duration
                effect.duration = Duration::Rounds(2);
                return context;
            }
        }
    }
    let owner = state.parts().belongs_to.get(target_id).0;
    let effect = TimedEffect {
        duration: effect::Duration::Rounds(2),
        phase: Phase::from_player_id(owner),
        effect: LastingEffect::Poison,
    };
    context.timed_effects.insert(target_id, vec![effect]);
    context
}

fn do_passive_ability(
    state: &mut State,
    cb: Cb,
    id: ObjId,
    target_pos: PosHex,
    ability: PassiveAbility,
    context: ExecuteContext,
) {
    let active_event = ActiveEvent::UsePassiveAbility(event::UsePassiveAbility {
        pos: target_pos,
        id,
        ability,
    });
    let event = Event {
        active_event,
        actor_ids: context.actor_ids,
        instant_effects: context.instant_effects,
        timed_effects: context.timed_effects,
    };
    do_event(state, cb, &event);
}

fn try_execute_passive_abilities_on_move(state: &mut State, cb: Cb, target_id: ObjId) {
    try_execute_passive_abilities_tick(state, cb, target_id)
}

fn try_execute_passive_abilities_tick(state: &mut State, cb: Cb, target_id: ObjId) {
    debug!("try_execute_passive_abilities_tick");
    if !state.parts().is_exist(target_id) {
        return;
    }
    let target_pos = state.parts().pos.get(target_id).0;
    let ids: Vec<_> = state.parts().passive_abilities.ids().collect();
    for id in ids {
        if !state.parts().is_exist(target_id) {
            continue;
        }
        if state.parts().agent.get_opt(target_id).is_none() {
            continue;
        }
        let abilities = state.parts().passive_abilities.get(id).clone();
        let pos = match state.parts().pos.get_opt(id) {
            Some(pos) => pos.0,
            None => continue,
        };
        if pos != target_pos {
            continue;
        }
        for &ability in &abilities.0 {
            assert!(state.parts().is_exist(target_id));
            match ability {
                PassiveAbility::SpikeTrap => {
                    let context = try_execute_passive_ability_spike_trap(state, target_id);
                    do_passive_ability(state, cb, id, target_pos, ability, context);
                }
                PassiveAbility::Burn => {
                    let context = try_execute_passive_ability_burn(state, target_id);
                    do_passive_ability(state, cb, id, target_pos, ability, context);
                }
                PassiveAbility::Poison => {
                    let context = try_execute_passive_ability_poison(state, target_id);
                    do_passive_ability(state, cb, id, target_pos, ability, context);
                }
                PassiveAbility::HeavyImpact
                | PassiveAbility::PoisonAttack
                | PassiveAbility::Regenerate(_)
                | PassiveAbility::SpawnPoisonCloudOnDeath => {}
            }
        }
    }
}

fn try_execute_passive_abilities_on_end_turn(state: &mut State, cb: Cb) {
    for id in state::players_agent_ids(state, state.player_id()) {
        try_execute_passive_abilities_tick(state, cb, id);
    }

    // TODO: extract to some self-abilities-method?
    {
        let ids: Vec<_> = state.parts().passive_abilities.ids().collect();
        for id in ids {
            assert!(state.parts().is_exist(id));
            let owner = match state.parts().belongs_to.get_opt(id) {
                Some(owner) => owner.0,
                None => continue,
            };
            if state.player_id() != owner {
                continue;
            }

            let abilities = state.parts().passive_abilities.get(id).clone();
            for &ability in &abilities.0 {
                assert!(state.parts().is_exist(id));
                if let PassiveAbility::Regenerate(regenerate) = ability {
                    if state.parts().strength.get(id).strength
                        >= state.parts().strength.get(id).base_strength
                    {
                        continue;
                    }
                    let active_event = ActiveEvent::UsePassiveAbility(event::UsePassiveAbility {
                        pos: state.parts().pos.get(id).0,
                        id,
                        ability,
                    });
                    let mut target_effects = Vec::new();
                    target_effects.push(Effect::Heal(effect::Heal {
                        strength: regenerate.0,
                    }));
                    let mut effects = HashMap::new();
                    effects.insert(id, target_effects);
                    let event = Event {
                        active_event,
                        actor_ids: vec![id],
                        instant_effects: effects,
                        timed_effects: HashMap::new(),
                    };
                    do_event(state, cb, &event);
                }
            }
        }
    }
}

fn try_execute_passive_abilities_on_attack(
    state: &mut State,
    attacker_id: ObjId,
    target_id: ObjId,
) -> (Vec<Effect>, Vec<TimedEffect>) {
    let mut instant_effects: Vec<Effect> = Vec::new();
    let mut timed_effects: Vec<TimedEffect> = Vec::new();
    let target_pos = state.parts().pos.get(target_id).0;
    let attacker_pos = state.parts().pos.get(attacker_id).0;
    if let Some(passive_abilities) = state.parts().passive_abilities.get_opt(attacker_id) {
        let abilities = passive_abilities.clone();
        for &ability in &abilities.0 {
            trace!("ability: {:?}", ability);
            match ability {
                PassiveAbility::HeavyImpact => {
                    let dir = Dir::get_dir_from_to(attacker_pos, target_pos);
                    let from = target_pos;
                    let to = Dir::get_neighbor_pos(target_pos, dir);
                    if state.map().is_inboard(to) && !state::is_tile_blocked(state, to) {
                        instant_effects.push(Effect::FlyOff(effect::FlyOff { from, to }));
                    }
                }
                PassiveAbility::PoisonAttack => {
                    let owner = state.parts().belongs_to.get(target_id).0;
                    let effect = TimedEffect {
                        duration: effect::Duration::Rounds(2),
                        phase: Phase::from_player_id(owner),
                        effect: LastingEffect::Poison,
                    };
                    timed_effects.push(effect);
                }
                PassiveAbility::Burn
                | PassiveAbility::SpikeTrap
                | PassiveAbility::Poison
                | PassiveAbility::Regenerate(_)
                | PassiveAbility::SpawnPoisonCloudOnDeath => (),
            }
        }
    }
    (instant_effects, timed_effects)
}

fn try_execute_reaction_attacks(state: &mut State, cb: Cb, target_id: ObjId) -> AttackStatus {
    let mut status = AttackStatus::Miss;
    let target_owner = match state.parts().belongs_to.get_opt(target_id) {
        Some(belongs_to) => belongs_to.0,
        None => return status,
    };
    let initial_player_id = state.player_id();
    for obj_id in state::enemy_agent_ids(state, initial_player_id) {
        if state.parts().agent.get_opt(obj_id).is_none() {
            // check if target is killed
            continue;
        }
        let this_unit_owner = state.parts().belongs_to.get(obj_id).0;
        if this_unit_owner == target_owner {
            continue;
        }
        let command_attack = command::Attack {
            attacker_id: obj_id,
            target_id,
        };
        let command = command::Command::Attack(command_attack.clone());
        state.set_player_id(this_unit_owner);
        if check(state, &command).is_err() {
            continue;
        }
        let mode = event::AttackMode::Reactive;
        let this_attack_status = execute_attack_internal(state, cb, &command_attack, mode);
        if this_attack_status != AttackStatus::Miss {
            status = this_attack_status;
        }
    }
    state.set_player_id(initial_player_id);
    status
}

fn execute_attack(state: &mut State, cb: Cb, command: &command::Attack) {
    execute_attack_internal(state, cb, command, event::AttackMode::Active);
    try_execute_reaction_attacks(state, cb, command.attacker_id);
}

fn execute_event_end_turn(state: &mut State, cb: Cb) {
    let player_id_old = state.player_id();
    let active_event = ActiveEvent::EndTurn(event::EndTurn {
        player_id: player_id_old,
    });
    let actor_ids = state::players_agent_ids(state, player_id_old);
    let event = Event {
        active_event,
        actor_ids,
        instant_effects: HashMap::new(),
        timed_effects: HashMap::new(),
    };
    do_event(state, cb, &event);
}

fn execute_event_begin_turn(state: &mut State, cb: Cb) {
    let player_id_new = state.next_player_id();
    let active_event = ActiveEvent::BeginTurn(event::BeginTurn {
        player_id: player_id_new,
    });
    let actor_ids = state::players_agent_ids(state, player_id_new);
    let event = Event {
        active_event,
        actor_ids,
        instant_effects: HashMap::new(),
        timed_effects: HashMap::new(),
    };
    do_event(state, cb, &event);
}

fn execute_planned_abilities(state: &mut State, cb: Cb) {
    let phase = Phase::from_player_id(state.player_id());
    let ids: Vec<_> = state.parts().schedule.ids().collect();
    for obj_id in ids {
        let pos = state.parts().pos.get(obj_id).0;
        let mut activated = Vec::new();
        {
            let schedule = state.parts_mut().schedule.get_mut(obj_id);
            for planned in &mut schedule.planned {
                if planned.phase != phase {
                    continue;
                }
                planned.rounds -= 1;
                if planned.rounds == 0 {
                    trace!("planned ability: ready!");
                    let c = command::UseAbility {
                        ability: planned.ability,
                        id: obj_id,
                        pos,
                    };
                    activated.push(c);
                }
            }
            schedule.planned.retain(|p| p.rounds > 0);
        }
        for command in activated {
            if state.parts().is_exist(obj_id) {
                execute_use_ability(state, cb, &command);
            }
        }
    }
}

// TODO: simplify
/// Ticks and kills all the lasting effects.
fn execute_effects(state: &mut State, cb: Cb) {
    let phase = Phase::from_player_id(state.player_id());
    let ids: Vec<_> = state.parts().effects.ids().collect();
    for obj_id in ids {
        for effect in &mut state.parts_mut().effects.get_mut(obj_id).0 {
            if effect.phase == phase {
                if let Duration::Rounds(ref mut n) = effect.duration {
                    *n -= 1;
                }
            }
        }

        for effect in &mut state.parts_mut().effects.get_mut(obj_id).0.clone() {
            if effect.phase != phase {
                continue;
            }
            assert!(state.parts().is_exist(obj_id));
            {
                let active_event = event::EffectTick {
                    id: obj_id,
                    effect: effect.effect.clone(),
                };
                let mut target_effects = Vec::new();
                match effect.effect {
                    LastingEffect::Poison => {
                        let damage = core::Strength(1);
                        target_effects.push(wound_or_kill(state, obj_id, damage));
                    }
                    LastingEffect::Stun => {
                        target_effects.push(Effect::Stun);
                    }
                }
                let mut instant_effects = HashMap::new();
                instant_effects.insert(obj_id, target_effects);
                let event = Event {
                    active_event: ActiveEvent::EffectTick(active_event),
                    actor_ids: vec![obj_id],
                    instant_effects,
                    timed_effects: HashMap::new(),
                };
                do_event(state, cb, &event);
            }
            if !state.parts().is_exist(obj_id) {
                break;
            }
            if effect.duration.is_over() {
                let active_event = event::EffectEnd {
                    id: obj_id,
                    effect: effect.effect.clone(),
                };
                let event = Event {
                    active_event: ActiveEvent::EffectEnd(active_event),
                    actor_ids: vec![obj_id],
                    instant_effects: HashMap::new(),
                    timed_effects: HashMap::new(),
                };
                do_event(state, cb, &event);
            }
        }

        if !state.parts().is_exist(obj_id) {
            continue;
        }

        let effects = state.parts_mut().effects.get_mut(obj_id);
        effects.0.retain(|effect| match effect.duration {
            effect::Duration::Rounds(n) => n > 0,
            _ => true,
        });
    }
}

fn execute_end_turn(state: &mut State, cb: Cb, _: &command::EndTurn) {
    execute_event_end_turn(state, cb);
    execute_event_begin_turn(state, cb);
    try_execute_passive_abilities_on_end_turn(state, cb);
    execute_planned_abilities(state, cb);
    execute_effects(state, cb);
}

fn schedule_ability(state: &mut State, id: ObjId, ability: Ability) {
    let phase = Phase::from_player_id(state.player_id());
    let schedule = {
        if state.parts().schedule.get_opt(id).is_none() {
            let component = component::Schedule {
                planned: Vec::new(),
            };
            state.parts_mut().schedule.insert(id, component);
        }
        state.parts_mut().schedule.get_mut(id)
    };
    let planned_ability = component::PlannedAbility {
        rounds: 2,
        phase,
        ability,
    };
    schedule.planned.push(planned_ability);
}

fn refresh_scheduled_ability(state: &mut State, id: ObjId, ability: Ability) {
    let schedule = state.parts_mut().schedule.get_mut(id);
    for planned_ability in &mut schedule.planned {
        if planned_ability.ability == ability {
            planned_ability.rounds = 2; // TODO: do not hardcode
            return;
        }
    }
    panic!("haven't found an object with this ability");
}

fn start_fire(state: &mut State, pos: PosHex) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    if let Some(id) = state::obj_with_passive_ability_at(state, pos, PassiveAbility::Burn) {
        refresh_scheduled_ability(state, id, Ability::Vanish);
    } else {
        let effect_create = effect_create_object(state, "fire", pos);
        let id = state.alloc_id();
        context.instant_effects.insert(id, vec![effect_create]);
        schedule_ability(state, id, Ability::Vanish);
        for target_id in state::agent_ids_at(state, pos) {
            context.merge_with(try_execute_passive_ability_burn(state, target_id));
        }
    }
    context
}

fn create_poison_cloud(state: &mut State, pos: PosHex) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    if let Some(id) = state::obj_with_passive_ability_at(state, pos, PassiveAbility::Poison) {
        refresh_scheduled_ability(state, id, Ability::Vanish);
    } else {
        let effect_create = effect_create_object(state, "poison_cloud", pos);
        let id = state.alloc_id();
        context.instant_effects.insert(id, vec![effect_create]);
        schedule_ability(state, id, Ability::Vanish);
        for target_id in state::agent_ids_at(state, pos) {
            context.merge_with(try_execute_passive_ability_poison(state, target_id));
        }
    }
    context
}

#[must_use]
#[derive(Default, Debug, PartialEq, Clone)]
struct ExecuteContext {
    actor_ids: Vec<ObjId>,
    moved_actor_ids: Vec<ObjId>,
    reaction_attack_targets: Vec<ObjId>,
    instant_effects: HashMap<ObjId, Vec<Effect>>,
    timed_effects: HashMap<ObjId, Vec<TimedEffect>>,
}

impl ExecuteContext {
    pub fn merge_with(&mut self, other: Self) {
        type M<T> = HashMap<ObjId, Vec<T>>;

        fn merge<T>(m: &mut M<T>, other: M<T>) {
            for (key, value) in other {
                m.entry(key).or_insert_with(Vec::new).extend(value);
            }
        }

        self.actor_ids.extend(other.actor_ids);
        self.moved_actor_ids.extend(other.moved_actor_ids);
        self.reaction_attack_targets
            .extend(other.reaction_attack_targets);
        merge(&mut self.instant_effects, other.instant_effects);
        merge(&mut self.timed_effects, other.timed_effects);
    }
}

fn execute_use_ability_knockback(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let id = state::blocker_id_at(state, command.pos);
    let from = command.pos;
    let actor_pos = state.parts().pos.get(command.id).0;
    let dir = Dir::get_dir_from_to(actor_pos, command.pos);
    let to = Dir::get_neighbor_pos(command.pos, dir);
    if state.map().is_inboard(to) && !state::is_tile_blocked(state, to) {
        let effect = Effect::Knockback(effect::Knockback { from, to });
        context.instant_effects.insert(id, vec![effect]);
        context.moved_actor_ids.push(id);
    }
    context.actor_ids.push(id);
    context
}

fn execute_use_ability_club(state: &mut State, command: &command::UseAbility) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let id = state::blocker_id_at(state, command.pos);
    let from = command.pos;
    let actor_pos = state.parts().pos.get(command.id).0;
    let dir = Dir::get_dir_from_to(actor_pos, command.pos);
    let to = Dir::get_neighbor_pos(command.pos, dir);
    if state.map().is_inboard(to) && !state::is_tile_blocked(state, to) {
        let effect = Effect::FlyOff(effect::FlyOff { from, to });
        context.instant_effects.insert(id, vec![effect]);
        context.moved_actor_ids.push(id);
    }
    if state.parts().belongs_to.get_opt(id).is_some() {
        let owner = state.parts().belongs_to.get(id).0;
        let phase = Phase::from_player_id(owner);
        let effect = TimedEffect {
            duration: effect::Duration::Rounds(2),
            phase,
            effect: LastingEffect::Stun,
        };
        context.timed_effects.insert(id, vec![effect]);
        let effects = context.instant_effects.entry(id).or_insert_with(Vec::new);
        effects.push(Effect::Stun);
    }
    context.actor_ids.push(id);
    context
}

fn execute_use_ability_explode_fire(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    assert!(context.instant_effects.get(&command.id).is_none());
    let effects = vec![Effect::Vanish];
    context.instant_effects.insert(command.id, effects);
    context.merge_with(start_fire(state, command.pos));
    for dir in map::dirs() {
        let pos = Dir::get_neighbor_pos(command.pos, dir);
        if state.map().is_inboard(pos) {
            context.merge_with(start_fire(state, pos));
        }
    }
    context
}

fn execute_use_ability_jump(_: &mut State, command: &command::UseAbility) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    context.moved_actor_ids.push(command.id);
    context
}

fn execute_use_ability_dash(_: &mut State, command: &command::UseAbility) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    context.moved_actor_ids.push(command.id);
    context
}

fn execute_use_ability_rage(_: &mut State, _: &command::UseAbility) -> ExecuteContext {
    ExecuteContext::default()
}

fn execute_use_ability_heal(
    state: &mut State,
    command: &command::UseAbility,
    ability: ability::Heal,
) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let id = state::blocker_id_at(state, command.pos);
    let effect = Effect::Heal(effect::Heal {
        strength: ability.0,
    });
    context.instant_effects.insert(id, vec![effect]);
    context
}

fn execute_use_ability_vanish(state: &mut State, command: &command::UseAbility) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    assert!(state.parts().is_exist(command.id));
    let effects = vec![Effect::Vanish];
    context.instant_effects.insert(command.id, effects);
    context
}

fn execute_use_ability_explode_poison(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    assert!(context.instant_effects.get(&command.id).is_none());
    let effects = vec![Effect::Vanish];
    context.instant_effects.insert(command.id, effects);
    context.merge_with(create_poison_cloud(state, command.pos));
    for dir in map::dirs() {
        let pos = Dir::get_neighbor_pos(command.pos, dir);
        if state.map().is_inboard(pos) {
            context.merge_with(create_poison_cloud(state, pos));
        }
    }
    context
}

fn wound_or_kill(state: &State, id: ObjId, damage: core::Strength) -> Effect {
    let strength = state.parts().strength.get(id);
    if strength.strength > damage {
        Effect::Wound(effect::Wound { damage })
    } else {
        Effect::Kill
    }
}

fn execute_use_ability_explode(state: &mut State, command: &command::UseAbility) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let from = state.parts().pos.get(command.id).0;
    for id in state.parts().agent.ids() {
        let pos = state.parts().pos.get(id).0;
        let distance = map::distance_hex(from, pos);
        if distance.0 > 1 || command.id == id {
            continue;
        }
        let dir = Dir::get_dir_from_to(from, pos);
        let to = Dir::get_neighbor_pos(pos, dir);
        let mut effects = Vec::new();
        if state.map().is_inboard(to) && !state::is_tile_blocked(state, to) {
            effects.push(Effect::Knockback(effect::Knockback { from: pos, to }));
            context.moved_actor_ids.push(id);
        }
        effects.push(wound_or_kill(state, id, core::Strength(1)));
        context.instant_effects.insert(id, effects);
    }
    assert!(context.instant_effects.get(&command.id).is_none());
    let effects = vec![Effect::Vanish];
    context.instant_effects.insert(command.id, effects);
    context
}

fn execute_use_ability_poison(state: &mut State, command: &command::UseAbility) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let id = state::blocker_id_at(state, command.pos);
    let owner = state.parts().belongs_to.get(id).0;
    let phase = Phase::from_player_id(owner);
    let effect = TimedEffect {
        duration: effect::Duration::Rounds(2),
        phase,
        effect: LastingEffect::Poison,
    };
    context.timed_effects.insert(id, vec![effect]);
    context.actor_ids.push(id);
    context
}

fn effect_create_object(state: &State, prototype: &str, pos: PosHex) -> Effect {
    let name = prototype.into();
    let mut components = state.prototype_for(prototype);
    components.extend_from_slice(&[
        Component::Pos(component::Pos(pos)),
        Component::Meta(component::Meta { name }),
    ]);
    Effect::Create(effect::Create {
        pos,
        prototype: prototype.into(),
        components,
    })
}

fn effect_create_agent(state: &State, prototype: &str, player_id: PlayerId, pos: PosHex) -> Effect {
    let name = prototype.into();
    let mut components = state.prototype_for(prototype);
    components.extend_from_slice(&[
        Component::Pos(component::Pos(pos)),
        Component::Meta(component::Meta { name }),
        Component::BelongsTo(component::BelongsTo(player_id)),
    ]);
    Effect::Create(effect::Create {
        pos,
        prototype: prototype.into(),
        components,
    })
}

fn throw_bomb(
    state: &mut State,
    command: &command::UseAbility,
    prototype: &str,
    ability: Ability,
) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let pos = state.parts().pos.get(command.id).0;
    let effect_create = effect_create_object(state, prototype, pos);
    let id = state.alloc_id();
    let effect_throw = Effect::Throw(effect::Throw {
        from: pos,
        to: command.pos,
    });
    let effects = vec![effect_create, effect_throw];
    context.instant_effects.insert(id, effects);
    {
        let phase = Phase::from_player_id(state.player_id());
        if state.parts().schedule.get_opt(id).is_none() {
            let component = component::Schedule {
                planned: Vec::new(),
            };
            state.parts_mut().schedule.insert(id, component);
        }
        let schedule = state.parts_mut().schedule.get_mut(id);
        let planned_ability = component::PlannedAbility {
            rounds: 1, // on next turn
            phase,
            ability,
        };
        schedule.planned.push(planned_ability);
    }
    context
}

fn execute_use_ability_bomb(state: &mut State, command: &command::UseAbility) -> ExecuteContext {
    throw_bomb(state, command, "bomb", Ability::Explode)
}

fn execute_use_ability_bomb_fire(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    throw_bomb(state, command, "bomb_fire", Ability::ExplodeFire)
}

fn execute_use_ability_bomb_poison(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    throw_bomb(state, command, "bomb_poison", Ability::ExplodePoison)
}

fn execute_use_ability_summon(
    state: &mut State,
    command: &command::UseAbility,
    ability: ability::Summon,
) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let max_summoned_count = ability.0;
    for pos in state::free_neighbor_positions(state, command.pos, max_summoned_count) {
        let prototypes = ["imp", "imp_toxic", "imp_bomber"];
        let prototype = thread_rng().choose(&prototypes).unwrap();
        let create = effect_create_agent(state, prototype, state.player_id(), pos);
        let id = state.alloc_id();
        let effects = vec![create, Effect::Stun];
        context.instant_effects.insert(id, effects);
        context.moved_actor_ids.push(id);
        context.reaction_attack_targets.push(id);
    }
    context
}

fn execute_use_ability(state: &mut State, cb: Cb, command: &command::UseAbility) {
    let mut context = match command.ability {
        Ability::Knockback => execute_use_ability_knockback(state, command),
        Ability::Club => execute_use_ability_club(state, command),
        Ability::Jump(_) => execute_use_ability_jump(state, command),
        Ability::Dash => execute_use_ability_dash(state, command),
        Ability::Rage(_) => execute_use_ability_rage(state, command),
        Ability::Heal(a) => execute_use_ability_heal(state, command, a),
        Ability::Vanish => execute_use_ability_vanish(state, command),
        Ability::ExplodeFire => execute_use_ability_explode_fire(state, command),
        Ability::ExplodePoison => execute_use_ability_explode_poison(state, command),
        Ability::Explode => execute_use_ability_explode(state, command),
        Ability::Poison => execute_use_ability_poison(state, command),
        Ability::Bomb(_) => execute_use_ability_bomb(state, command),
        Ability::BombFire(_) => execute_use_ability_bomb_fire(state, command),
        Ability::BombPoison(_) => execute_use_ability_bomb_poison(state, command),
        Ability::Summon(a) => execute_use_ability_summon(state, command, a),
    };
    context.actor_ids.push(command.id);
    let active_event = ActiveEvent::UseAbility(event::UseAbility {
        id: command.id,
        pos: command.pos,
        ability: command.ability,
    });
    let event = Event {
        active_event,
        actor_ids: context.actor_ids,
        instant_effects: context.instant_effects,
        timed_effects: context.timed_effects,
    };
    do_event(state, cb, &event);
    for id in context.moved_actor_ids {
        try_execute_passive_abilities_on_move(state, cb, id);
    }
    for id in context.reaction_attack_targets {
        try_execute_reaction_attacks(state, cb, id);
    }
    if command.ability != Ability::Dash {
        try_execute_reaction_attacks(state, cb, command.id);
    }
}

fn random_free_pos(state: &State) -> Option<PosHex> {
    let attempts = 30;
    let radius = state.map().radius();
    for _ in 0..attempts {
        let pos = PosHex {
            q: thread_rng().gen_range(-radius.0, radius.0),
            r: thread_rng().gen_range(-radius.0, radius.0),
        };
        if state::is_tile_plain_and_completely_free(state, pos) {
            return Some(pos);
        }
    }
    None
}

fn random_free_sector_pos(state: &State, player_id: PlayerId) -> Option<PosHex> {
    let attempts = 30;
    let radius = state.map().radius();
    let start_sector_width = radius.0 + 1;
    for _ in 0..attempts {
        let q = radius.0 - thread_rng().gen_range(0, start_sector_width);
        let pos = PosHex {
            q: match player_id.0 {
                0 => -q,
                1 => q,
                _ => unimplemented!(),
            },
            r: thread_rng().gen_range(-radius.0, radius.0),
        };
        if state::is_tile_plain_and_completely_free(state, pos) {
            return Some(pos);
        }
    }
    None
}

pub fn create_terrain(state: &mut State) {
    for _ in 0..15 {
        let pos = match random_free_pos(state) {
            Some(pos) => pos,
            None => continue,
        };
        state.map_mut().set_tile(pos, TileType::Rocks);
    }
}

pub fn create_objects(state: &mut State, cb: Cb) {
    let player_id_initial = state.player_id();
    for &(owner, typename, count) in &[
        (None, "spike_trap", 3),
        (None, "boulder", 7),
        (Some(PlayerId(0)), "swordsman", 1),
        (Some(PlayerId(0)), "hammerman", 1),
        (Some(PlayerId(0)), "spearman", 1),
        (Some(PlayerId(0)), "alchemist", 1),
        (Some(PlayerId(1)), "imp", 3),
        (Some(PlayerId(1)), "imp_toxic", 1),
        (Some(PlayerId(1)), "imp_bomber", 2),
        (Some(PlayerId(1)), "imp_summoner", 2),
    ] {
        if let Some(player_id) = owner {
            state.set_player_id(player_id);
        }
        for _ in 0..count {
            // TODO: different radiouses - put summoner in a good distance
            let pos = match owner {
                Some(player_id) => random_free_sector_pos(state, player_id),
                None => random_free_pos(state),
            }.unwrap();
            let command = Command::Create(command::Create {
                prototype: typename.into(),
                pos,
                owner,
            });
            execute(state, &command, cb).expect("Can't create object");
        }
    }
    state.set_player_id(player_id_initial);
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use core::effect::Effect;
    use core::ObjId;

    use super::ExecuteContext;

    #[test]
    fn test_merge_with_vector() {
        let mut context1 = ExecuteContext {
            actor_ids: vec![ObjId(0), ObjId(1)],
            ..Default::default()
        };
        let context2 = ExecuteContext {
            actor_ids: vec![ObjId(2), ObjId(3)],
            ..Default::default()
        };
        let context_expected = ExecuteContext {
            actor_ids: vec![ObjId(0), ObjId(1), ObjId(2), ObjId(3)],
            ..Default::default()
        };
        context1.merge_with(context2);
        assert_eq!(context_expected, context1);
    }

    #[test]
    fn test_merge_with_hashmap() {
        let mut instant_effects1 = HashMap::new();
        instant_effects1.insert(ObjId(0), vec![Effect::Kill, Effect::Stun]);
        let mut context1 = ExecuteContext {
            instant_effects: instant_effects1,
            ..Default::default()
        };
        let mut instant_effects2 = HashMap::new();
        instant_effects2.insert(ObjId(0), vec![Effect::Vanish, Effect::Miss]);
        let context2 = ExecuteContext {
            instant_effects: instant_effects2,
            ..Default::default()
        };
        let mut instant_effects_expected = HashMap::new();
        instant_effects_expected.insert(
            ObjId(0),
            vec![Effect::Kill, Effect::Stun, Effect::Vanish, Effect::Miss],
        );
        let context_expected = ExecuteContext {
            instant_effects: instant_effects_expected,
            ..Default::default()
        };
        context1.merge_with(context2);
        assert_eq!(context_expected, context1);
    }
}
