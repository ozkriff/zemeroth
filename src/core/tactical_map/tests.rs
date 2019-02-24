use pretty_assertions::assert_eq;

// TODO: don't construct your own ObjId(*), get them from the state!
//     They're kind of implementation detail (can be shifted
//     if I decide to create some intermediate object).

use crate::core::{
    map::{Dir, Distance, PosHex},
    tactical_map::{
        ability::{self, Ability, PassiveAbility, RechargeableAbility},
        check,
        command::{self, Command},
        component::{self, Component, ObjType, PlannedAbility, Prototypes, WeaponType},
        effect::{self, Effect},
        event::{self, ActiveEvent, AttackMode, Event},
        execute::{execute, ApplyPhase},
        movement::Path,
        scenario::{self, ExactObject, Scenario},
        state::BattleResult,
        Accuracy, Attacks, Dodge, Jokers, MovePoints, Moves, ObjId, Phase, PlayerId, State,
        Strength,
    },
};

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

trait ScenarioConstructor {
    fn object(self, player_id: PlayerId, object_name: &str, pos: PosHex) -> Self;
    fn object_without_owner(self, object_name: &str, pos: PosHex) -> Self;
}

impl ScenarioConstructor for Scenario {
    fn object(mut self, owner: PlayerId, object_name: &str, pos: PosHex) -> Self {
        self.exact_objects.push(ExactObject {
            owner: Some(owner),
            typename: object_name.into(),
            pos,
        });
        self
    }

    fn object_without_owner(mut self, object_name: &str, pos: PosHex) -> Self {
        self.exact_objects.push(ExactObject {
            owner: None,
            typename: object_name.into(),
            pos,
        });
        self
    }
}

/// A basic agent that can't do anything.
fn agent_dull() -> component::Agent {
    component::Agent {
        moves: Moves(0),
        attacks: Attacks(0),
        jokers: Jokers(0),
        attack_strength: Strength(0),
        attack_distance: Distance(0),
        attack_accuracy: Accuracy(0),
        weapon_type: WeaponType::Slash,
        attack_break: Strength(0),
        dodge: Dodge(0),
        move_points: MovePoints(0),
        reactive_attacks: Attacks(0),
        base_moves: Moves(0),
        base_attacks: Attacks(0),
        base_jokers: Jokers(0),
    }
}

fn component_agent_dull() -> Component {
    agent_dull().into()
}

fn component_agent_move_basic() -> Component {
    component::Agent {
        moves: Moves(1),
        move_points: MovePoints(3),
        ..agent_dull()
    }
    .into()
}

fn component_agent_always_hit() -> Component {
    component::Agent {
        attack_distance: Distance(1),
        attacks: Attacks(1),
        attack_accuracy: Accuracy(10 + 1),
        ..agent_dull()
    }
    .into()
}

fn component_agent_always_hit_strength_1() -> Component {
    component::Agent {
        attack_distance: Distance(1),
        attacks: Attacks(1),
        attack_accuracy: Accuracy(10 + 1),
        attack_strength: Strength(1),
        ..agent_dull()
    }
    .into()
}

fn component_agent_one_attack() -> Component {
    component::Agent {
        attacks: Attacks(1),
        ..agent_dull()
    }
    .into()
}

fn component_strength(n: i32) -> Component {
    component::Strength {
        strength: Strength(n),
        base_strength: Strength(n),
    }
    .into()
}

fn component_blocker() -> Component {
    component::Blocker.into()
}

fn component_abilities(abilities: &[Ability]) -> Component {
    let abilities = abilities.iter().cloned().map(rechargeable).collect();
    component::Abilities(abilities).into()
}

fn component_passive_abilities(abilities: &[PassiveAbility]) -> Component {
    component::PassiveAbilities(abilities.to_vec()).into()
}

fn component_meta(name: &str) -> Component {
    component::Meta { name: name.into() }.into()
}

fn event_end_turn(player_id: PlayerId, actor_ids: &[ObjId]) -> Event {
    Event {
        active_event: event::EndTurn { player_id }.into(),
        actor_ids: actor_ids.to_vec(),
        instant_effects: Vec::new(),
        timed_effects: Vec::new(),
        scheduled_abilities: Vec::new(),
    }
}

fn event_begin_turn(player_id: PlayerId, actor_ids: &[ObjId]) -> Event {
    Event {
        active_event: event::BeginTurn { player_id }.into(),
        actor_ids: actor_ids.to_vec(),
        instant_effects: Vec::new(),
        timed_effects: Vec::new(),
        scheduled_abilities: Vec::new(),
    }
}

fn event_end_battle(winner_id: PlayerId, survivor_types: &[ObjType]) -> Event {
    let active_event = event::EndBattle {
        result: BattleResult {
            winner_id,
            survivor_types: survivor_types.to_vec(),
        },
    };
    Event {
        active_event: active_event.into(),
        actor_ids: Vec::new(),
        instant_effects: Vec::new(),
        timed_effects: Vec::new(),
        scheduled_abilities: Vec::new(),
    }
}

