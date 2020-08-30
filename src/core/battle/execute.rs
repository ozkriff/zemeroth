use std::collections::HashMap;

use log::{error, trace};

use crate::core::{
    battle::{
        self,
        ability::{self, Ability, PassiveAbility},
        check::{check, Error},
        command::{self, Command},
        component::{self, ObjType},
        effect::{self, Effect},
        event::{self, ActiveEvent, Event},
        movement::Path,
        state::{self, BattleResult, State},
        Id, Moves, Phase, PlayerId, PushStrength, Strength, Weight,
    },
    map::{self, Dir, PosHex},
    utils::{self, roll_dice},
};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ApplyPhase {
    Pre,
    Post,
}

/// A callback for visualization of the events/effects with the correct state.
pub type Cb<'c> = &'c mut dyn FnMut(&State, &Event, ApplyPhase);

pub fn execute(state: &mut State, command: &Command, cb: Cb) -> Result<(), Error> {
    trace!("Simulator: do_command: {:?}", command);
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
    execute_planned_abilities(state, cb);
    match *command {
        Command::Create(_) => {}
        _ => try_execute_end_battle(state, cb),
    }
    Ok(())
}

fn do_event(state: &mut State, cb: Cb, event: &Event) {
    cb(state, event, ApplyPhase::Pre);
    state.apply(event);
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

fn do_move(state: &mut State, cb: Cb, id: Id, cost: Option<Moves>, path: Path) {
    let cost = cost.unwrap_or(Moves(0));
    let active_event = event::MoveTo { id, path, cost }.into();
    let event = Event {
        active_event,
        actor_ids: vec![id],
        instant_effects: Vec::new(),
        timed_effects: Vec::new(),
        scheduled_abilities: Vec::new(),
    };
    do_event(state, cb, &event);
}

fn execute_create(state: &mut State, cb: Cb, command: &command::Create) {
    let mut components = state.prototype_for(&command.prototype);
    if let Some(player_id) = command.owner {
        components.push(component::BelongsTo(player_id).into());
    }
    let name = command.prototype.clone();
    components.extend_from_slice(&[
        component::Pos(command.pos).into(),
        component::Meta { name }.into(),
    ]);
    let id = state.alloc_id();

    let mut instant_effects = Vec::new();
    let effect_create = effect::Create {
        pos: command.pos,
        prototype: command.prototype.clone(),
        components,
        is_teleported: false,
    }
    .into();
    instant_effects.push((id, vec![effect_create]));

    let event = Event {
        active_event: ActiveEvent::Create,
        actor_ids: vec![id],
        instant_effects,
        timed_effects: Vec::new(),
        scheduled_abilities: Vec::new(),
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
    let weapon_type = state.parts().agent.get(command.attacker_id).weapon_type;
    let active_event = event::Attack {
        attacker_id: command.attacker_id,
        target_id: command.target_id,
        mode,
        weapon_type,
    }
    .into();
    let mut target_effects = Vec::new();
    let mut is_kill = false;
    if let Some(effect) = try_attack(state, command.attacker_id, command.target_id) {
        if let Effect::Kill(_) = effect {
            is_kill = true;
        }
        target_effects.push(effect);
    }
    let mut timed_effects = Vec::new();
    let status = if target_effects.is_empty() {
        let attacker_pos = state.parts().pos.get(command.attacker_id).0;
        target_effects.push(effect::Dodge { attacker_pos }.into());
        AttackStatus::Miss
    } else {
        if !is_kill {
            let mut effects = try_execute_passive_abilities_on_attack(
                state,
                command.attacker_id,
                command.target_id,
            );
            target_effects.append(&mut effects.instant);
            if !effects.timed.is_empty() {
                timed_effects.push((command.target_id, effects.timed));
            }
        }
        AttackStatus::Hit
    };
    let mut effects = Vec::new();
    effects.push((command.target_id, target_effects));
    let event = Event {
        active_event,
        actor_ids: vec![command.attacker_id],
        instant_effects: effects,
        timed_effects,
        scheduled_abilities: Vec::new(),
    };
    do_event(state, cb, &event);
    status
}

fn try_execute_passive_ability_burn(state: &mut State, target_id: Id) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let damage = battle::Strength(1);
    let target_effects = vec![wound_or_kill(state, target_id, damage)];
    context.instant_effects.push((target_id, target_effects));
    context
}

fn try_execute_passive_ability_spike_trap(state: &mut State, target_id: Id) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let damage = battle::Strength(1);
    let target_effects = vec![wound_or_kill(state, target_id, damage)];
    context.instant_effects.push((target_id, target_effects));
    context
}

