use crate::core::{
    map::PosHex,
    tactical_map::{ability::Ability, component::ObjType, movement::Path, ObjId, PlayerId},
};

#[derive(Debug, Clone)]
pub enum Command {
    Create(Create),
    Attack(Attack),
    MoveTo(MoveTo),
    EndTurn(EndTurn),
    UseAbility(UseAbility),
}

#[derive(Debug, Clone)]
pub struct Create {
    pub owner: Option<PlayerId>,
    pub pos: PosHex,
    pub prototype: ObjType,
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

#[derive(Debug, Clone)]
pub struct UseAbility {
    pub id: ObjId,
    pub pos: PosHex,
    pub ability: Ability,
}