fn prototypes(slice: &[(&str, Vec<Component>)]) -> Prototypes {
    let map = slice
        .iter()
        .cloned()
        .map(|(name, components)| (name.into(), components))
        .collect();
    let mut prototypes = Prototypes(map);
    prototypes.init_components();
    prototypes
}

fn debug_state_with_events(prototypes: Prototypes, scenario: Scenario) -> (State, Vec<Event>) {
    let mut events = Vec::new();
    let mut state = State::new(prototypes, scenario, &mut |_, event, phase| {
        if phase == ApplyPhase::Pre {
            events.push(event.clone());
        }
    });
    state.set_deterministic_mode(true);
    (state, events)
}

fn debug_state(prototypes: Prototypes, scenario: Scenario) -> State {
    let (state, _events) = debug_state_with_events(prototypes, scenario);
    state
}

fn try_exec(state: &mut State, command: impl Into<Command>) -> Result<Vec<Event>, check::Error> {
    let mut events = Vec::new();
    execute(state, &command.into(), &mut |_state, event, phase| {
        if phase == ApplyPhase::Pre {
            events.push(event.clone());
        }
    })?;
    Ok(events)
}

fn exec(state: &mut State, command: impl Into<Command>) -> Vec<Event> {
    try_exec(state, command).unwrap()
}

fn exec_and_check(state: &mut State, command: impl Into<Command>, expected_events: &[Event]) {
    let events = exec(state, command);
    assert_eq!(events.as_slice(), expected_events);
}

fn rechargeable_with_base_cooldown(ability: Ability, base_cooldown: i32) -> RechargeableAbility {
    let status = ability::Status::Ready;
    RechargeableAbility {
        ability,
        status,
        base_cooldown,
    }
}

fn rechargeable(ability: Ability) -> RechargeableAbility {
    let default_base_cooldown = 1;
    rechargeable_with_base_cooldown(ability, default_base_cooldown)
}

fn ability_throw_bomb() -> Ability {
    ability::Bomb(Distance(2)).into()
}

fn ability_throw_bomb_fire() -> Ability {
    ability::BombFire(Distance(2)).into()
}

fn ability_throw_bomb_poison() -> Ability {
    ability::BombPoison(Distance(2)).into()
}

fn ability_club() -> Ability {
    Ability::Club
}

#[test]
#[should_panic(expected = "NoPlayerAgents")]
fn bad_scenario_no_player_agents() {
    let prototypes = prototypes(&[("obj", Vec::new())]);
    let scenario = scenario::default().object_without_owner("obj", PosHex { q: 0, r: 0 });
    let (_state, _events) = debug_state_with_events(prototypes, scenario);
}

#[test]
#[should_panic(expected = "NoEnemyAgents")]
fn bad_scenario_no_enemy_agents() {
    let prototypes = prototypes(&[("agent", [component_agent_dull()].to_vec())]);
    let scenario = scenario::default().object(P0, "agent", PosHex { q: 0, r: 0 });
    let (_state, _events) = debug_state_with_events(prototypes, scenario);
}

#[should_panic(expected = "PosOutsideOfMap")]
#[test]
fn bad_scenario_bad_pos() {
    let prototypes = prototypes(&[("obj", Vec::new())]);
    let scenario = scenario::default().object(P0, "obj", PosHex { q: 10, r: 0 });
    let (_state, _events) = debug_state_with_events(prototypes, scenario);
}

// TODO: test that you can't execute any commands after the battle is over

#[test]
fn create() {
    let prototypes = prototypes(&[("agent", [component_agent_dull()].to_vec())]);
    let scenario = scenario::default()
        .object(P0, "agent", PosHex { q: 0, r: 0 })
        .object(P1, "agent", PosHex { q: 0, r: 2 });
    let (_state, events) = debug_state_with_events(prototypes, scenario);
    let expected_event_0 = Event {
        active_event: ActiveEvent::Create,
        actor_ids: vec![ObjId(0)],
        instant_effects: vec![(
            ObjId(0),
            vec![effect::Create {
                pos: PosHex { q: 0, r: 0 },
                prototype: "agent".into(),
                components: vec![
                    component_agent_dull(),
                    component::BelongsTo(P0).into(),
                    component::Pos(PosHex { q: 0, r: 0 }).into(),
                    component_meta("agent"),
                ],
            }
            .into()],
        )],
        timed_effects: Vec::new(),
        scheduled_abilities: Vec::new(),
    };
    let expected_event_1 = Event {
        active_event: ActiveEvent::Create,
        actor_ids: vec![ObjId(1)],
        instant_effects: vec![(
            ObjId(1),
            vec![effect::Create {
                pos: PosHex { q: 0, r: 2 },
                prototype: "agent".into(),
                components: vec![
                    component_agent_dull(),
                    component::BelongsTo(P1).into(),
                    component::Pos(PosHex { q: 0, r: 2 }).into(),
                    component_meta("agent"),
                ],
            }
            .into()],
        )],
        timed_effects: Vec::new(),
        scheduled_abilities: Vec::new(),
    };
    let expected_events = &[expected_event_0, expected_event_1];
    assert_eq!(&events, expected_events);
}

