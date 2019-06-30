use std::collections::HashMap;

use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};

use crate::core::{
    battle::{component::ObjType, scenario::Scenario, state::BattleResult, PlayerId},
    utils,
};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    /// A mode for recruiting/upgrading fighters, etc.
    PreparingForBattle,

    /// The player is ready to start a new battle.
    ReadyForBattle,

    /// Campaign is finished, the player have won.
    Won,

    /// Campaign is finished, the player have lost.
    Failed,
}

/// An award that is given to the player after the successful battle.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Award {
    #[serde(default)]
    pub recruits: Vec<ObjType>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Action {
    Recruit(ObjType),
    Upgrade { from: ObjType, to: ObjType },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CampaignNode {
    pub scenario: Scenario,
    pub award: Award,
}

fn casualties(initial_agents: &[ObjType], survivors: &[ObjType]) -> Vec<ObjType> {
    let mut agents = initial_agents.to_vec();
    for typename in survivors {
        assert!(utils::try_remove_item(&mut agents, typename));
    }
    agents
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Plan {
    initial_agents: Vec<ObjType>,
    nodes: Vec<CampaignNode>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct State {
    scenarios: Vec<CampaignNode>,
    current_scenario_index: i32,
    mode: Mode,
    agents: Vec<ObjType>,
    last_battle_casualties: Vec<ObjType>,
    upgrades: HashMap<ObjType, Vec<ObjType>>,
    actions: Vec<Action>,
}

impl State {
    pub fn new(plan: Plan, upgrades: HashMap<ObjType, Vec<ObjType>>) -> Self {
        assert!(!plan.nodes.is_empty(), "No scenarios");
        Self {
            current_scenario_index: 0,
            scenarios: plan.nodes,
            mode: Mode::ReadyForBattle,
            agents: plan.initial_agents,
            last_battle_casualties: vec![],
            actions: vec![],
            upgrades,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn last_battle_casualties(&self) -> &[ObjType] {
        &self.last_battle_casualties
    }

    pub fn scenario(&self) -> &Scenario {
        assert!(!self.scenarios.is_empty());
        let i = self.current_scenario_index as usize;
        &self.scenarios[i].scenario
    }

    pub fn current_scenario_index(&self) -> i32 {
        self.current_scenario_index
    }

    pub fn scenarios_count(&self) -> i32 {
        self.scenarios.len() as _
    }

    pub fn agents(&self) -> &[ObjType] {
        &self.agents
    }

    pub fn available_actions(&self) -> &[Action] {
        &self.actions
    }

    pub fn exectute_action(&mut self, action: Action) {
        assert_eq!(self.mode(), Mode::PreparingForBattle);
        assert!(self.actions.contains(&action));
        match action {
            Action::Recruit(typename) => self.agents.push(typename),
            Action::Upgrade { from, to } => {
                assert!(utils::try_remove_item(&mut self.agents, &from));
                self.agents.push(to);
            }
        }
        self.actions = Vec::new();
        self.mode = Mode::ReadyForBattle;
    }

    pub fn report_battle_results(&mut self, result: &BattleResult) -> Result<(), ()> {
        if self.mode != Mode::ReadyForBattle {
            return Err(());
        }

        for survivor in &result.survivor_types {
            if !self.agents.contains(survivor) {
                // This agent isn't a survivor.
                return Err(());
            }
        }

        if result.winner_id == PlayerId(0) && result.survivor_types.is_empty() {
            // You can't win with no survivors.
            return Err(());
        }

        self.last_battle_casualties = casualties(&self.agents, &result.survivor_types);
        self.agents = result.survivor_types.clone();

        if result.winner_id != PlayerId(0) {
            self.mode = Mode::Failed;
            return Ok(());
        }

        if self.current_scenario_index + 1 >= self.scenarios.len() as _ {
            self.mode = Mode::Won;
        } else {
            let i = self.current_scenario_index as usize;

            for recruit in &self.scenarios[i].award.recruits {
                self.actions.push(Action::Recruit(recruit.clone()));
            }

            // Add one random upgrade (if available).
            {
                let mut upgrade_candidates = Vec::new();
                for agent in &self.agents {
                    for (from, upgrades) in &self.upgrades {
                        if from == agent {
                            for upgrade in upgrades {
                                let from = agent.clone();
                                let to = upgrade.clone();
                                upgrade_candidates.push(Action::Upgrade { from, to });
                            }
                        }
                    }
                }
                if let Some(final_action) = upgrade_candidates.choose(&mut thread_rng()) {
                    self.actions.push(final_action.clone());
                }
            }

            self.current_scenario_index += 1;

            self.mode = Mode::PreparingForBattle;
            if self.available_actions().is_empty() {
                // Skip the preparation step.
                self.mode = Mode::ReadyForBattle;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::core::{
        battle::{
            component::ObjType,
            scenario::{self, Line, ObjectsGroup, Scenario},
            state::BattleResult,
            PlayerId,
        },
        campaign::{Action, Award, CampaignNode, Mode, Plan, State},
    };

    type GroupTuple<'a> = (Option<PlayerId>, &'a str, Option<Line>, i32);

    impl<'a> From<GroupTuple<'a>> for ObjectsGroup {
        fn from(tuple: GroupTuple) -> Self {
            let (owner, typename, line, count) = tuple;
            let typename = typename.into();
            Self {
                owner,
                typename,
                line,
                count,
            }
        }
    }

    fn initial_agents() -> Vec<ObjType> {
        vec!["swordsman".into(), "alchemist".into()]
    }

    fn upgrades() -> HashMap<ObjType, Vec<ObjType>> {
        HashMap::new()
    }

    fn campaign_plan_short() -> Plan {
        let initial_agents = initial_agents();
        let nodes = {
            let id_0 = Some(PlayerId(0));
            let id_1 = Some(PlayerId(1));
            let scenario = Scenario {
                objects: vec![
                    (None, "boulder", None, 3).into(),
                    (id_0, "swordsman", Some(Line::Front), 1).into(),
                    (id_1, "imp", Some(Line::Front), 2).into(),
                ],
                ..scenario::default()
            };
            let award = Award { recruits: vec![] };
            let node = CampaignNode { scenario, award };
            vec![node]
        };
        Plan {
            nodes,
            initial_agents,
        }
    }

    fn campaign_plan_two_battles() -> Plan {
        let initial_agents = initial_agents();
        let id_0 = Some(PlayerId(0));
        let id_1 = Some(PlayerId(1));
        let nodes = vec![
            CampaignNode {
                scenario: Scenario {
                    objects: vec![
                        (None, "boulder", None, 3).into(),
                        (id_0, "swordsman", Some(Line::Front), 1).into(),
                        (id_1, "imp", Some(Line::Front), 2).into(),
                    ],
                    ..scenario::default()
                },
                award: Award {
                    recruits: vec!["spearman".into()],
                },
            },
            CampaignNode {
                scenario: Scenario {
                    objects: vec![
                        (None, "boulder", None, 3).into(),
                        (id_1, "imp", Some(Line::Front), 4).into(),
                    ],
                    ..scenario::default()
                },
                award: Award { recruits: vec![] },
            },
        ];
        Plan {
            nodes,
            initial_agents,
        }
    }

    #[test]
    #[should_panic(expected = "No scenarios")]
    fn empty_scenarios() {
        let empty_plan = Plan {
            nodes: Vec::new(),
            initial_agents: Vec::new(),
        };
        let _state = State::new(empty_plan, upgrades());
    }

    #[test]
    fn short_happy_path() {
        let mut state = State::new(campaign_plan_short(), upgrades());
        assert!(state.available_actions().is_empty());
        assert_eq!(state.mode(), Mode::ReadyForBattle);
        let battle_result = BattleResult {
            winner_id: PlayerId(0),
            survivor_types: initial_agents(),
        };
        state.report_battle_results(&battle_result).unwrap();
        assert_eq!(state.mode(), Mode::Won);
    }

    #[test]
    fn short_fail_path() {
        let mut state = State::new(campaign_plan_short(), upgrades());
        assert!(state.available_actions().is_empty());
        assert_eq!(state.mode(), Mode::ReadyForBattle);
        let battle_result = BattleResult {
            winner_id: PlayerId(1),
            survivor_types: vec![],
        };
        state.report_battle_results(&battle_result).unwrap();
        assert_eq!(state.last_battle_casualties().to_vec(), initial_agents());
        assert_eq!(state.mode(), Mode::Failed);
    }

    #[test]
    fn bad_survivors() {
        let mut state = State::new(campaign_plan_short(), upgrades());
        let battle_result = BattleResult {
            winner_id: PlayerId(1),
            survivor_types: vec!["imp".into()],
        };
        assert!(state.report_battle_results(&battle_result).is_err());
    }

    #[test]
    fn bad_battle_win_no_survivors() {
        let mut state = State::new(campaign_plan_short(), upgrades());
        let battle_result = BattleResult {
            winner_id: PlayerId(0),
            survivor_types: vec![],
        };
        assert!(state.report_battle_results(&battle_result).is_err());
    }

    #[test]
    fn recruit_and_casualty() {
        let mut state = State::new(campaign_plan_two_battles(), upgrades());
        assert!(state.available_actions().is_empty());
        assert_eq!(state.mode(), Mode::ReadyForBattle);
        {
            let battle_result = BattleResult {
                winner_id: PlayerId(0),
                survivor_types: initial_agents(),
            };
            state.report_battle_results(&battle_result).unwrap();
        }
        assert_eq!(
            state.available_actions(),
            &[Action::Recruit("spearman".into())]
        );
        assert!(state.last_battle_casualties().is_empty());
        assert_eq!(state.mode(), Mode::PreparingForBattle);
        state.exectute_action(Action::Recruit("spearman".into()));
        assert!(state.available_actions().is_empty());
        assert_eq!(state.mode(), Mode::ReadyForBattle);
        {
            let battle_result = BattleResult {
                winner_id: PlayerId(0),
                survivor_types: initial_agents(),
            };
            state.report_battle_results(&battle_result).unwrap();
        }
        assert_eq!(state.mode(), Mode::Won);
        assert_eq!(state.last_battle_casualties(), &["spearman".into()]);
    }

    #[test]
    fn upgrade_and_casualty() {
        let mut upgrades = HashMap::new();
        upgrades.insert("swordsman".into(), vec!["heavy_swordsman".into()]);
        let mut state = State::new(campaign_plan_two_battles(), upgrades);
        assert!(state.available_actions().is_empty());
        assert_eq!(state.mode(), Mode::ReadyForBattle);
        {
            let battle_result = BattleResult {
                winner_id: PlayerId(0),
                survivor_types: initial_agents(),
            };
            state.report_battle_results(&battle_result).unwrap();
        }
        let action_upgrade = Action::Upgrade {
            from: "swordsman".into(),
            to: "heavy_swordsman".into(),
        };
        let action_reqruit = Action::Recruit("spearman".into());
        assert_eq!(
            state.available_actions(),
            &[action_reqruit, action_upgrade.clone()]
        );
        assert!(state.last_battle_casualties().is_empty());
        assert_eq!(state.mode(), Mode::PreparingForBattle);
        state.exectute_action(action_upgrade);
        assert!(state.available_actions().is_empty());
        assert_eq!(state.mode(), Mode::ReadyForBattle);
        {
            let battle_result = BattleResult {
                winner_id: PlayerId(0),
                survivor_types: vec!["alchemist".into()],
            };
            state.report_battle_results(&battle_result).unwrap();
        }
        assert_eq!(state.mode(), Mode::Won);
        assert_eq!(state.last_battle_casualties(), &["heavy_swordsman".into()]);
    }
}
