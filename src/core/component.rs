use core::{self, map, Attacks, Jokers, MovePoints, Moves, PlayerId};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pos(pub map::PosHex);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Blocker;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Strength {
    pub base_strength: core::Strength,
    pub strength: core::Strength,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Meta {
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BelongsTo(pub PlayerId);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Agent {
    // dynamic
    pub moves: Moves,
    pub attacks: Attacks,
    pub jokers: Jokers,

    // static
    pub attack_distance: map::Distance,
    pub move_points: MovePoints,
    pub reactive_attacks: Attacks,
    pub base_moves: Moves,
    pub base_attacks: Attacks,
    pub base_jokers: Jokers,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Component {
    Pos(Pos),
    Strength(Strength),
    Meta(Meta),
    BelongsTo(BelongsTo),
    Agent(Agent),
    Blocker(Blocker),
}