#[test]
fn basic_move() {
    let prototypes = prototypes(&[
        ("mover", [component_agent_move_basic()].to_vec()),
        ("dull", [component_agent_dull()].to_vec()),
    ]);
    let scenario = scenario::default()
        .object(P0, "mover", PosHex { q: 0, r: 0 })
        .object(P1, "dull", PosHex { q: 0, r: 2 });
    let mut state = debug_state(prototypes, scenario);
    let path = Path::new(vec![PosHex { q: 0, r: 0 }, PosHex { q: 0, r: 1 }]);
    let command = command::MoveTo {
        id: ObjId(0),
        path: path.clone(),
    };
    exec_and_check(
        &mut state,
        command,
        &[Event {
            active_event: event::MoveTo {
                path,
                cost: Moves(1),
                id: ObjId(0),
            }
            .into(),
            actor_ids: vec![ObjId(0)],
            instant_effects: Vec::new(),
            timed_effects: Vec::new(),
            scheduled_abilities: Vec::new(),
        }],
    );
}

#[test]
fn basic_attack() {
    let prototypes = prototypes(&[
        (
            "swordsman",
            [component_agent_always_hit(), component_strength(1)].to_vec(),
        ),
        (
            "imp",
            [component_agent_dull(), component_strength(1)].to_vec(),
        ),
    ]);
    let scenario = scenario::default()
        .object(P0, "swordsman", PosHex { q: 0, r: 0 })
        .object(P1, "imp", PosHex { q: 0, r: 1 });
    let mut state = debug_state(prototypes, scenario);
    exec_and_check(
        &mut state,
        command::Attack {
            attacker_id: ObjId(0),
            target_id: ObjId(1),
        },
        &[Event {
            active_event: event::Attack {
                attacker_id: ObjId(0),
                target_id: ObjId(1),
                mode: AttackMode::Active,
                weapon_type: WeaponType::Slash,
            }
            .into(),
            actor_ids: vec![ObjId(0)],
            instant_effects: vec![(
                ObjId(1),
                vec![effect::Wound {
                    damage: Strength(0),
                    armor_break: Strength(0),
                    dir: Some(Dir::SouthWest),
                }
                .into()],
            )],
            timed_effects: Vec::new(),
            scheduled_abilities: Vec::new(),
        }],
    );
}

#[test]
fn kill_and_end_the_battle() {
    let prototypes = prototypes(&[
        (
            "swordsman",
            [
                component_agent_always_hit_strength_1(),
                component_strength(1),
            ]
            .to_vec(),
        ),
        (
            "imp",
            [component_agent_dull(), component_strength(1)].to_vec(),
        ),
    ]);
    let scenario = scenario::default()
        .object(P0, "swordsman", PosHex { q: 0, r: 0 })
        .object(P1, "imp", PosHex { q: 0, r: 1 });
    let mut state = debug_state(prototypes, scenario);
    exec_and_check(
        &mut state,
        command::Attack {
            attacker_id: ObjId(0),
            target_id: ObjId(1),
        },
        &[
            Event {
                active_event: event::Attack {
                    attacker_id: ObjId(0),
                    target_id: ObjId(1),
                    mode: AttackMode::Active,
                    weapon_type: WeaponType::Slash,
                }
                .into(),
                actor_ids: vec![ObjId(0)],
                instant_effects: vec![(
                    ObjId(1),
                    vec![effect::Kill {
                        dir: Some(Dir::SouthWest),
                    }
                    .into()],
                )],
                timed_effects: Vec::new(),
                scheduled_abilities: Vec::new(),
            },
            event_end_battle(PlayerId(0), &["swordsman".into()]),
        ],
    );
}

#[test]
fn push_boulder() {
    // TODO: hammerman push a boulder
}

#[test]
fn stun_and_push_to_spikes() {
    // TODO: Stun an agent to spikes
    // They should die on these spikes before LastingEffect::Stun is over
}

