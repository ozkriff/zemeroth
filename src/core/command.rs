use core::{ObjId, PlayerId};
use core::map::PosHex;
use core::movement::Path;

#[derive(Debug, Clone)]
pub enum Command {
    Create(Create),
    Attack(Attack),
    MoveTo(MoveTo),
    EndTurn(EndTurn),
}

#[derive(Debug, Clone)]
pub struct Create {
    pub owner: PlayerId,
    pub pos: PosHex,
    pub prototype: String,
}

#[derive(Debug, Clone)]
pub struct Attack {
    pub attacker_id: ObjId,
    pub target_id: ObjId,
}

#[derive(Debug, Clone)]
pub struct MoveTo {
    pub id: ObjId,
    pub path: Path,
}

#[derive(Debug, Clone)]
pub struct EndTurn;
