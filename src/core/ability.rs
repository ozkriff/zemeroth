use core::{Attacks, Strength};
use core::map::Distance;

/// Active ability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ability {
    Knockback,
    Club,
    Jump(Jump),
    Poison,
    Explode,
    ExplodeFire,
    ExplodePoison,
    Bomb(Bomb),
    BombFire(BombFire),
    BombPoison(BombPoison),
    Summon(Summon),
    Vanish,
    Dash,
    Rage(Rage),
    Heal(Heal),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Summon(pub i32); // TODO: i32 -> ???

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Jump(pub Distance);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rage(pub Attacks);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Heal(pub Strength);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Bomb(pub Distance);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BombPoison(pub Distance);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BombFire(pub Distance);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RechargeableAbility {
    pub ability: Ability,

    #[serde(default = "default_status")]
    pub status: Status,

    pub base_cooldown: i32, // TODO: i32 -> Rounds
}

impl Ability {
    pub fn to_str(&self) -> &str {
        match *self {
            Ability::Knockback => "Knockback",
            Ability::Club => "Club",
            Ability::Jump(_) => "Jump",
            Ability::Poison => "Poison",
            Ability::Explode => "Explode",
            Ability::ExplodeFire => "Explode Fire",
            Ability::ExplodePoison => "Explode Poison",
            Ability::Bomb(_) => "Bomb",
            Ability::BombFire(_) => "Fire bomb",
            Ability::BombPoison(_) => "Poison bomb",
            Ability::Vanish => "Vanish",
            Ability::Summon(_) => "Summon",
            Ability::Dash => "Dash",
            Ability::Rage(_) => "Rage",
            Ability::Heal(_) => "Heal",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PassiveAbility {
    HeavyImpact,
    SpawnPoisonCloudOnDeath,
    Burn,
    Poison,
    PoisonAttack,
    Regenerate(Regenerate),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Regenerate(pub Strength);

impl PassiveAbility {
    pub fn to_str(&self) -> &str {
        match *self {
            PassiveAbility::HeavyImpact => "Heavy impact",
            PassiveAbility::SpawnPoisonCloudOnDeath => "Spawn a poison cloud on death",
            PassiveAbility::Burn => "Burn",
            PassiveAbility::Poison => "Poison",
            PassiveAbility::PoisonAttack => "Poison attack",
            PassiveAbility::Regenerate(_) => "Regenerate",
        }
    }
}