#[test]
fn throw_bomb_no_harm() {
    let prototypes = prototypes(&[
        (
            "thrower",
            vec![
                component_agent_one_attack(),
                component_abilities(&[ability_throw_bomb()]),
            ],
        ),
        ("dull", [component_agent_dull()].to_vec()),
        ("bomb_damage", Vec::new()),
    ]);
    let scenario = scenario::default()
        .object(P0, "thrower", PosHex { q: 0, r: 0 })
        .object(P1, "dull", PosHex { q: 0, r: 3 });
    let mut state = debug_state(prototypes, scenario);
    exec_and_check(
        &mut state,
        command::UseAbility {
            id: ObjId(0),
            pos: PosHex { q: 0, r: -2 },
            ability: ability_throw_bomb(),
        },
        &[Event {
            active_event: event::UseAbility {
                id: ObjId(0),
                pos: PosHex { q: 0, r: -2 },
                ability: ability::Bomb(Distance(2)).into(),
            }
            .into(),
            actor_ids: vec![ObjId(0)],
            instant_effects: vec![(
                ObjId(2),
                vec![
                    effect::Create {
                        pos: PosHex { q: 0, r: 0 },
                        prototype: "bomb_damage".into(),
                        components: vec![
                            component::Pos(PosHex { q: 0, r: 0 }).into(),
                            component_meta("bomb_damage"),
                        ],
                    }
                    .into(),
                    effect::Throw {
                        from: PosHex { q: 0, r: 0 },
                        to: PosHex { q: 0, r: -2 },
                    }
                    .into(),
                ],
            )],
            timed_effects: Vec::new(),
            scheduled_abilities: vec![(
                ObjId(2),
                vec![PlannedAbility {
                    rounds: 1,
                    phase: Phase(0),
                    ability: Ability::ExplodeDamage,
                }],
            )],
        }],
    );
    exec_and_check(
        &mut state,
        command::EndTurn,
        &[
            event_end_turn(P0, &[ObjId(0)]),
            event_begin_turn(P1, &[ObjId(1)]),
        ],
    );
    exec_and_check(
        &mut state,
        command::EndTurn,
        &[
            event_end_turn(P1, &[ObjId(1)]),
            event_begin_turn(P0, &[ObjId(0)]),
            Event {
                active_event: event::UseAbility {
                    id: ObjId(2),
                    pos: PosHex { q: 0, r: -2 },
                    ability: Ability::ExplodeDamage,
                }
                .into(),
                actor_ids: vec![ObjId(2)],
                instant_effects: vec![(ObjId(2), vec![Effect::Vanish])],
                timed_effects: Vec::new(),
                scheduled_abilities: Vec::new(),
            },
        ],
    );
}

#[test]
fn throw_bomb_damage() {
    let prototypes = prototypes(&[
        (
            "thrower",
            vec![
                component_agent_one_attack(),
                component_abilities(&[ability_throw_bomb()]),
            ],
        ),
        (
            "weak",
            [component_agent_dull(), component_strength(2)].to_vec(),
        ),
        ("bomb_damage", Vec::new()),
    ]);
    let scenario = scenario::default()
        .object(P0, "thrower", PosHex { q: 0, r: 0 })
        .object(P1, "weak", PosHex { q: 0, r: 3 });
    let mut state = debug_state(prototypes, scenario);
    exec_and_check(
        &mut state,
        command::UseAbility {
            id: ObjId(0),
            pos: PosHex { q: 0, r: 2 },
            ability: ability_throw_bomb(),
        },
        &[Event {
            active_event: event::UseAbility {
                id: ObjId(0),
                pos: PosHex { q: 0, r: 2 },
                ability: ability::Bomb(Distance(2)).into(),
            }
            .into(),
            actor_ids: vec![ObjId(0)],
            instant_effects: vec![(
                ObjId(2),
                vec![
                    effect::Create {
                        pos: PosHex { q: 0, r: 0 },
                        prototype: "bomb_damage".into(),
                        components: vec![
                            component::Pos(PosHex { q: 0, r: 0 }).into(),
                            component_meta("bomb_damage"),
                        ],
                    }
                    .into(),
                    effect::Throw {
                        from: PosHex { q: 0, r: 0 },
                        to: PosHex { q: 0, r: 2 },
                    }
                    .into(),
                ],
            )],
            timed_effects: Vec::new(),
            scheduled_abilities: vec![(
                ObjId(2),
                vec![PlannedAbility {
                    rounds: 1,
                    phase: Phase(0),
                    ability: Ability::ExplodeDamage,
                }],
            )],
        }],
    );
    exec_and_check(
        &mut state,
        command::EndTurn,
        &[
            event_end_turn(P0, &[ObjId(0)]),
            event_begin_turn(P1, &[ObjId(1)]),
        ],
    );
    exec_and_check(
        &mut state,
        command::EndTurn,
        &[
            event_end_turn(P1, &[ObjId(1)]),
            event_begin_turn(P0, &[ObjId(0)]),
            Event {
                active_event: event::UseAbility {
                    id: ObjId(2),
                    pos: PosHex { q: 0, r: 2 },
                    ability: Ability::ExplodeDamage,
                }
                .into(),
                actor_ids: vec![ObjId(2)],
                instant_effects: vec![
                    (
                        ObjId(1),
                        vec![effect::Wound {
                            damage: Strength(1),
                            armor_break: Strength(0),
                            dir: None,
                        }
                        .into()],
                    ),
                    (ObjId(2), vec![Effect::Vanish]),
                ],
                timed_effects: Vec::new(),
                scheduled_abilities: Vec::new(),
            },
        ],
    );
}

