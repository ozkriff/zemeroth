use serde::{Deserialize, Serialize};

use crate::core::tactical_map::{scenario::Scenario, state::BattleResult, PlayerId};

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
    pub recruits: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CampaignNode {
    pub scenario: Scenario,
    pub award: Award,
}

fn casualties(initial_agents: &[String], survivors: &[String]) -> Vec<String> {
    let mut agents = initial_agents.to_vec();
    for typename in survivors {
        if let Some(i) = agents.iter().position(|v| v == typename) {
            agents.remove(i);
        }
    }
    agents
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Plan {
    initial_agents: Vec<String>,
    nodes: Vec<CampaignNode>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct State {
    scenarios: Vec<CampaignNode>,
    current_scenario_index: i32,
    mode: Mode,

    /// Unittypes.
    agents: Vec<String>,

    last_battle_casualties: Vec<String>,
    recruits: Vec<String>,
}

impl State {
    pub fn from_plan(plan: Plan) -> Self {
        assert!(!plan.nodes.is_empty(), "No scenarios");
        Self {
            current_scenario_index: 0,
            scenarios: plan.nodes,
            mode: Mode::ReadyForBattle,
            agents: plan.initial_agents,
            last_battle_casualties: vec![],
            recruits: vec![],
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn last_battle_casualties(&self) -> &[String] {
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

    pub fn agents(&self) -> &[String] {
        &self.agents
    }

    pub fn recruit(&mut self, typename: String) {
        assert_eq!(self.mode(), Mode::PreparingForBattle);
        assert!(self.recruits.contains(&typename));
        self.agents.push(typename);
        self.recruits = Vec::new();
        self.mode = Mode::ReadyForBattle;
    }

    pub fn aviable_recruits(&self) -> &[String] {
        if self.mode != Mode::PreparingForBattle {
            assert!(self.recruits.is_empty());
        }
        &self.recruits
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
            self.recruits = self.scenarios[i].award.recruits.clone();

            self.current_scenario_index += 1;

            self.mode = Mode::PreparingForBattle;
            if self.aviable_recruits().is_empty() {
                // Skip the preparation step.
                self.mode = Mode::ReadyForBattle;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{
        campaign::{Award, CampaignNode, Mode, Plan, State},
        tactical_map::{
            scenario::{self, Line, Scenario},
            state::BattleResult,
            PlayerId,
        },
    };

    fn initial_agents() -> Vec<String> {
        vec!["swordsman".into(), "alchemist".into()]
    }

    fn campaign_plan_short() -> Plan {
        let initial_agents = initial_agents();
        let nodes = {
            let id_0 = Some(PlayerId(0));
            let id_1 = Some(PlayerId(1));
            let scenario = Scenario {
                objects: vec![
                    (None, "boulder", Line::Any, 3).into(),
                    (id_0, "swordsman", Line::Front, 1).into(),
                    (id_1, "imp", Line::Front, 2).into(),
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
                        (None, "boulder", Line::Any, 3).into(),
                        (id_0, "swordsman", Line::Front, 1).into(),
                        (id_1, "imp", Line::Front, 2).into(),
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
                        (None, "boulder", Line::Any, 3).into(),
                        (id_1, "imp", Line::Front, 4).into(),
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
        let _state = State::from_plan(empty_plan);
    }

    #[test]
    fn short_happy_path() {
        let mut state = State::from_plan(campaign_plan_short());
        assert!(state.aviable_recruits().is_empty());
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
        let mut state = State::from_plan(campaign_plan_short());
        assert!(state.aviable_recruits().is_empty());
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
        let mut state = State::from_plan(campaign_plan_short());
        let battle_result = BattleResult {
            winner_id: PlayerId(1),
            survivor_types: vec!["imp".into()],
        };
        assert!(state.report_battle_results(&battle_result).is_err());
    }

    #[test]
    fn bad_battle_win_no_survivors() {
        let mut state = State::from_plan(campaign_plan_short());
        let battle_result = BattleResult {
            winner_id: PlayerId(0),
            survivor_types: vec![],
        };
        assert!(state.report_battle_results(&battle_result).is_err());
    }

    #[test]
    fn upgrade() {
        let mut state = State::from_plan(campaign_plan_two_battles());
        assert!(state.aviable_recruits().is_empty());
        assert_eq!(state.mode(), Mode::ReadyForBattle);
        {
            let battle_result = BattleResult {
                winner_id: PlayerId(0),
                survivor_types: initial_agents(),
            };
            state.report_battle_results(&battle_result).unwrap();
        }
        assert_eq!(state.aviable_recruits(), &["spearman".to_string()]);
        assert!(state.last_battle_casualties().is_empty());
        assert_eq!(state.mode(), Mode::PreparingForBattle);
        state.recruit("spearman".into());
        assert!(state.aviable_recruits().is_empty());
        assert_eq!(state.mode(), Mode::ReadyForBattle);
        {
            let battle_result = BattleResult {
                winner_id: PlayerId(0),
                survivor_types: initial_agents(),
            };
            state.report_battle_results(&battle_result).unwrap();
        }
        assert_eq!(state.mode(), Mode::Won);
        assert_eq!(state.last_battle_casualties(), &["spearman".to_string()]);
    }
}
