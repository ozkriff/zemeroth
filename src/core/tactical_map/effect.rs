use serde::{Deserialize, Serialize};

use crate::core::{
    map::Dir,
    tactical_map::{
        component::{Component, ObjType},
        Phase, PosHex, Strength,
    },
};

#[derive(Clone, Debug, Copy, PartialEq, Serialize, Deserialize)]
pub enum Duration {
    Forever,
    Rounds(i32),
}

impl Duration {
    pub fn is_over(self) -> bool {
        match self {
            Duration::Rounds(n) => n <= 0,
            Duration::Forever => false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Timed {
    pub duration: Duration,
    pub phase: Phase,
    pub effect: Lasting,
}

/// Instant effects
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum Effect {
    Create(Create),
    Kill(Kill),
    Vanish,
    Stun,
    Heal(Heal),
    Wound(Wound),
    Knockback(Knockback),
    FlyOff(FlyOff), // TODO: flying boulders should make some damage
    Throw(Throw),
    Miss,
}

impl Effect {
    pub fn to_str(&self) -> &str {
        match *self {
            Effect::Create(_) => "Create",
            Effect::Kill(_) => "Kill",
            Effect::Vanish => "Vanish",
            Effect::Stun => "Stun",
            Effect::Heal(_) => "Heal",
            Effect::Wound(_) => "Wound",
            Effect::Knockback(_) => "Knockback",
            Effect::FlyOff(_) => "Fly off",
            Effect::Throw(_) => "Throw",
            Effect::Miss => "Miss",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Lasting {
    Poison,
    Stun,
}

impl Lasting {
    pub fn to_str(&self) -> &str {
        match *self {
            Lasting::Poison => "Poison",
            Lasting::Stun => "Stun",
        }
    }
}

// TODO: Move `armor_break` to a separate effect?
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Wound {
    pub damage: Strength,
    pub armor_break: Strength,
    pub dir: Option<Dir>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Kill {
    pub dir: Option<Dir>,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub struct Heal {
    pub strength: Strength,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Create {
    pub pos: PosHex,
    pub prototype: ObjType,
    pub components: Vec<Component>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct FlyOff {
    pub from: PosHex,
    pub to: PosHex,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Throw {
    pub from: PosHex,
    pub to: PosHex,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Knockback {
    pub from: PosHex,
    pub to: PosHex,
}