fn try_execute_passive_ability_poison(state: &State, target_id: Id) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    if state.parts().strength.get(target_id).strength <= Strength(1) {
        return context;
    }
    let owner = state.parts().belongs_to.get(target_id).0;
    let effect = effect::Timed {
        duration: effect::Duration::Rounds(2),
        phase: Phase::from_player_id(owner),
        effect: effect::Lasting::Poison,
    };
    context.timed_effects.push((target_id, vec![effect]));
    context
}

fn do_passive_ability(
    state: &mut State,
    cb: Cb,
    id: Id,
    target_pos: PosHex,
    ability: PassiveAbility,
    context: ExecuteContext,
) {
    assert!(
        !context.instant_effects.is_empty()
            || !context.timed_effects.is_empty()
            || !context.scheduled_abilities.is_empty()
    );
    let active_event = event::UsePassiveAbility {
        pos: target_pos,
        id,
        ability,
    }
    .into();
    let event = Event {
        active_event,
        actor_ids: context.actor_ids,
        instant_effects: context.instant_effects,
        timed_effects: context.timed_effects,
        scheduled_abilities: Vec::new(),
    };
    do_event(state, cb, &event);
}

fn try_execute_passive_abilities_on_move(state: &mut State, cb: Cb, target_id: Id) {
    try_execute_passive_abilities_tick(state, cb, target_id)
}

fn try_execute_passive_abilities_tick(state: &mut State, cb: Cb, target_id: Id) {
    trace!("try_execute_passive_abilities_tick");
    if !state.parts().is_exist(target_id) {
        return;
    }
    let target_pos = state.parts().pos.get(target_id).0;
    let ids = state.parts().passive_abilities.ids_collected();
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
                    if !context.timed_effects.is_empty() {
                        do_passive_ability(state, cb, id, target_pos, ability, context);
                    }
                }
                PassiveAbility::HeavyImpact
                | PassiveAbility::PoisonAttack
                | PassiveAbility::Regenerate
                | PassiveAbility::SpawnPoisonCloudOnDeath => {}
            }
        }
    }
}

fn try_execute_passive_abilities_on_begin_turn(state: &mut State, cb: Cb) {
    for id in state::players_agent_ids(state, state.player_id()) {
        try_execute_passive_abilities_tick(state, cb, id);
    }

    // TODO: extract to some self-abilities-method?
    {
        let ids = state.parts().passive_abilities.ids_collected();
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
                if let PassiveAbility::Regenerate = ability {
                    if state.parts().strength.get(id).strength
                        >= state.parts().strength.get(id).base_strength
                    {
                        continue;
                    }
                    let pos = state.parts().pos.get(id).0;
                    let active_event = event::UsePassiveAbility { pos, id, ability }.into();
                    let mut target_effects = Vec::new();
                    let strength = Strength(1);
                    target_effects.push(effect::Heal { strength }.into());
                    let instant_effects = vec![(id, target_effects)];
                    let event = Event {
                        active_event,
                        actor_ids: vec![id],
                        instant_effects,
                        timed_effects: Vec::new(),
                        scheduled_abilities: Vec::new(),
                    };
                    do_event(state, cb, &event);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
struct Effects {
    instant: Vec<Effect>,
    timed: Vec<effect::Timed>,
}

fn try_execute_passive_abilities_on_attack(
    state: &mut State,
    attacker_id: Id,
    target_id: Id,
) -> Effects {
    let mut effects = Effects::default();
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
                    let strength = PushStrength(Weight::Normal);
                    let blocker_weight = state.parts().blocker.get(target_id).weight;
                    let to = if strength.can_push(blocker_weight) {
                        Dir::get_neighbor_pos(target_pos, dir)
                    } else {
                        from
                    };
                    let is_inboard = state.map().is_inboard(to);
                    if to == from || is_inboard && !state::is_tile_blocked(state, to) {
                        let effect = effect::FlyOff { from, to, strength }.into();
                        effects.instant.push(effect);
                    }
                }
                PassiveAbility::PoisonAttack => {
                    let owner = state.parts().belongs_to.get(target_id).0;
                    let effect = effect::Timed {
                        duration: effect::Duration::Rounds(2),
                        phase: Phase::from_player_id(owner),
                        effect: effect::Lasting::Poison,
                    };
                    effects.timed.push(effect);
                }
                PassiveAbility::Burn
                | PassiveAbility::SpikeTrap
                | PassiveAbility::Poison
                | PassiveAbility::Regenerate
                | PassiveAbility::SpawnPoisonCloudOnDeath => (),
            }
        }
    }
    effects
}

