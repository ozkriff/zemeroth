use std::collections::HashMap;
use core::{Moves, ObjId, PlayerId, PosHex};
use core::component::Component;
use core::effect::Effect;
use core::movement::Path;

#[derive(Clone, Debug)]
pub struct Event {
    pub active_event: ActiveEvent,
    pub actor_ids: Vec<ObjId>,
    pub effects: HashMap<ObjId, Vec<Effect>>,
}

#[derive(Debug, Clone)]
pub enum ActiveEvent {
    Create(Create),
    MoveTo(MoveTo),
    Attack(Attack),
    EndTurn(EndTurn),
    BeginTurn(BeginTurn),
}

#[derive(Debug, Clone)]
pub struct Create {
    pub id: ObjId,
    pub pos: PosHex,
    pub prototype: String,
    pub components: Vec<Component>,
}

#[derive(Debug, Clone)]
pub struct MoveTo {
    pub path: Path,
    pub cost: Moves,
    pub id: ObjId,
}

#[derive(PartialEq, Clone, Debug)]
pub enum AttackMode {
    Active,
    Reactive,
}

#[derive(Debug, Clone)]
pub struct Attack {
    pub attacker_id: ObjId,
    pub target_id: ObjId,
    pub mode: AttackMode,
}

#[derive(Debug, Clone)]
pub struct EndTurn {
    pub player_id: PlayerId,
}

#[derive(Debug, Clone)]
pub struct BeginTurn {
    pub player_id: PlayerId,
}
