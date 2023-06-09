use std::{default::Default, fmt};

use serde::{Deserialize, Serialize};

pub use crate::core::{
    battle::{check::check, execute::execute, movement::MovePoints, state::State},
    map::PosHex,
};

pub mod ability;
pub mod ai;
pub mod command;
pub mod component;
pub mod effect;
pub mod event;
pub mod execute;
pub mod movement;
pub mod scenario;
pub mod state;

mod check;

#[cfg(test)]
mod tests;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlayerId(pub i32);

/// An index of player's turn.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Phase(i32);

impl Phase {
    pub fn from_player_id(player_id: PlayerId) -> Self {
        Phase(player_id.0 as _)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, derive_more::From)]
#[serde(transparent)]
pub struct Rounds(pub i32);

impl Rounds {
    pub fn decrease(&mut self) {
        self.0 -= 1;
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for Rounds {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Turns(pub i32);

impl Turns {
    pub fn decrease(&mut self) {
        self.0 -= 1;
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for Turns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(
    Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq, Hash,
)]
pub struct Id(i32);

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Strength(pub i32);

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Weight {
    #[default]
    Normal = 0,

    Heavy = 1,

    Immovable = 2,
}

impl fmt::Display for Weight {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Weight::Normal => write!(f, "Normal"),
            Weight::Heavy => write!(f, "Heavy"),
            Weight::Immovable => write!(f, "Immovable"),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct PushStrength(pub Weight);

impl PushStrength {
    pub fn can_push(self, weight: Weight) -> bool {
        weight != Weight::Immovable && self.0 >= weight
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Attacks(pub i32);

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Moves(pub i32);

/// Move or Attack
#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Jokers(pub i32);

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Accuracy(pub i32);

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Dodge(pub i32);

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    #[default]
    Plain,

    Rocks,
}