fn try_execute_reaction_attacks(state: &mut State, cb: Cb, target_id: Id) -> AttackStatus {
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
        let this_agent_owner = state.parts().belongs_to.get(obj_id).0;
        if this_agent_owner == target_owner {
            continue;
        }
        let command_attack = command::Attack {
            attacker_id: obj_id,
            target_id,
        };
        let command = command_attack.clone().into();
        state.set_player_id(this_agent_owner);
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
    let active_event = event::EndTurn {
        player_id: player_id_old,
    }
    .into();
    let actor_ids = state::players_agent_ids(state, player_id_old);
    let event = Event {
        active_event,
        actor_ids,
        instant_effects: Vec::new(),
        timed_effects: Vec::new(),
        scheduled_abilities: Vec::new(),
    };
    do_event(state, cb, &event);
}

fn execute_event_begin_turn(state: &mut State, cb: Cb) {
    let player_id_new = state.next_player_id();
    let active_event = event::BeginTurn {
        player_id: player_id_new,
    }
    .into();
    let actor_ids = state::players_agent_ids(state, player_id_new);
    let event = Event {
        active_event,
        actor_ids,
        instant_effects: Vec::new(),
        timed_effects: Vec::new(),
        scheduled_abilities: Vec::new(),
    };
    do_event(state, cb, &event);
}

fn execute_planned_abilities(state: &mut State, cb: Cb) {
    let mut ids = state.parts().schedule.ids_collected();
    ids.sort();
    for obj_id in ids {
        let pos = state.parts().pos.get(obj_id).0;
        let mut activated = Vec::new();
        {
            let schedule = state.parts().schedule.get(obj_id);
            for planned in &schedule.planned {
                if planned.rounds <= 0 {
                    trace!("planned ability: ready!");
                    let c = command::UseAbility {
                        ability: planned.ability.clone(),
                        id: obj_id,
                        pos,
                    };
                    activated.push(c);
                }
            }
        }
        for command in activated {
            if state.parts().is_exist(obj_id) {
                execute_use_ability(state, cb, &command);
            }
        }
    }
}

fn try_execute_end_battle(state: &mut State, cb: Cb) {
    for i in 0..state.scenario().players_count {
        let player_id = PlayerId(i);
        let enemies_count = state::enemy_agent_ids(state, player_id).len();
        if enemies_count == 0 {
            let result = BattleResult {
                winner_id: player_id,
                survivor_types: state::players_agent_types(state, PlayerId(0)),
            };
            let event = Event {
                active_event: event::EndBattle { result }.into(),
                actor_ids: Vec::new(),
                instant_effects: Vec::new(),
                timed_effects: Vec::new(),
                scheduled_abilities: Vec::new(),
            };
            do_event(state, cb, &event);
        }
    }
}

// TODO: simplify
/// Ticks and kills all the lasting effects.
fn execute_effects(state: &mut State, cb: Cb) {
    let phase = Phase::from_player_id(state.player_id());
    for id in state.parts().effects.ids_collected() {
        for effect in &state.parts().effects.get(id).0.clone() {
            if effect.phase != phase {
                continue;
            }
            assert!(state.parts().is_exist(id));
            {
                let active_event = event::EffectTick {
                    id,
                    effect: effect.effect.clone(),
                };
                let mut target_effects = Vec::new();
                match effect.effect {
                    effect::Lasting::Poison => {
                        let strength = state.parts().strength.get(id).strength;
                        if strength > battle::Strength(1) {
                            let damage = battle::Strength(1);
                            target_effects.push(wound_or_kill(state, id, damage));
                        }
                    }
                    effect::Lasting::Stun => {
                        target_effects.push(Effect::Stun);
                    }
                    effect::Lasting::Bloodlust => target_effects.push(Effect::Bloodlust),
                }
                let instant_effects = vec![(id, target_effects)];
                let event = Event {
                    active_event: ActiveEvent::EffectTick(active_event),
                    actor_ids: vec![id],
                    instant_effects,
                    timed_effects: Vec::new(),
                    scheduled_abilities: Vec::new(),
                };
                do_event(state, cb, &event);
            }
            if !state.parts().is_exist(id) {
                break;
            }
            if state::is_lasting_effect_over(state, id, effect) {
                let active_event = event::EffectEnd {
                    id,
                    effect: effect.effect.clone(),
                };
                let event = Event {
                    active_event: ActiveEvent::EffectEnd(active_event),
                    actor_ids: vec![id],
                    instant_effects: Vec::new(),
                    timed_effects: Vec::new(),
                    scheduled_abilities: Vec::new(),
                };
                do_event(state, cb, &event);
            }
        }

        if !state.parts().is_exist(id) {
            continue;
        }
    }
}

