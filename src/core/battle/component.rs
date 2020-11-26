use serde::{Deserialize, Serialize};
use zcomponents::zcomponents_storage;

use crate::core::{
    battle::{
        self,
        ability::{Ability, PassiveAbility, RechargeableAbility},
        effect::Timed,
        Attacks, Id, Jokers, MovePoints, Moves, Phase, PlayerId, Rounds,
    },
    map,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Pos(pub map::PosHex);

/// Blocks the whole tile. Two blocker objects can't coexist in one tile.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Blocker {
    #[serde(default)]
    pub weight: battle::Weight,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Strength {
    #[serde(default)]
    pub base_strength: battle::Strength,

    pub strength: battle::Strength,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Armor {
    pub armor: battle::Strength,
}

#[serde(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ObjType(pub String);

impl From<&str> for ObjType {
    fn from(s: &str) -> Self {
        ObjType(s.into())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Meta {
    pub name: ObjType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BelongsTo(pub PlayerId);

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug, Eq, Hash)]
pub enum WeaponType {
    Slash,
    Smash,
    Pierce,
    Claw,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Agent {
    // dynamic
    pub moves: Moves,
    pub attacks: Attacks,
    pub jokers: Jokers,

    // static
    pub attack_strength: battle::Strength,
    pub attack_distance: map::Distance,
    pub attack_accuracy: battle::Accuracy,
    pub weapon_type: WeaponType,

    #[serde(default)]
    pub attack_break: battle::Strength,

    #[serde(default)]
    pub dodge: battle::Dodge,

    pub move_points: MovePoints,
    pub reactive_attacks: Attacks,

    #[serde(default)]
    pub base_moves: Moves,

    #[serde(default)]
    pub base_attacks: Attacks,

    #[serde(default)]
    pub base_jokers: Jokers,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Abilities(pub Vec<RechargeableAbility>);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PassiveAbilities(pub Vec<PassiveAbility>);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Effects(pub Vec<Timed>);

// TODO: Move to `ability` mod?
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PlannedAbility {
    pub rounds: Rounds,
    pub phase: Phase,
    pub ability: Ability,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Schedule {
    pub planned: Vec<PlannedAbility>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Summoner {
    pub count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, derive_more::From)]
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

zcomponents_storage!(Parts<Id>: {
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
pub struct Prototypes(pub HashMap<ObjType, Vec<Component>>);

fn init_component(component: &mut Component) {
    match component {
        Component::Agent(agent) => {
            agent.base_moves = agent.moves;
            agent.base_attacks = agent.attacks;
            agent.base_jokers = agent.jokers;
        }
        Component::Strength(strength) => {
            strength.base_strength = strength.strength;
        }
        _ => {}
    }
}

impl Prototypes {
    pub fn from_str(s: &str) -> Self {
        let mut prototypes: Prototypes = ron::de::from_str(s).expect("Can't parse the prototypes");
        prototypes.init_components();
        prototypes
    }

    pub fn init_components(&mut self) {
        for components in self.0.values_mut() {
            for component in components {
                init_component(component);
            }
        }
    }
}
