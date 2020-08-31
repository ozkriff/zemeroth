use serde::{Deserialize, Serialize};

use crate::core::battle::Weight;

/// Active ability.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, derive_more::From)]
pub enum Ability {
    Knockback,
    Club,
    Jump,
    LongJump,
    Poison,
    ExplodePush,
    ExplodeDamage,
    ExplodeFire,
    ExplodePoison,
    Bomb,
    BombPush,
    BombFire,
    BombPoison,
    BombDemonic,
    Summon,
    Vanish,
    Dash,
    Rage,
    Heal,
    GreatHeal,
    Bloodlust,
}

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
            Ability::Knockback => "Knockback".into(),
            Ability::Club => "Club".into(),
            Ability::Jump => "Jump".into(),
            Ability::LongJump => "Long Jump".into(),
            Ability::Poison => "Poison".into(),
            Ability::ExplodePush => "Explode Push".into(),
            Ability::ExplodeDamage => "Explode Damage".into(),
            Ability::ExplodeFire => "Explode Fire".into(),
            Ability::ExplodePoison => "Explode Poison".into(),
            Ability::Bomb => "Bomb".into(),
            Ability::BombPush => "Bomb Push".into(),
            Ability::BombFire => "Fire Bomb".into(),
            Ability::BombPoison => "Poison Bomb".into(),
            Ability::BombDemonic => "Demonic Bomb".into(),
            Ability::Vanish => "Vanish".into(),
            Ability::Summon => "Summon".into(),
            Ability::Dash => "Dash".into(),
            Ability::Rage => "Rage".into(),
            Ability::Heal => "Heal".into(),
            Ability::GreatHeal => "Great Heal".into(),
            Ability::Bloodlust => "Bloodlust".into(),
        }
    }

    pub fn description(&self) -> Vec<String> {
        match *self {
            Ability::Knockback => vec![
                "Push an adjusted object one tile away.".into(),
                "Can move objects with a weight up to Normal.".into(),
            ],
            Ability::Club => vec!["Stun an adjusted agent for one turn.".into()],
            Ability::Jump => vec![
                "Jump for up to 2 tiles.".into(),
                "Note: Triggers reaction attacks on landing.".into(),
            ],
            Ability::LongJump => vec![
                "Jump for up to 3 tiles.".into(),
                "Note: Triggers reaction attacks on landing.".into(),
            ],
            Ability::Bomb => vec![
                "Throw a bomb that explodes on the next turn.".into(),
                "Damages all agents on the neighbour tiles.".into(),
                "Can be thrown for up to 3 tiles.".into(),
            ],
            Ability::BombPush => vec![
                "Throw a bomb that explodes *instantly*.".into(),
                "Pushes all agents on the neighbour tiles.".into(),
                "Can be thrown for up to 3 tiles.".into(),
                format!("Can move objects with a weight up to {}.", Weight::Normal),
            ],
            Ability::BombFire => vec![
                "Throw a bomb that explodes on the next turn.".into(),
                "Creates 7 fires.".into(),
                "Can be thrown for up to 3 tiles.".into(),
            ],
            Ability::BombPoison => vec![
                "Throw a bomb that explodes on the next turn.".into(),
                "Creates 7 poison clouds.".into(),
                "Can be thrown for up to 3 tiles.".into(),
            ],
            Ability::BombDemonic => vec![
                "Throw a demonic bomb".into(),
                "that explodes on the next turn.".into(),
                "Damages all agents on the neighbour tiles.".into(),
                "Can be thrown for up to 3 tiles.".into(),
            ],
            Ability::Dash => vec![
                "Move one tile".into(),
                "without triggering any reaction attacks.".into(),
            ],
            Ability::Rage => vec!["Instantly receive 3 additional attacks.".into()],
            Ability::Heal => vec![
                "Heal 2 strength points.".into(),
                "Also, removes 'Poison' and 'Stun' lasting effects.".into(),
            ],
            Ability::GreatHeal => vec![
                "Heal 3 strength points.".into(),
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
    Regenerate,
}

impl PassiveAbility {
    pub fn title(self) -> String {
        match self {
            PassiveAbility::HeavyImpact => "Heavy Impact".into(),
            PassiveAbility::SpawnPoisonCloudOnDeath => "Spawn Poison Cloud on Death".into(),
            PassiveAbility::Burn => "Burn".into(),
            PassiveAbility::Poison => "Poison".into(),
            PassiveAbility::SpikeTrap => "Spike Trap".into(),
            PassiveAbility::PoisonAttack => "Poison Attack".into(),
            PassiveAbility::Regenerate => "Regenerate".into(),
        }
    }

    pub fn description(self) -> Vec<String> {
        match self {
            PassiveAbility::HeavyImpact => vec![
                "Regular attack throws the target one tile away.".into(),
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
            PassiveAbility::PoisonAttack => vec!["Regular attack poisons the target.".into()],
            PassiveAbility::Regenerate => vec!["Regenerates 1 strength points every turn.".into()],
        }
    }
}
