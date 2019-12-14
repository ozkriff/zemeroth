use std::{default::Default, fmt};

use serde::{Deserialize, Serialize};

use crate::core::{battle::movement::MovePoints, map::PosHex};

pub use crate::core::battle::{check::check, execute::execute, state::State};

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

#[derive(
    Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq, Hash,
)]
pub struct Id(i32);

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Strength(pub i32);

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Weight {
    Normal = 0,
    Heavy = 1,
    Immovable = 2,
}
impl Default for Weight {
    fn default() -> Self {
        Weight::Normal
    }
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
        weight != Weight::Immovable && self.0 <= weight
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq)]
pub enum TileType {
    Plain,
    Rocks,
}

impl Default for TileType {
    fn default() -> Self {
        TileType::Plain
    }
}