fn execute_end_turn(state: &mut State, cb: Cb, _: &command::EndTurn) {
    execute_event_end_turn(state, cb);
    execute_event_begin_turn(state, cb);
    try_execute_passive_abilities_on_begin_turn(state, cb);
    execute_effects(state, cb);
}

fn start_fire(state: &mut State, pos: PosHex) -> ExecuteContext {
    let vanish = component::PlannedAbility {
        rounds: 2, // TODO: Replace this magic number
        phase: Phase::from_player_id(state.player_id()),
        ability: Ability::Vanish,
    };
    let mut context = ExecuteContext::default();
    if let Some(id) = state::obj_with_passive_ability_at(state, pos, PassiveAbility::Burn) {
        context.scheduled_abilities.push((id, vec![vanish]));
    } else {
        let effect_create = effect_create_object(state, &"fire".into(), pos);
        let id = state.alloc_id();
        context.instant_effects.push((id, vec![effect_create]));
        context.scheduled_abilities.push((id, vec![vanish]));
        for target_id in state::agent_ids_at(state, pos) {
            context.merge_with(try_execute_passive_ability_burn(state, target_id));
        }
    }
    context
}

fn create_poison_cloud(state: &mut State, pos: PosHex) -> ExecuteContext {
    let vanish = component::PlannedAbility {
        rounds: 2, // TODO: Replace this magic number
        phase: Phase::from_player_id(state.player_id()),
        ability: Ability::Vanish,
    };
    let mut context = ExecuteContext::default();
    if let Some(id) = state::obj_with_passive_ability_at(state, pos, PassiveAbility::Poison) {
        context.scheduled_abilities.push((id, vec![vanish]));
    } else {
        let effect_create = effect_create_object(state, &"poison_cloud".into(), pos);
        let id = state.alloc_id();
        context.instant_effects.push((id, vec![effect_create]));
        context.scheduled_abilities.push((id, vec![vanish]));
        for target_id in state::agent_ids_at(state, pos) {
            context.merge_with(try_execute_passive_ability_poison(state, target_id));
        }
    }
    context
}

fn extend_or_crate_sub_vec<T>(vec: &mut Vec<(Id, Vec<T>)>, id: Id, values: Vec<T>) {
    if let Some(i) = vec.iter().position(|(this_id, _)| this_id == &id) {
        vec[i].1.extend(values);
    } else {
        vec.push((id, values));
    }
}

#[must_use]
#[derive(Default, Debug, PartialEq, Clone)]
struct ExecuteContext {
    actor_ids: Vec<Id>,
    moved_actor_ids: Vec<Id>,
    reaction_attack_targets: Vec<Id>,
    instant_effects: Vec<(Id, Vec<Effect>)>,
    timed_effects: Vec<(Id, Vec<effect::Timed>)>,
    scheduled_abilities: Vec<(Id, Vec<component::PlannedAbility>)>,
}

impl ExecuteContext {
    fn merge_with(&mut self, other: Self) {
        type M<T> = Vec<(Id, Vec<T>)>;

        fn merge<T>(m: &mut M<T>, other: M<T>) {
            for (id, values) in other {
                extend_or_crate_sub_vec(m, id, values);
            }
        }

        self.actor_ids.extend(other.actor_ids);
        self.moved_actor_ids.extend(other.moved_actor_ids);
        self.reaction_attack_targets
            .extend(other.reaction_attack_targets);
        merge(&mut self.instant_effects, other.instant_effects);
        merge(&mut self.timed_effects, other.timed_effects);
        merge(&mut self.scheduled_abilities, other.scheduled_abilities);
    }
}

