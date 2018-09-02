use core::{map::PosHex, strategy_map::Id};

#[derive(Debug, Clone)]
pub enum Event {
    MoveTo(MoveTo),
    EndTurn(EndTurn),
}

#[derive(Debug, Clone)]
pub struct MoveTo {
    pub id: Id,
    pub from: PosHex,
    pub to: PosHex,
}

#[derive(Debug, Clone)]
pub struct EndTurn;