#[test]
fn throw_bomb_poison() {
    let prototypes = prototypes(&[
        (
            "thrower",
            vec![
                component_agent_one_attack(),
                component_abilities(&[ability_throw_bomb_poison()]),
            ],
        ),
        ("weak", vec![component_agent_dull(), component_strength(2)]),
        ("bomb_poison", Vec::new()),
        (
            "poison_cloud",
            vec![component_passive_abilities(&[PassiveAbility::Poison])],
        ),
    ]);
    let scenario = scenario::default()
        .object(P0, "thrower", PosHex { q: 0, r: 0 })
        .object(P1, "weak", PosHex { q: 0, r: 3 });
    let mut state = debug_state(prototypes, scenario);
    exec_and_check(
        &mut state,
        command::UseAbility {
            id: ObjId(0),
            pos: PosHex { q: 0, r: 2 },
            ability: ability_throw_bomb_poison(),
        },
        &[Event {
            active_event: event::UseAbility {
                id: ObjId(0),
                pos: PosHex { q: 0, r: 2 },
                ability: ability::BombPoison(Distance(2)).into(),
            }
            .into(),
            actor_ids: vec![ObjId(0)],
            instant_effects: vec![(
                ObjId(2),
                vec![
                    effect::Create {
                        pos: PosHex { q: 0, r: 0 },
                        prototype: "bomb_poison".into(),
                        components: vec![
                            component::Pos(PosHex { q: 0, r: 0 }).into(),
                            component_meta("bomb_poison"),
                        ],
                    }
                    .into(),
                    effect::Throw {
                        from: PosHex { q: 0, r: 0 },
                        to: PosHex { q: 0, r: 2 },
                    }
                    .into(),
                ],
            )],
            timed_effects: Vec::new(),
            scheduled_abilities: vec![(
                ObjId(2),
                vec![PlannedAbility {
                    rounds: 1,
                    phase: Phase(0),
                    ability: Ability::ExplodePoison,
                }],
            )],
        }],
    );
    exec_and_check(
        &mut state,
        command::EndTurn,
        &[
            event_end_turn(P0, &[ObjId(0)]),
            event_begin_turn(P1, &[ObjId(1)]),
        ],
    );
    let create_poison_cloud = |pos| -> Effect {
        effect::Create {
            pos,
            prototype: "poison_cloud".into(),
            components: vec![
                component_passive_abilities(&[PassiveAbility::Poison]),
                component::Pos(pos).into(),
                component_meta("poison_cloud"),
            ],
        }
        .into()
    };
    let planned_ability_vanish = || -> PlannedAbility {
        PlannedAbility {
            rounds: 2,
            phase: Phase(0),
            ability: Ability::Vanish,
        }
    };
    exec_and_check(
        &mut state,
        command::EndTurn,
        &[
            event_end_turn(P1, &[ObjId(1)]),
            event_begin_turn(P0, &[ObjId(0)]),
            Event {
                active_event: event::UseAbility {
                    id: ObjId(2),
                    pos: PosHex { q: 0, r: 2 },
                    ability: Ability::ExplodePoison,
                }
                .into(),
                actor_ids: vec![ObjId(2)],
                instant_effects: vec![
                    (ObjId(2), vec![Effect::Vanish]),
                    (ObjId(3), vec![create_poison_cloud(PosHex { q: 0, r: 2 })]),
                    (ObjId(4), vec![create_poison_cloud(PosHex { q: 1, r: 2 })]),
                    (ObjId(5), vec![create_poison_cloud(PosHex { q: 1, r: 1 })]),
                    (ObjId(6), vec![create_poison_cloud(PosHex { q: 0, r: 1 })]),
                    (ObjId(7), vec![create_poison_cloud(PosHex { q: -1, r: 2 })]),
                    (ObjId(8), vec![create_poison_cloud(PosHex { q: -1, r: 3 })]),
                    (ObjId(9), vec![create_poison_cloud(PosHex { q: 0, r: 3 })]),
                ],
                timed_effects: vec![(
                    ObjId(1),
                    vec![effect::Timed {
                        duration: effect::Duration::Rounds(2),
                        phase: Phase(1),
                        effect: effect::Lasting::Poison,
                    }],
                )],
                scheduled_abilities: vec![
                    (ObjId(3), vec![planned_ability_vanish()]),
                    (ObjId(4), vec![planned_ability_vanish()]),
                    (ObjId(5), vec![planned_ability_vanish()]),
                    (ObjId(6), vec![planned_ability_vanish()]),
                    (ObjId(7), vec![planned_ability_vanish()]),
                    (ObjId(8), vec![planned_ability_vanish()]),
                    (ObjId(9), vec![planned_ability_vanish()]),
                ],
            },
        ],
    );
    exec_and_check(
        &mut state,
        command::EndTurn,
        &[
            event_end_turn(P0, &[ObjId(0)]),
            event_begin_turn(P1, &[ObjId(1)]),
            Event {
                active_event: event::UsePassiveAbility {
                    id: ObjId(9),
                    pos: PosHex { q: 0, r: 3 },
                    ability: PassiveAbility::Poison,
                }
                .into(),
                actor_ids: Vec::new(),
                instant_effects: Vec::new(),
                timed_effects: vec![(
                    ObjId(1),
                    vec![effect::Timed {
                        duration: effect::Duration::Rounds(2),
                        phase: Phase(1),
                        effect: effect::Lasting::Poison,
                    }],
                )],
                scheduled_abilities: Vec::new(),
            },
            Event {
                active_event: event::EffectTick {
                    id: ObjId(1),
                    effect: effect::Lasting::Poison,
                }
                .into(),
                actor_ids: vec![ObjId(1)],
                instant_effects: vec![(
                    ObjId(1),
                    vec![effect::Wound {
                        damage: Strength(1),
                        armor_break: Strength(0),
                        dir: None,
                    }
                    .into()],
                )],
                timed_effects: Vec::new(),
                scheduled_abilities: Vec::new(),
            },
            Event {
                active_event: event::EffectEnd {
                    id: ObjId(1),
                    effect: effect::Lasting::Poison,
                }
                .into(),
                actor_ids: vec![ObjId(1)],
                instant_effects: Vec::new(),
                timed_effects: Vec::new(),
                scheduled_abilities: Vec::new(),
            },
        ],
    );
    exec_and_check(
        &mut state,
        command::EndTurn,
        &[
            event_end_turn(P1, &[ObjId(1)]),
            event_begin_turn(P0, &[ObjId(0)]),
        ],
    );
    exec_and_check(
        &mut state,
        command::EndTurn,
        &[
            event_end_turn(P0, &[ObjId(0)]),
            event_begin_turn(P1, &[ObjId(1)]),
        ],
    );
    let event_vanish = |id, pos| -> Event {
        Event {
            active_event: event::UseAbility {
                id,
                pos,
                ability: Ability::Vanish,
            }
            .into(),
            actor_ids: vec![id],
            instant_effects: vec![(id, vec![Effect::Vanish])],
            timed_effects: Vec::new(),
            scheduled_abilities: Vec::new(),
        }
    };
    exec_and_check(
        &mut state,
        command::EndTurn,
        &[
            event_end_turn(P1, &[ObjId(1)]),
            event_begin_turn(P0, &[ObjId(0)]),
            event_vanish(ObjId(3), PosHex { q: 0, r: 2 }),
            event_vanish(ObjId(4), PosHex { q: 1, r: 2 }),
            event_vanish(ObjId(5), PosHex { q: 1, r: 1 }),
            event_vanish(ObjId(6), PosHex { q: 0, r: 1 }),
            event_vanish(ObjId(7), PosHex { q: -1, r: 2 }),
            event_vanish(ObjId(8), PosHex { q: -1, r: 3 }),
            event_vanish(ObjId(9), PosHex { q: 0, r: 3 }),
        ],
    );
}

