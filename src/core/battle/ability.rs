use std::fmt;

use serde::{Deserialize, Serialize};

use crate::core::{
    battle::{Attacks, Strength},
    map::Distance,
};

/// Active ability.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, derive_more::From)]
pub enum Ability {
    Knockback(Knockback),
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
    Bloodlust,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Knockback {
    pub strength: Strength,
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

impl fmt::Display for Ability {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Ability::Knockback(a) => write!(f, "Knockback-{}", a.strength.0),
            Ability::Club => write!(f, "Club"),
            Ability::Jump(a) => write!(f, "Jump-{}", (a.0).0),
            Ability::Poison => write!(f, "Poison"),
            Ability::ExplodePush => write!(f, "Explode Push"),
            Ability::ExplodeDamage => write!(f, "Explode Damage"),
            Ability::ExplodeFire => write!(f, "Explode Fire"),
            Ability::ExplodePoison => write!(f, "Explode Poison"),
            Ability::Bomb(a) => write!(f, "Bomb-{}", (a.0).0),
            Ability::BombPush(a) => write!(f, "Bomb Push-{}", (a.throw_distance).0),
            Ability::BombFire(a) => write!(f, "Fire bomb-{}", (a.0).0),
            Ability::BombPoison(a) => write!(f, "Poison bomb-{}", (a.0).0),
            Ability::BombDemonic(a) => write!(f, "Bomb Demonic-{}", (a.0).0),
            Ability::Vanish => write!(f, "Vanish"),
            Ability::Summon => write!(f, "Summon"),
            Ability::Dash => write!(f, "Dash"),
            Ability::Rage(a) => write!(f, "Rage-{}", (a.0).0),
            Ability::Heal(a) => write!(f, "Heal-{}", (a.0).0),
            Ability::Bloodlust => write!(f, "Bloodlust"),
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

impl fmt::Display for PassiveAbility {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PassiveAbility::HeavyImpact => write!(f, "Heavy impact"),
            PassiveAbility::SpawnPoisonCloudOnDeath => write!(f, "Spawn a poison cloud on death"),
            PassiveAbility::Burn => write!(f, "Burn"),
            PassiveAbility::Poison => write!(f, "Poison"),
            PassiveAbility::SpikeTrap => write!(f, "SpikeTrap"),
            PassiveAbility::PoisonAttack => write!(f, "Poison attack"),
            PassiveAbility::Regenerate(a) => write!(f, "Regenerate-{}", (a.0).0),
        }
    }
}
