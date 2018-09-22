use core::map;
use core::tactical_map::{
    self,
    ability::{Ability, PassiveAbility, RechargeableAbility},
    effect::TimedEffect,
    Attacks, Jokers, MovePoints, Moves, ObjId, Phase, PlayerId,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Pos(pub map::PosHex);

/// Blocks the whole tile. Two blocker objects can't coexist in one tile.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Blocker;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Strength {
    pub base_strength: tactical_map::Strength,
    pub strength: tactical_map::Strength,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Armor {
    pub armor: tactical_map::Strength,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Meta {
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BelongsTo(pub PlayerId);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Agent {
    // dynamic
    pub moves: Moves,
    pub attacks: Attacks,
    pub jokers: Jokers,

    // static
    pub attack_strength: tactical_map::Strength,
    pub attack_distance: map::Distance,

    #[serde(default = "strength_zero")]
    pub attack_break: tactical_map::Strength,

    pub move_points: MovePoints,
    pub reactive_attacks: Attacks,

    pub base_moves: Moves,
    pub base_attacks: Attacks,
    pub base_jokers: Jokers,
}

fn strength_zero() -> tactical_map::Strength {
    tactical_map::Strength(0)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Abilities(pub Vec<RechargeableAbility>);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PassiveAbilities(pub Vec<PassiveAbility>);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Effects(pub Vec<TimedEffect>);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PlannedAbility {
    // TODO: use real types + take effect::Duration into consideration
    pub rounds: i32,
    pub phase: Phase,
    pub ability: Ability,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Schedule {
    pub planned: Vec<PlannedAbility>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Summoner {
    pub count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Component {
    Pos(Pos),
    Strength(Strength),
    Armor(Armor),
    Meta(Meta),
    BelongsTo(BelongsTo),
    Agent(Agent),
    Blocker(Blocker),
    Abilities(Abilities),
    PassiveAbilities(PassiveAbilities),
    Effects(Effects),
    Schedule(Schedule),
    Summoner(Summoner),
}

zcomponents_storage!(Parts<ObjId>: {
    strength: Strength,
    armor: Armor,
    pos: Pos,
    meta: Meta,
    belongs_to: BelongsTo,
    agent: Agent,
    blocker: Blocker,
    abilities: Abilities,
    passive_abilities: PassiveAbilities,
    effects: Effects,
    schedule: Schedule,
    summoner: Summoner,
});

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prototypes(pub HashMap<String, Vec<Component>>);