fn execute_use_ability_knockback(
    state: &mut State,
    command: &command::UseAbility,
    ability: ability::Knockback,
) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let id = state::blocker_id_at(state, command.pos);
    let from = command.pos;
    let strength = ability.strength;
    let actor_pos = state.parts().pos.get(command.id).0;
    let dir = Dir::get_dir_from_to(actor_pos, command.pos);
    let blocker_weight = state.parts().blocker.get(id).weight;
    let to = if strength.can_push(blocker_weight) {
        Dir::get_neighbor_pos(command.pos, dir)
    } else {
        from
    };
    if to == from || state.map().is_inboard(to) && !state::is_tile_blocked(state, to) {
        let effect = effect::Knockback { from, to, strength }.into();
        context.instant_effects.push((id, vec![effect]));
        context.moved_actor_ids.push(id);
    }
    context.actor_ids.push(id);
    context
}

fn execute_use_ability_club(state: &mut State, command: &command::UseAbility) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let id = state::blocker_id_at(state, command.pos);
    if state.parts().belongs_to.get_opt(id).is_some() {
        let owner = state.parts().belongs_to.get(id).0;
        let phase = Phase::from_player_id(owner);
        let effect = effect::Timed {
            duration: effect::Duration::Rounds(1),
            phase,
            effect: effect::Lasting::Stun,
        };
        context.timed_effects.push((id, vec![effect]));
        extend_or_crate_sub_vec(&mut context.instant_effects, id, vec![Effect::Stun]);
    }
    context.actor_ids.push(id);
    context
}

fn execute_use_ability_explode_fire(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    assert!(context
        .instant_effects
        .iter()
        .position(|(id, _)| id == &command.id)
        .is_none());
    let effects = vec![Effect::Vanish];
    context.instant_effects.push((command.id, effects));
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
    let effect = effect::Heal {
        strength: ability.0,
    }
    .into();
    context.instant_effects.push((id, vec![effect]));
    context
}

fn execute_use_ability_vanish(state: &mut State, command: &command::UseAbility) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    assert!(state.parts().is_exist(command.id));
    let effects = vec![Effect::Vanish];
    context.instant_effects.push((command.id, effects));
    context
}

fn execute_use_ability_explode_poison(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    assert!(context
        .instant_effects
        .iter()
        .position(|(id, _)| id == &command.id)
        .is_none());
    let effects = vec![Effect::Vanish];
    context.instant_effects.push((command.id, effects));
    context.merge_with(create_poison_cloud(state, command.pos));
    for dir in map::dirs() {
        let pos = Dir::get_neighbor_pos(command.pos, dir);
        if state.map().is_inboard(pos) {
            context.merge_with(create_poison_cloud(state, pos));
        }
    }
    context
}

fn correct_damage_with_armor(
    state: &State,
    target_id: Id,
    damage: battle::Strength,
) -> battle::Strength {
    let id = target_id;
    let armor = state::get_armor(state, id);
    battle::Strength(utils::clamp_min(damage.0 - armor.0, 0))
}

fn wound_or_kill(state: &State, id: Id, damage: battle::Strength) -> Effect {
    let armor_break = battle::Strength(0);
    wound_break_kill(state, id, damage, armor_break)
}

fn wound_break_kill(
    state: &State,
    id: Id,
    damage: battle::Strength,
    armor_break: battle::Strength,
) -> Effect {
    let parts = state.parts();
    let strength = parts.strength.get(id).strength;
    let attacker_pos = None; // Let's assume that this is not a directed attack.
    if strength > damage {
        effect::Wound {
            damage,
            armor_break,
            attacker_pos,
        }
        .into()
    } else {
        effect::Kill { attacker_pos }.into()
    }
}

// TODO: Return a `Result` or an `Option` (check that attack is possible at all?).
// TODO: Return a struct with named fields.
// TODO: Move to some other module.
pub fn hit_chance(state: &State, attacker_id: Id, target_id: Id) -> (i32, i32) {
    let parts = state.parts();
    let agent_target = parts.agent.get(target_id);
    let agent_attacker = parts.agent.get(attacker_id);
    let attacker_strength = parts.strength.get(attacker_id).strength;
    let attacker_base_strength = parts.strength.get(attacker_id).base_strength;
    let attacker_wounds = utils::clamp_max(attacker_base_strength.0 - attacker_strength.0, 3);
    let target_dodge = agent_target.dodge;
    let attack_accuracy = agent_attacker.attack_accuracy;
    let attack_strength = agent_attacker.attack_strength;
    let k_min = attack_accuracy.0 - target_dodge.0 - attacker_wounds;
    let k_max = k_min + attack_strength.0;
    (k_min, k_max)
}

