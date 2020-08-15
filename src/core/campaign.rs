use std::collections::HashMap;

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::core::{
    battle::{component::ObjType, scenario::Scenario, state::BattleResult, PlayerId},
    utils::{self, zrng},
};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    /// Recruiting/upgrading fighters or starting a new battle.
    PreparingForBattle,

    /// Campaign is finished, the player have won.
    Won,

    /// Campaign is finished, the player have lost.
    Failed,
}

#[serde(transparent)]
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, derive_more::From)]
pub struct Renown(pub i32);

// TODO: impl `Add` and `Sub` traits for `Renown`.

/// An award that is given to the player after the successful battle.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Award {
    #[serde(default)]
    pub recruits: Vec<ObjType>,

    pub renown: Renown,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Action {
    Recruit { agent_type: ObjType },
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
pub struct AgentInfo {
    pub cost: Renown,

    #[serde(default)]
    pub upgrades: Vec<ObjType>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct State {
    scenarios: Vec<CampaignNode>,
    current_scenario_index: i32,
    mode: Mode,
    agents: Vec<ObjType>,
    last_battle_casualties: Vec<ObjType>,
    agent_info: HashMap<ObjType, AgentInfo>,
    actions: Vec<Action>,
    renown: Renown,
}

impl State {
    pub fn new(plan: Plan, agent_info: HashMap<ObjType, AgentInfo>) -> Self {
        assert!(!plan.nodes.is_empty(), "No scenarios");
        Self {
            current_scenario_index: 0,
            scenarios: plan.nodes,
            mode: Mode::PreparingForBattle,
            agents: plan.initial_agents,
            last_battle_casualties: Vec::new(),
            actions: Vec::new(),
            agent_info,
            renown: Renown(0),
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

    pub fn renown(&self) -> Renown {
        self.renown
    }

    pub fn available_actions(&self) -> &[Action] {
        &self.actions
    }

    pub fn execute_action(&mut self, action: Action) {
        assert_eq!(self.mode(), Mode::PreparingForBattle);
        assert!(utils::try_remove_item(&mut self.actions, &action));
        let cost = self.action_cost(&action);
        assert!(self.renown.0 >= cost.0);
        self.renown.0 -= cost.0;
        match action {
            Action::Recruit { agent_type } => {
                self.agents.push(agent_type);
            }
            Action::Upgrade { from, to } => {
                assert!(utils::try_remove_item(&mut self.agents, &from));
                self.agents.push(to);
            }
        }
    }

    pub fn action_cost(&self, action: &Action) -> Renown {
        match action {
            Action::Recruit { agent_type } => {
                let squad_size_penalty = self.agents.len() as i32;
                let agent_cost = self.agent_info[&agent_type].cost;
                Renown(agent_cost.0 + squad_size_penalty)
            }
            Action::Upgrade { from, to } => {
                let cost_from = self.agent_info[&from].cost;
                let cost_to = self.agent_info[&to].cost;
                Renown(cost_to.0 - cost_from.0)
            }
        }
    }

    pub fn report_battle_results(&mut self, result: &BattleResult) -> Result<(), ()> {
        if self.mode != Mode::PreparingForBattle {
            return Err(());
        }

        self.actions.clear();

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
            let award = &self.scenarios[i].award;
            self.renown.0 += award.renown.0;
            for recruit in &award.recruits {
                let action = Action::Recruit {
                    agent_type: recruit.clone(),
                };
                self.actions.push(action);
            }
            {
                let mut upgrade_candidates = Vec::new();
                for agent in &self.agents {
                    for (agent_type, agent_info) in &self.agent_info {
                        if agent_type == agent {
                            if let Some(upgrade) = agent_info.upgrades.choose(&mut zrng()) {
                                let from = agent.clone();
                                let to = upgrade.clone();
                                upgrade_candidates.push(Action::Upgrade { from, to });
                            }
                        }
                    }
                }
                let amount = 2;
                for action in upgrade_candidates.choose_multiple(&mut zrng(), amount) {
                    self.actions.push(action.clone());
                }
            }
            self.current_scenario_index += 1;
            self.mode = Mode::PreparingForBattle;
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
        campaign::{Action, AgentInfo, Award, CampaignNode, Mode, Plan, State},
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

    fn agent_info_empty() -> HashMap<ObjType, AgentInfo> {
        let mut m = HashMap::new();
        m.insert(
            "swordsman".into(),
            AgentInfo {
                upgrades: Vec::new(),
                cost: 10.into(),
            },
        );
        m.insert(
            "spearman".into(),
            AgentInfo {
                upgrades: Vec::new(),
                cost: 10.into(),
            },
        );
        m
    }

    fn agent_info_heavy_swordsman_upgrade() -> HashMap<ObjType, AgentInfo> {
        let mut m = HashMap::new();
        m.insert(
            "swordsman".into(),
            AgentInfo {
                upgrades: vec!["heavy_swordsman".into()],
                cost: 10.into(),
            },
        );
        m.insert(
            "heavy_swordsman".into(),
            AgentInfo {
                upgrades: Vec::new(),
                cost: 15.into(),
            },
        );
        m.insert(
            "spearman".into(),
            AgentInfo {
                upgrades: Vec::new(),
                cost: 10.into(),
            },
        );
        m.insert(
            "alchemist".into(),
            AgentInfo {
                upgrades: Vec::new(),
                cost: 10.into(),
            },
        );
        m
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
            let award = Award {
                recruits: vec![],
                renown: 10.into(),
            };
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
                    renown: 20.into(),
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
                award: Award {
                    recruits: Vec::new(),
                    renown: 20.into(),
                },
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
        let _state = State::new(empty_plan, agent_info_empty());
    }

    #[test]
    fn short_happy_path() {
        let mut state = State::new(campaign_plan_short(), agent_info_empty());
        assert!(state.available_actions().is_empty());
        assert_eq!(state.mode(), Mode::PreparingForBattle);
        let battle_result = BattleResult {
            winner_id: PlayerId(0),
            survivor_types: initial_agents(),
        };
        state.report_battle_results(&battle_result).unwrap();
        assert_eq!(state.mode(), Mode::Won);
    }

    #[test]
    fn short_fail_path() {
        let mut state = State::new(campaign_plan_short(), agent_info_empty());
        assert!(state.available_actions().is_empty());
        assert_eq!(state.mode(), Mode::PreparingForBattle);
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
        let mut state = State::new(campaign_plan_short(), agent_info_empty());
        let battle_result = BattleResult {
            winner_id: PlayerId(1),
            survivor_types: vec!["imp".into()],
        };
        assert!(state.report_battle_results(&battle_result).is_err());
    }

    #[test]
    fn bad_battle_win_no_survivors() {
        let mut state = State::new(campaign_plan_short(), agent_info_empty());
        let battle_result = BattleResult {
            winner_id: PlayerId(0),
            survivor_types: vec![],
        };
        assert!(state.report_battle_results(&battle_result).is_err());
    }

    #[test]
    fn recruit_and_casualty() {
        let mut state = State::new(campaign_plan_two_battles(), agent_info_empty());
        assert!(state.available_actions().is_empty());
        assert_eq!(state.mode(), Mode::PreparingForBattle);
        {
            let battle_result = BattleResult {
                winner_id: PlayerId(0),
                survivor_types: initial_agents(),
            };
            state.report_battle_results(&battle_result).unwrap();
        }
        assert_eq!(
            state.available_actions(),
            &[Action::Recruit {
                agent_type: "spearman".into()
            }]
        );
        assert!(state.last_battle_casualties().is_empty());
        assert_eq!(state.mode(), Mode::PreparingForBattle);
        state.execute_action(Action::Recruit {
            agent_type: "spearman".into(),
        });
        assert!(state.available_actions().is_empty());
        assert_eq!(state.mode(), Mode::PreparingForBattle);
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
        let mut state = State::new(
            campaign_plan_two_battles(),
            agent_info_heavy_swordsman_upgrade(),
        );
        assert!(state.available_actions().is_empty());
        assert_eq!(state.mode(), Mode::PreparingForBattle);
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
        let action_recruit = Action::Recruit {
            agent_type: "spearman".into(),
        };
        assert_eq!(
            state.available_actions(),
            &[action_recruit.clone(), action_upgrade.clone()]
        );
        assert!(state.last_battle_casualties().is_empty());
        assert_eq!(state.mode(), Mode::PreparingForBattle);
        state.execute_action(action_upgrade);
        assert_eq!(state.available_actions(), &[action_recruit]);
        assert_eq!(state.mode(), Mode::PreparingForBattle);
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
