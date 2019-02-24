use serde::{Deserialize, Serialize};

use crate::core::{
    map::Distance,
    tactical_map::{Attacks, Strength},
};

/// Active ability.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, derive_more::From)]
pub enum Ability {
    Knockback,
    Club,
    Jump(Jump),
    Poison,
    ExplodePush,
    ExplodeDamage,
    ExplodeFire,
    ExplodePoison,
    Bomb(Bomb),
    BombPush(BombPush),
    BombFire(BombFire),
    BombPoison(BombPoison),
    BombDemonic(BombDemonic),
    Summon,
    Vanish,
    Dash,
    Rage(Rage),
    Heal(Heal),
}

// TODO: use named fields?
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Jump(pub Distance);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rage(pub Attacks);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Heal(pub Strength);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bomb(pub Distance);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BombDemonic(pub Distance);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BombPush {
    pub throw_distance: Distance,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BombPoison(pub Distance);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BombFire(pub Distance);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Status {
    Ready,
    Cooldown(i32), // TODO: i32 -> Rounds
}

impl Status {
    pub fn update(&mut self) {
        if let Status::Cooldown(ref mut rounds) = *self {
            *rounds -= 1;
        }
        if *self == Status::Cooldown(0) {
            *self = Status::Ready;
        }
    }
}

fn default_status() -> Status {
    Status::Ready
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RechargeableAbility {
    pub ability: Ability,

    #[serde(default = "default_status")]
    pub status: Status,

    pub base_cooldown: i32, // TODO: i32 -> Rounds
}

impl Ability {
    pub fn to_string(&self) -> String {
        match *self {
            Ability::Knockback => "Knockback".into(),
            Ability::Club => "Club".into(),
            Ability::Jump(a) => format!("Jump-{}", (a.0).0),
            Ability::Poison => "Poison".into(),
            Ability::ExplodePush => "Explode Push".into(),
            Ability::ExplodeDamage => "Explode Damage".into(),
            Ability::ExplodeFire => "Explode Fire".into(),
            Ability::ExplodePoison => "Explode Poison".into(),
            Ability::Bomb(a) => format!("Bomb-{}", (a.0).0),
            Ability::BombPush(a) => format!("Bomb Push-{}", (a.throw_distance).0),
            Ability::BombFire(a) => format!("Fire bomb-{}", (a.0).0),
            Ability::BombPoison(a) => format!("Poison bomb-{}", (a.0).0),
            Ability::BombDemonic(a) => format!("Bomb Demonic-{}", (a.0).0),
            Ability::Vanish => "Vanish".into(),
            Ability::Summon => "Summon".into(),
            Ability::Dash => "Dash".into(),
            Ability::Rage(a) => format!("Rage-{}", (a.0).0),
            Ability::Heal(a) => format!("Heal-{}", (a.0).0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum PassiveAbility {
    HeavyImpact,
    SpawnPoisonCloudOnDeath,
    Burn,
    Poison,
    SpikeTrap,
    PoisonAttack,
    Regenerate(Regenerate),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Regenerate(pub Strength);

impl PassiveAbility {
    pub fn to_string(self) -> String {
        match self {
            PassiveAbility::HeavyImpact => "Heavy impact".into(),
            PassiveAbility::SpawnPoisonCloudOnDeath => "Spawn a poison cloud on death".into(),
            PassiveAbility::Burn => "Burn".into(),
            PassiveAbility::Poison => "Poison".into(),
            PassiveAbility::SpikeTrap => "SpikeTrap".into(),
            PassiveAbility::PoisonAttack => "Poison attack".into(),
            PassiveAbility::Regenerate(a) => format!("Regenerate-{}", (a.0).0),
        }
    }
}
