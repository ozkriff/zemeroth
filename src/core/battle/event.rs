use crate::core::battle::{
    ability::{Ability, PassiveAbility},
    component::{PlannedAbility, WeaponType},
    effect::{self, Effect},
    movement::Path,
    state::BattleResult,
    Id, Moves, PlayerId, PosHex,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    /// "Core" event
    pub active_event: ActiveEvent,

    /// These agent's stats must be updated
    pub actor_ids: Vec<Id>,

    pub instant_effects: Vec<(Id, Vec<Effect>)>,

    /// If a lasting effect is applied to the same object twice
    /// then the new effect replaces the old one.
    pub timed_effects: Vec<(Id, Vec<effect::Timed>)>,

    /// If a scheduled ability is applied to the same object twice
    /// then the new planned ability replaces the old one.
    /// This can be used to reset bomb timers or to make fire last longer.
    pub scheduled_abilities: Vec<(Id, Vec<PlannedAbility>)>,
}

#[derive(Debug, Clone, PartialEq, derive_more::From)]
pub enum ActiveEvent {
    Create,
    EndBattle(EndBattle),
    EndTurn(EndTurn),
    BeginTurn(BeginTurn),
    UseAbility(UseAbility),
    UsePassiveAbility(UsePassiveAbility),
    MoveTo(MoveTo),
    Attack(Attack),
    EffectTick(EffectTick),
    EffectEnd(EffectEnd),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoveTo {
    pub path: Path,
    pub cost: Moves,
    pub id: Id,
}

#[derive(PartialEq, Clone, Debug)]
pub enum AttackMode {
    Active,
    Reactive,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attack {
    pub attacker_id: Id,
    pub target_id: Id,
    pub mode: AttackMode,
    pub weapon_type: WeaponType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EndBattle {
    pub result: BattleResult,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EndTurn {
    pub player_id: PlayerId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BeginTurn {
    pub player_id: PlayerId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UseAbility {
    pub id: Id,
    pub pos: PosHex,
    pub ability: Ability,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsePassiveAbility {
    pub id: Id,
    pub pos: PosHex,
    pub ability: PassiveAbility,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectTick {
    pub id: Id,
    pub effect: effect::Lasting,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectEnd {
    pub id: Id,
    pub effect: effect::Lasting,
}