#[test]
fn throw_two_fire_bombs() {
    let prototypes = prototypes(&[
        (
            "thrower",
            vec![
                component_agent_one_attack(),
                component_abilities(&[ability_throw_bomb_fire()]),
            ],
        ),
        ("dull", [component_agent_dull()].to_vec()),
        ("bomb_fire", Vec::new()),
        (
            "fire",
            vec![component_passive_abilities(&[PassiveAbility::Burn])],
        ),
    ]);
    let scenario = scenario::default()
        .object(P0, "thrower", PosHex { q: 0, r: 0 })
        .object(P1, "dull", PosHex { q: 0, r: -3 });
    let mut state = debug_state(prototypes, scenario);
    exec_and_check(
        &mut state,
        command::UseAbility {
            id: ObjId(0),
            pos: PosHex { q: 0, r: 2 },
            ability: ability_throw_bomb_fire(),
        },
        &[Event {
            active_event: event::UseAbility {
                id: ObjId(0),
                pos: PosHex { q: 0, r: 2 },
                ability: ability::BombFire(Distance(2)).into(),
            }
            .into(),
            actor_ids: vec![ObjId(0)],
            instant_effects: vec![(
                ObjId(2),
                vec![
                    effect::Create {
                        pos: PosHex { q: 0, r: 0 },
                        prototype: "bomb_fire".into(),
                        components: vec![
                            component::Pos(PosHex { q: 0, r: 0 }).into(),
                            component_meta("bomb_fire"),
                        ],
                    }
                    .into(),
                    effect::Throw {
                        from: PosHex { q: 0, r: 0 },
                        to: PosHex { q: 0, r: 2 },
                    }
                    .into(),
                ],
            )],
            timed_effects: Vec::new(),
            scheduled_abilities: vec![(
                ObjId(2),
                vec![PlannedAbility {
                    rounds: 1,
                    phase: Phase(0),
                    ability: Ability::ExplodeFire,
                }],
            )],
        }],
    );
    exec_and_check(
        &mut state,
        command::EndTurn,
        &[
            event_end_turn(P0, &[ObjId(0)]),
            event_begin_turn(P1, &[ObjId(1)]),
        ],
    );
    let create_fire = |pos| -> Effect {
        effect::Create {
            pos,
            prototype: "fire".into(),
            components: vec![
                component_passive_abilities(&[PassiveAbility::Burn]),
                component::Pos(pos).into(),
                component_meta("fire"),
            ],
        }
        .into()
    };
    let planned_ability_vanish = || -> PlannedAbility {
        PlannedAbility {
            rounds: 2,
            phase: Phase(0),
            ability: Ability::Vanish,
        }
    };
    exec_and_check(
        &mut state,
        command::EndTurn,
        &[
            event_end_turn(P1, &[ObjId(1)]),
            event_begin_turn(P0, &[ObjId(0)]),
            Event {
                active_event: event::UseAbility {
                    id: ObjId(2),
                    pos: PosHex { q: 0, r: 2 },
                    ability: Ability::ExplodeFire,
                }
                .into(),
                actor_ids: vec![ObjId(2)],
                instant_effects: vec![
                    (ObjId(2), vec![Effect::Vanish]),
                    (ObjId(3), vec![create_fire(PosHex { q: 0, r: 2 })]),
                    (ObjId(4), vec![create_fire(PosHex { q: 1, r: 2 })]),
                    (ObjId(5), vec![create_fire(PosHex { q: 1, r: 1 })]),
                    (ObjId(6), vec![create_fire(PosHex { q: 0, r: 1 })]),
                    (ObjId(7), vec![create_fire(PosHex { q: -1, r: 2 })]),
                    (ObjId(8), vec![create_fire(PosHex { q: -1, r: 3 })]),
                    (ObjId(9), vec![create_fire(PosHex { q: 0, r: 3 })]),
                ],
                timed_effects: Vec::new(),
                scheduled_abilities: vec![
                    (ObjId(3), vec![planned_ability_vanish()]),
                    (ObjId(4), vec![planned_ability_vanish()]),
                    (ObjId(5), vec![planned_ability_vanish()]),
                    (ObjId(6), vec![planned_ability_vanish()]),
                    (ObjId(7), vec![planned_ability_vanish()]),
                    (ObjId(8), vec![planned_ability_vanish()]),
                    (ObjId(9), vec![planned_ability_vanish()]),
                ],
            },
        ],
    );
    exec_and_check(
        &mut state,
        command::UseAbility {
            id: ObjId(0),
            pos: PosHex { q: 1, r: 1 },
            ability: ability_throw_bomb_fire(),
        },
        &[Event {
            active_event: event::UseAbility {
                id: ObjId(0),
                pos: PosHex { q: 1, r: 1 },
                ability: ability::BombFire(Distance(2)).into(),
            }
            .into(),
            actor_ids: vec![ObjId(0)],
            instant_effects: vec![(
                ObjId(10),
                vec![
                    effect::Create {
                        pos: PosHex { q: 0, r: 0 },
                        prototype: "bomb_fire".into(),
                        components: vec![
                            component::Pos(PosHex { q: 0, r: 0 }).into(),
                            component_meta("bomb_fire"),
                        ],
                    }
                    .into(),
                    effect::Throw {
                        from: PosHex { q: 0, r: 0 },
                        to: PosHex { q: 1, r: 1 },
                    }
                    .into(),
                ],
            )],
            timed_effects: Vec::new(),
            scheduled_abilities: vec![(
                ObjId(10),
                vec![PlannedAbility {
                    rounds: 1,
                    phase: Phase(0),
                    ability: Ability::ExplodeFire,
                }],
            )],
        }],
    );
    exec_and_check(
        &mut state,
        command::EndTurn,
        &[
            event_end_turn(P0, &[ObjId(0)]),
            event_begin_turn(P1, &[ObjId(1)]),
        ],
    );
    exec_and_check(
        &mut state,
        command::EndTurn,
        &[
            event_end_turn(P1, &[ObjId(1)]),
            event_begin_turn(P0, &[ObjId(0)]),
            Event {
                active_event: event::UseAbility {
                    id: ObjId(10),
                    pos: PosHex { q: 1, r: 1 },
                    ability: Ability::ExplodeFire,
                }
                .into(),
                actor_ids: vec![ObjId(10)],
                instant_effects: vec![
                    (ObjId(10), vec![Effect::Vanish]),
                    (ObjId(11), vec![create_fire(PosHex { q: 2, r: 1 })]),
                    (ObjId(12), vec![create_fire(PosHex { q: 2, r: 0 })]),
                    (ObjId(13), vec![create_fire(PosHex { q: 1, r: 0 })]),
                ],
                timed_effects: Vec::new(),
                scheduled_abilities: vec![
                    (ObjId(5), vec![planned_ability_vanish()]),
                    (ObjId(11), vec![planned_ability_vanish()]),
                    (ObjId(12), vec![planned_ability_vanish()]),
                    (ObjId(13), vec![planned_ability_vanish()]),
                    (ObjId(6), vec![planned_ability_vanish()]),
                    (ObjId(3), vec![planned_ability_vanish()]),
                    (ObjId(4), vec![planned_ability_vanish()]),
                ],
            },
        ],
    );
    exec_and_check(
        &mut state,
        command::EndTurn,
        &[
            event_end_turn(P0, &[ObjId(0)]),
            event_begin_turn(P1, &[ObjId(1)]),
        ],
    );
    let event_vanish = |id, pos| -> Event {
        Event {
            active_event: event::UseAbility {
                id,
                pos,
                ability: Ability::Vanish,
            }
            .into(),
            actor_ids: vec![id],
            instant_effects: vec![(id, vec![Effect::Vanish])],
            timed_effects: Vec::new(),
            scheduled_abilities: Vec::new(),
        }
    };
    exec_and_check(
        &mut state,
        command::EndTurn,
        &[
            event_end_turn(P1, &[ObjId(1)]),
            event_begin_turn(P0, &[ObjId(0)]),
            event_vanish(ObjId(7), PosHex { q: -1, r: 2 }),
            event_vanish(ObjId(8), PosHex { q: -1, r: 3 }),
            event_vanish(ObjId(9), PosHex { q: 0, r: 3 }),
        ],
    );
    exec_and_check(
        &mut state,
        command::EndTurn,
        &[
            event_end_turn(P0, &[ObjId(0)]),
            event_begin_turn(P1, &[ObjId(1)]),
        ],
    );
    exec_and_check(
        &mut state,
        command::EndTurn,
        &[
            event_end_turn(P1, &[ObjId(1)]),
            event_begin_turn(P0, &[ObjId(0)]),
            event_vanish(ObjId(3), PosHex { q: 0, r: 2 }),
            event_vanish(ObjId(4), PosHex { q: 1, r: 2 }),
            event_vanish(ObjId(5), PosHex { q: 1, r: 1 }),
            event_vanish(ObjId(6), PosHex { q: 0, r: 1 }),
            event_vanish(ObjId(11), PosHex { q: 2, r: 1 }),
            event_vanish(ObjId(12), PosHex { q: 2, r: 0 }),
            event_vanish(ObjId(13), PosHex { q: 1, r: 0 }),
        ],
    );
}

