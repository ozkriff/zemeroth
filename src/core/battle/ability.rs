use serde::{Deserialize, Serialize};

use crate::core::{
    battle::{Attacks, PushStrength, Strength, Weight},
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
    #[serde(default)]
    pub strength: PushStrength,
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
    pub fn title(&self) -> String {
        match self {
            Ability::Knockback(a) => format!("Knockback ({})", a.strength.0),
            Ability::Club => "Club".into(),
            Ability::Jump(a) => format!("Jump ({})", (a.0).0),
            Ability::Poison => "Poison".into(),
            Ability::ExplodePush => "Explode Push".into(),
            Ability::ExplodeDamage => "Explode Damage".into(),
            Ability::ExplodeFire => "Explode Fire".into(),
            Ability::ExplodePoison => "Explode Poison".into(),
            Ability::Bomb(a) => format!("Bomb ({})", (a.0).0),
            Ability::BombPush(a) => format!("Bomb Push ({})", a.throw_distance.0),
            Ability::BombFire(a) => format!("Fire Bomb ({})", (a.0).0),
            Ability::BombPoison(a) => format!("Poison Bomb ({})", (a.0).0),
            Ability::BombDemonic(a) => format!("Demonic Bomb ({})", (a.0).0),
            Ability::Vanish => "Vanish".into(),
            Ability::Summon => "Summon".into(),
            Ability::Dash => "Dash".into(),
            Ability::Rage(a) => format!("Rage ({})", (a.0).0),
            Ability::Heal(a) => format!("Heal ({})", (a.0).0),
            Ability::Bloodlust => "Bloodlust".into(),
        }
    }

    pub fn extended_description(&self) -> Vec<String> {
        match *self {
            Ability::Knockback(a) => vec![
                "Push an adjusted object one tile away.".into(),
                format!("Can move objects with a weight up to {}.", a.strength.0),
            ],
            Ability::Club => vec!["Stun an adjusted agent for one turn.".into()],
            Ability::Jump(a) => vec![
                format!("Jump for up to {} tiles.", (a.0).0),
                "Note: Triggers reaction attacks on landing.".into(),
            ],
            Ability::Bomb(a) => vec![
                "Throw a bomb that explodes on the next turn.".into(),
                "Damages all agents on the neighbour tiles.".into(),
                format!("Can be thrown for up to {} tiles.", (a.0).0),
            ],
            Ability::BombPush(a) => vec![
                "Throw a bomb that explodes *instantly*.".into(),
                "Pushes all agents on the neighbour tiles.".into(),
                format!("Can be thrown for up to {} tiles.", a.throw_distance.0),
                format!("Can move objects with a weight up to {}.", Weight::Normal),
            ],
            Ability::BombFire(a) => vec![
                "Throw a bomb that explodes on the next turn.".into(),
                "Creates 7 fires.".into(),
                format!("Can be thrown for up to {} tiles.", (a.0).0),
            ],
            Ability::BombPoison(a) => vec![
                "Throw a bomb that explodes on the next turn.".into(),
                "Creates 7 poison clouds.".into(),
                format!("Can be thrown for up to {} tiles.", (a.0).0),
            ],
            Ability::BombDemonic(a) => vec![
                "Throw a demonic bomb".into(),
                "that explodes on the next turn.".into(),
                "Damages all agents on the neighbour tiles.".into(),
                format!("Can be thrown for up to {} tiles.", (a.0).0),
            ],
            Ability::Dash => vec![
                "Move one tile".into(),
                "without triggering any reaction attacks.".into(),
            ],
            Ability::Rage(a) => vec![format!("Instantly receive {} additional attacks.", (a.0).0)],
            Ability::Heal(a) => vec![
                format!("Heal {} strength points.", (a.0).0),
                "Also, removes 'Poison' and 'Stun' lasting effects.".into(),
            ],
            Ability::Summon => vec![
                "Summon a few lesser daemons.".into(),
                "The number of summoned daemons increases".into(),
                "by one with every use (up to six).".into(),
            ],
            Ability::Bloodlust => vec![
                "Cast the 'Bloodlust' lasting effect on a friendly agent.".into(),
                "This agent will receive three additional Jokers".into(),
                "for a few turns.".into(),
            ],
            Ability::Poison
            | Ability::Vanish
            | Ability::ExplodePush
            | Ability::ExplodeDamage
            | Ability::ExplodeFire
            | Ability::ExplodePoison => vec!["<internal ability>".into()],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum PassiveAbility {
    HeavyImpact,
    SpawnPoisonCloudOnDeath, // TODO: implement and employ it!
    Burn,
    Poison,
    SpikeTrap,
    PoisonAttack,
    Regenerate(Regenerate),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Regenerate(pub Strength);

impl PassiveAbility {
    pub fn title(self) -> String {
        match self {
            PassiveAbility::HeavyImpact => "Heavy Impact".into(),
            PassiveAbility::SpawnPoisonCloudOnDeath => "Spawn Poison Cloud on Death".into(),
            PassiveAbility::Burn => "Burn".into(),
            PassiveAbility::Poison => "Poison".into(),
            PassiveAbility::SpikeTrap => "Spike Trap".into(),
            PassiveAbility::PoisonAttack => "Poison Attack".into(),
            PassiveAbility::Regenerate(a) => format!("Regenerate ({})", (a.0).0),
        }
    }

    pub fn extended_description(self) -> Vec<String> {
        match self {
            PassiveAbility::HeavyImpact => vec![
                "Regular attack throws target one tile away.".into(),
                format!(
                    "Works on targets with a weight for up to {}.",
                    Weight::Normal
                ),
            ],
            PassiveAbility::SpawnPoisonCloudOnDeath => vec!["Not implemented yet.".into()],
            PassiveAbility::Burn => {
                vec!["Damages agents that enter into or begin their turn in the same tile.".into()]
            }
            PassiveAbility::Poison => {
                vec!["Poisons agents that enter into or begin their turn in the same tile.".into()]
            }
            PassiveAbility::SpikeTrap => {
                vec!["Damages agents that enter into or begin their turn in the same tile.".into()]
            }
            PassiveAbility::PoisonAttack => vec!["Regular attack poisons target.".into()],
            PassiveAbility::Regenerate(a) => vec![format!(
                "Regenerates {} strength points every turn.",
                (a.0).0
            )],
        }
    }
}