fn try_attack(state: &State, attacker_id: Id, target_id: Id) -> Option<Effect> {
    let parts = state.parts();
    let agent_attacker = state.parts().agent.get(attacker_id);
    let target_strength = parts.strength.get(target_id).strength;
    let target_armor = state::get_armor(state, target_id);
    let attack_strength = agent_attacker.attack_strength;
    let attacker_pos = Some(state.parts().pos.get(attacker_id).0);
    let (k_min, k_max) = hit_chance(state, attacker_id, target_id);
    if state.deterministic_mode() {
        // I want to be sure that I either will totally miss
        // or that I'll surely hit the target at a full force.
        let sure_miss = k_min < 0;
        let sure_hit = k_min > 10;
        assert!(
            sure_miss || sure_hit,
            "Hit isn't determined: {:?}",
            (k_min, k_max)
        );
    }
    let r = roll_dice(0, 11);
    let damage_raw = Strength(k_max - r);
    let damage = Strength(utils::clamp(damage_raw.0, 0, attack_strength.0));
    if damage_raw < Strength(0) {
        // That was a total miss
        return None;
    }
    let damage = correct_damage_with_armor(state, target_id, damage);
    let attack_break = utils::clamp_max(agent_attacker.attack_break, target_armor);
    let effect = if target_strength > damage {
        effect::Wound {
            damage,
            armor_break: attack_break,
            attacker_pos,
        }
        .into()
    } else {
        effect::Kill { attacker_pos }.into()
    };
    Some(effect)
}

fn execute_use_ability_explode_damage(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let from = state.parts().pos.get(command.id).0;
    for id in state.parts().agent.ids() {
        let pos = state.parts().pos.get(id).0;
        let distance = map::distance_hex(from, pos);
        if distance.0 > 1 || command.id == id {
            continue;
        }
        let damage = battle::Strength(1);
        let damage = correct_damage_with_armor(state, id, damage);
        let armor_break = Strength(1);
        let effects = vec![wound_break_kill(state, id, damage, armor_break)];
        context.instant_effects.push((id, effects));
    }
    assert!(context
        .instant_effects
        .iter()
        .position(|(id, _)| id == &command.id)
        .is_none());
    let effects = vec![Effect::Vanish];
    context.instant_effects.push((command.id, effects));
    context
}

fn execute_use_ability_explode_push(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let from = state.parts().pos.get(command.id).0;
    for id in state.parts().agent.ids() {
        let pos = state.parts().pos.get(id).0;
        let distance = map::distance_hex(from, pos);
        if distance.0 > 1 || command.id == id {
            continue;
        }
        let blocker_weight = state.parts().blocker.get(id).weight;
        let dir = Dir::get_dir_from_to(from, pos);
        let to = if PushStrength(Weight::Normal).can_push(blocker_weight) {
            Dir::get_neighbor_pos(pos, dir)
        } else {
            pos
        };
        let mut effects = Vec::new();
        if to == pos || (state.map().is_inboard(to) && !state::is_tile_blocked(state, to)) {
            let effect = effect::Knockback {
                from: pos,
                to,
                strength: PushStrength(Weight::Normal),
            };
            effects.push(effect.into());
            context.moved_actor_ids.push(id);
        }
        context.instant_effects.push((id, effects));
    }
    assert!(context
        .instant_effects
        .iter()
        .position(|(id, _)| id == &command.id)
        .is_none());
    let effects = vec![Effect::Vanish];
    context.instant_effects.push((command.id, effects));
    context
}

fn execute_use_ability_poison(state: &mut State, command: &command::UseAbility) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let id = state::blocker_id_at(state, command.pos);
    let owner = state.parts().belongs_to.get(id).0;
    let phase = Phase::from_player_id(owner);
    let effect = effect::Timed {
        duration: effect::Duration::Rounds(2),
        phase,
        effect: effect::Lasting::Poison,
    };
    context.timed_effects.push((id, vec![effect]));
    context.actor_ids.push(id);
    context
}

