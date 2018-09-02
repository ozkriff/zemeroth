pub use self::state::State;

pub mod command;
pub mod component;
pub mod event;
pub mod execute;
pub mod state;

#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub enum TileType {
    Plain,
    Water,
}

impl Default for TileType {
    fn default() -> Self {
        TileType::Plain
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(i32);

// TODO: is this really a good idea? Not sure. This way `0` id can be easily created by everyone.
impl Default for Id {
    fn default() -> Self {
        Id(0)
    }
}
