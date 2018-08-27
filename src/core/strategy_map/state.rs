use core::map::{Distance, HexMap};

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

#[derive(Clone, Debug)]
pub struct State {
    map: HexMap<TileType>,
}

impl State {
    pub fn new() -> Self {
        let radius = Distance(3); // TODO: pass `Options` struct
        let map = HexMap::new(radius);
        Self { map }
    }

    pub fn map(&self) -> &HexMap<TileType> {
        &self.map
    }
}