fn effect_create_object(state: &State, prototype: &ObjType, pos: PosHex) -> Effect {
    let name = prototype.clone();
    let mut components = state.prototype_for(prototype);
    components.extend_from_slice(&[component::Pos(pos).into(), component::Meta { name }.into()]);
    effect::Create {
        pos,
        prototype: prototype.clone(),
        components,
        is_teleported: false,
    }
    .into()
}

fn effect_create_agent(
    state: &State,
    prototype: &ObjType,
    player_id: PlayerId,
    pos: PosHex,
) -> Effect {
    let name = prototype.clone();
    let mut components = state.prototype_for(prototype);
    components.extend_from_slice(&[
        component::Pos(pos).into(),
        component::Meta { name }.into(),
        component::BelongsTo(player_id).into(),
    ]);
    effect::Create {
        pos,
        prototype: prototype.clone(),
        components,
        is_teleported: true,
    }
    .into()
}

fn throw_bomb(
    state: &mut State,
    command: &command::UseAbility,
    prototype: &ObjType,
    rounds: i32,
    ability: Ability,
) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let pos = state.parts().pos.get(command.id).0;
    let effect_create = effect_create_object(state, prototype, pos);
    let id = state.alloc_id();
    let effect_throw = effect::Throw {
        from: pos,
        to: command.pos,
    }
    .into();
    let effects = vec![effect_create, effect_throw];
    context.instant_effects.push((id, effects));
    let planned_ability = component::PlannedAbility {
        rounds,
        phase: Phase::from_player_id(state.player_id()),
        ability,
    };
    context
        .scheduled_abilities
        .push((id, vec![planned_ability]));
    context
}

fn execute_use_ability_bomb_push(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    throw_bomb(state, command, &"bomb_push".into(), 0, Ability::ExplodePush)
}

fn execute_use_ability_bomb_damage(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    throw_bomb(
        state,
        command,
        &"bomb_damage".into(),
        1,
        Ability::ExplodeDamage,
    )
}

fn execute_use_ability_bomb_fire(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    throw_bomb(state, command, &"bomb_fire".into(), 1, Ability::ExplodeFire)
}

fn execute_use_ability_bomb_poison(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    throw_bomb(
        state,
        command,
        &"bomb_poison".into(),
        1,
        Ability::ExplodePoison,
    )
}

fn execute_use_ability_bomb_demonic(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    throw_bomb(
        state,
        command,
        &"bomb_demonic".into(),
        1,
        Ability::ExplodeDamage,
    )
}

fn execute_use_ability_summon(state: &mut State, command: &command::UseAbility) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let max_summoned_count = state.parts().summoner.get(command.id).count;
    let available_typenames = &["imp".into(), "toxic_imp".into(), "imp_bomber".into()];
    let existing_agents = existing_agent_typenames(state, state.player_id());
    let mut new_agents = Vec::new();
    for pos in state::free_neighbor_positions(state, command.pos, max_summoned_count as _) {
        let prototype = choose_who_to_summon(&existing_agents, &new_agents, available_typenames);
        let effect_create = effect_create_agent(state, &prototype, state.player_id(), pos);
        let id = state.alloc_id();
        let effects = vec![effect_create, Effect::Stun];
        new_agents.push(prototype);
        context.instant_effects.push((id, effects));
        context.moved_actor_ids.push(id);
        context.reaction_attack_targets.push(id);
    }
    context
}

fn execute_use_ability_bloodlust(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let id = state::blocker_id_at(state, command.pos);
    if state.parts().belongs_to.get_opt(id).is_some() {
        let owner = state.parts().belongs_to.get(id).0;
        let phase = Phase::from_player_id(owner);
        let effect = effect::Timed {
            duration: effect::Duration::Rounds(3),
            phase,
            effect: effect::Lasting::Bloodlust,
        };
        context.timed_effects.push((id, vec![effect]));
    }
    context.actor_ids.push(id);
    context
}

