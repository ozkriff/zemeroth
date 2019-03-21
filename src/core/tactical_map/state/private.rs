use log::error;

use crate::core::{
    map,
    tactical_map::{
        command::{self, Command},
        component::{Component, ObjType, Parts, Prototypes},
        event::Event,
        execute,
        scenario::{self, Scenario},
        state::apply::apply,
        ObjId, PlayerId, TileType,
    },
};

#[derive(Clone, Debug, PartialEq)]
pub struct BattleResult {
    pub winner_id: PlayerId,
    pub survivor_types: Vec<ObjType>,
}

#[derive(Clone, Debug)]
pub struct State {
    parts: Parts,
    map: map::HexMap<TileType>,
    scenario: Scenario,
    player_id: PlayerId,
    prototypes: Prototypes,
    battle_result: Option<BattleResult>,

    /// Enables panics when non-deterministic functions are called.
    deterministic_mode: bool,
}

impl State {
    pub fn new(prototypes: Prototypes, scenario: Scenario, cb: execute::Cb) -> Self {
        scenario.check().expect("Bad scenario");
        assert!(scenario.map_radius.0 >= 3);
        let mut this = Self {
            map: map::HexMap::new(scenario.map_radius),
            player_id: PlayerId(0),
            scenario,
            parts: Parts::new(),
            prototypes,
            battle_result: None,
            deterministic_mode: false,
        };
        this.create_terrain();
        this.create_objects(cb);
        this
    }

    #[allow(dead_code)]
    pub fn set_deterministic_mode(&mut self, value: bool) {
        self.deterministic_mode = value;
    }

    pub fn deterministic_mode(&self) -> bool {
        self.deterministic_mode
    }

    pub fn scenario(&self) -> &Scenario {
        &self.scenario
    }

    // TODO: Handle Scenario::exact_tiles
    fn create_terrain(&mut self) {
        for _ in 0..self.scenario.rocky_tiles_count {
            let pos = match scenario::random_free_pos(self) {
                Some(pos) => pos,
                None => continue,
            };
            self.map.set_tile(pos, TileType::Rocks);
        }
    }

    // TODO: Handle Scenario::exact_objects
    fn create_objects(&mut self, cb: execute::Cb) {
        let player_id_initial = self.player_id();
        // TODO: Merge the cycles. Generate `exact_objects` based on `objects`.
        for group in self.scenario.objects.clone() {
            if let Some(player_id) = group.owner {
                self.set_player_id(player_id);
            }
            for _ in 0..group.count {
                let pos = match scenario::random_pos(self, group.owner, group.line) {
                    Some(pos) => pos,
                    None => {
                        error!("Can't find the position");
                        continue;
                    }
                };
                let command = Command::Create(command::Create {
                    prototype: group.typename.clone(),
                    pos,
                    owner: group.owner,
                });
                execute::execute(self, &command, cb).expect("Can't create an object");
            }
        }
        for group in self.scenario.exact_objects.clone() {
            if let Some(player_id) = group.owner {
                self.set_player_id(player_id);
            }
            let command = Command::Create(command::Create {
                prototype: group.typename.clone(),
                pos: group.pos,
                owner: group.owner,
            });
            execute::execute(self, &command, cb).expect("Can't create an object");
        }
        self.set_player_id(player_id_initial);
    }

    pub fn player_id(&self) -> PlayerId {
        self.player_id
    }

    pub fn next_player_id(&self) -> PlayerId {
        let current_player_id = PlayerId(self.player_id().0 + 1);
        if current_player_id.0 < self.scenario.players_count {
            current_player_id
        } else {
            PlayerId(0)
        }
    }

    pub fn parts(&self) -> &Parts {
        &self.parts
    }

    pub fn map(&self) -> &map::HexMap<TileType> {
        &self.map
    }

    pub(in crate::core) fn prototype_for(&self, name: &ObjType) -> Vec<Component> {
        let prototypes = &self.prototypes.0;
        prototypes[name].clone()
    }

    pub fn battle_result(&self) -> &Option<BattleResult> {
        &self.battle_result
    }
}

/// Mutators. Be careful with them!
impl State {
    pub(super) fn parts_mut(&mut self) -> &mut Parts {
        &mut self.parts
    }

    pub(in crate::core) fn set_player_id(&mut self, new_value: PlayerId) {
        self.player_id = new_value;
    }

    pub(super) fn set_battle_result(&mut self, result: BattleResult) {
        self.battle_result = Some(result);
    }

    pub(in crate::core) fn alloc_id(&mut self) -> ObjId {
        self.parts.alloc_id()
    }

    pub fn apply(&mut self, event: &Event) {
        apply(self, event);
    }
}