#[test]
fn stun() {
    let prototypes = prototypes(&[
        (
            "attacker",
            vec![
                component_agent_one_attack(),
                component_abilities(&[ability_club()]),
            ],
        ),
        (
            "target",
            vec![
                component_agent_dull(),
                component_blocker(),
                component_strength(1),
            ],
        ),
    ]);
    let scenario = scenario::default()
        .object(P0, "attacker", PosHex { q: 0, r: 0 })
        .object(P1, "target", PosHex { q: 0, r: 1 });
    let mut state = debug_state(prototypes, scenario);
    exec_and_check(
        &mut state,
        command::UseAbility {
            id: ObjId(0),
            pos: PosHex { q: 0, r: 1 },
            ability: ability_club(),
        },
        &[Event {
            active_event: event::UseAbility {
                id: ObjId(0),
                pos: PosHex { q: 0, r: 1 },
                ability: ability_club(),
            }
            .into(),
            actor_ids: vec![ObjId(1), ObjId(0)],
            instant_effects: vec![(
                ObjId(1),
                vec![
                    effect::FlyOff {
                        from: PosHex { q: 0, r: 1 },
                        to: PosHex { q: 0, r: 2 },
                    }
                    .into(),
                    Effect::Stun,
                ],
            )],
            timed_effects: vec![(
                ObjId(1),
                vec![effect::Timed {
                    duration: effect::Duration::Rounds(2),
                    phase: Phase(1),
                    effect: effect::Lasting::Stun,
                }
                .into()],
            )],
            scheduled_abilities: Vec::new(),
        }],
    );
    exec_and_check(
        &mut state,
        command::EndTurn,
        &[
            event_end_turn(P0, &[ObjId(0)]),
            event_begin_turn(P1, &[ObjId(1)]),
            Event {
                active_event: event::EffectTick {
                    id: ObjId(1),
                    effect: effect::Lasting::Stun,
                }
                .into(),
                actor_ids: vec![ObjId(1)],
                instant_effects: vec![(ObjId(1), vec![Effect::Stun])],
                timed_effects: Vec::new(),
                scheduled_abilities: Vec::new(),
            },
        ],
    );
    exec_and_check(
        &mut state,
        command::EndTurn,
        &[
            event_end_turn(P1, &[ObjId(1)]),
            event_begin_turn(P0, &[ObjId(0)]),
        ],
    );
    exec_and_check(
        &mut state,
        command::EndTurn,
        &[
            event_end_turn(P0, &[ObjId(0)]),
            event_begin_turn(P1, &[ObjId(1)]),
            Event {
                active_event: event::EffectTick {
                    id: ObjId(1),
                    effect: effect::Lasting::Stun,
                }
                .into(),
                actor_ids: vec![ObjId(1)],
                instant_effects: vec![(ObjId(1), vec![Effect::Stun])],
                timed_effects: Vec::new(),
                scheduled_abilities: Vec::new(),
            },
            Event {
                active_event: event::EffectEnd {
                    id: ObjId(1),
                    effect: effect::Lasting::Stun,
                }
                .into(),
                actor_ids: vec![ObjId(1)],
                instant_effects: Vec::new(),
                timed_effects: Vec::new(),
                scheduled_abilities: Vec::new(),
            },
        ],
    );
}
