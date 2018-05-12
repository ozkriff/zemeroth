use std::collections::HashMap;

use core::{Moves, ObjId, PlayerId, PosHex};
use core::ability::{Ability, PassiveAbility};
use core::effect::{Effect, LastingEffect, TimedEffect};
use core::movement::Path;

#[derive(Clone, Debug)]
pub struct Event {
    /// "Core" event
    pub active_event: ActiveEvent,

    /// These agent's stats must be updated
    pub actor_ids: Vec<ObjId>,

    pub instant_effects: HashMap<ObjId, Vec<Effect>>,
    pub timed_effects: HashMap<ObjId, Vec<TimedEffect>>,
}

#[derive(Debug, Clone)]
pub enum ActiveEvent {
    Create,
    EndTurn(EndTurn),
    BeginTurn(BeginTurn),
    UseAbility(UseAbility),
    UsePassiveAbility(UsePassiveAbility),
    MoveTo(MoveTo),
    Attack(Attack),
    EffectTick(EffectTick),
    EffectEnd(EffectEnd),
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

#[derive(Debug, Clone)]
pub struct UseAbility {
    pub id: ObjId,
    pub pos: PosHex,
    pub ability: Ability,
}

#[derive(Debug, Clone)]
pub struct UsePassiveAbility {
    pub id: ObjId,
    pub pos: PosHex,
    pub ability: PassiveAbility,
}

#[derive(Debug, Clone)]
pub struct EffectTick {
    pub id: ObjId,
    pub effect: LastingEffect,
}

#[derive(Debug, Clone)]
pub struct EffectEnd {
    pub id: ObjId,
    pub effect: LastingEffect,
}