fn execute_use_ability(state: &mut State, cb: Cb, command: &command::UseAbility) {
    let mut context = match command.ability {
        Ability::Knockback(a) => execute_use_ability_knockback(state, command, a),
        Ability::Club => execute_use_ability_club(state, command),
        Ability::Jump(_) => execute_use_ability_jump(state, command),
        Ability::Dash => execute_use_ability_dash(state, command),
        Ability::Rage => execute_use_ability_rage(state, command),
        Ability::Heal(a) => execute_use_ability_heal(state, command, a),
        Ability::Vanish => execute_use_ability_vanish(state, command),
        Ability::ExplodeFire => execute_use_ability_explode_fire(state, command),
        Ability::ExplodePoison => execute_use_ability_explode_poison(state, command),
        Ability::ExplodePush => execute_use_ability_explode_push(state, command),
        Ability::ExplodeDamage => execute_use_ability_explode_damage(state, command),
        Ability::Poison => execute_use_ability_poison(state, command),
        Ability::Bomb(_) => execute_use_ability_bomb_damage(state, command),
        Ability::BombPush(_) => execute_use_ability_bomb_push(state, command),
        Ability::BombFire(_) => execute_use_ability_bomb_fire(state, command),
        Ability::BombPoison(_) => execute_use_ability_bomb_poison(state, command),
        Ability::BombDemonic(_) => execute_use_ability_bomb_demonic(state, command),
        Ability::Summon => execute_use_ability_summon(state, command),
        Ability::Bloodlust => execute_use_ability_bloodlust(state, command),
    };
    context.actor_ids.push(command.id);
    let active_event = event::UseAbility {
        id: command.id,
        pos: command.pos,
        ability: command.ability.clone(),
    }
    .into();
    let event = Event {
        active_event,
        actor_ids: context.actor_ids,
        instant_effects: context.instant_effects,
        timed_effects: context.timed_effects,
        scheduled_abilities: context.scheduled_abilities,
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

fn existing_agent_typenames(state: &State, player_id: PlayerId) -> Vec<ObjType> {
    let mut existing_agents = Vec::new();
    for id in state::players_agent_ids(state, player_id) {
        let typename = state.parts().meta.get(id).name.clone();
        existing_agents.push(typename);
    }
    existing_agents
}

fn choose_who_to_summon(
    existing_agents: &[ObjType],
    new_agents: &[ObjType],
    available_typenames: &[ObjType],
) -> ObjType {
    assert!(!available_typenames.is_empty());
    let agents = existing_agents.iter().chain(new_agents);
    let mut map = HashMap::new();
    for typename in available_typenames {
        map.insert(typename, 0);
    }
    for typename in agents {
        if let Some(count) = map.get_mut(typename) {
            *count += 1;
        }
    }
    let (key, _value) = map
        .into_iter()
        .min_by_key(|&(_key, value)| value)
        .expect("The map can't be empty");
    key.clone()
}

#[cfg(test)]
mod tests {
    use crate::core::{
        battle::{
            effect::{self, Effect},
            Id,
        },
        map::PosHex,
    };

    use super::ExecuteContext;

    // TODO: Don't create Id's manually? Use a mocked State instead.

    #[test]
    fn test_merge_with_vector() {
        let mut context1 = ExecuteContext {
            actor_ids: vec![Id(0), Id(1)],
            ..Default::default()
        };
        let context2 = ExecuteContext {
            actor_ids: vec![Id(2), Id(3)],
            ..Default::default()
        };
        let context_expected = ExecuteContext {
            actor_ids: vec![Id(0), Id(1), Id(2), Id(3)],
            ..Default::default()
        };
        context1.merge_with(context2);
        assert_eq!(context_expected, context1);
    }

    #[test]
    fn test_merge_with_hashmap() {
        let mut instant_effects1 = Vec::new();
        let attacker_pos = PosHex { q: 0, r: 0 };
        let effect_kill: Effect = effect::Kill {
            attacker_pos: Some(attacker_pos),
        }
        .into();
        instant_effects1.push((Id(0), vec![effect_kill.clone(), Effect::Stun]));
        let mut context1 = ExecuteContext {
            instant_effects: instant_effects1,
            ..Default::default()
        };
        let effect_dodge = effect::Dodge { attacker_pos };
        let mut instant_effects2 = Vec::new();
        instant_effects2.push((Id(0), vec![Effect::Vanish, effect_dodge.clone().into()]));
        let context2 = ExecuteContext {
            instant_effects: instant_effects2,
            ..Default::default()
        };
        let mut instant_effects_expected = Vec::new();
        instant_effects_expected.push((
            Id(0),
            vec![
                effect_kill,
                Effect::Stun,
                Effect::Vanish,
                effect_dodge.into(),
            ],
        ));
        let context_expected = ExecuteContext {
            instant_effects: instant_effects_expected,
            ..Default::default()
        };
        context1.merge_with(context2);
        assert_eq!(context_expected, context1);
    }
}
