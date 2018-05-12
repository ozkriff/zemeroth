use std::default::Default;

use core::map::PosHex;
use core::movement::MovePoints;

pub use core::execute::execute;
pub use core::check::check;
pub use core::state::State;

pub mod command;
pub mod event;
pub mod movement;
pub mod effect;
pub mod map;
pub mod execute;
pub mod component;
pub mod ability;
pub mod utils;
pub mod state;

mod check;
mod apply;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlayerId(pub i32);

/// An index of player's turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phase(i32);

impl Phase {
    pub fn from_player_id(player_id: PlayerId) -> Self {
        Phase(player_id.0 as _)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ObjId(i32);

impl Default for ObjId {
    fn default() -> Self {
        ObjId(0)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Strength(pub i32);

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Attacks(pub i32);

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Moves(pub i32);

/// Move or Attack
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Jokers(pub i32);

#[derive(Clone, Copy, Debug, Hash)]
pub enum TileType {
    Plain,
    Rocks,
}

impl Default for TileType {
    fn default() -> Self {
        TileType::Plain
    }
}
