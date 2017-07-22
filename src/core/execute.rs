use std::collections::{HashMap, VecDeque};
use rand::{thread_rng, Rng};
use core::map::PosHex;
use core::{State, Unit, PlayerId};
use core::command;
use core::command::Command;
use core::event;
use core::event::Event;
use core::effect::Effect;
use core::check::check;
use core::movement::MovePoints;

#[derive(Clone, Debug)]
pub struct Simulator {
    commands: VecDeque<Command>,
    events: VecDeque<Event>,
    // triggers: Vec<Trigger>, // TODO:
}

impl Simulator {
    pub fn new() -> Self {
        Self {
            commands: VecDeque::new(),
            events: VecDeque::new(),
        }
    }

    pub fn do_command(&mut self, state: &State, command: Command) {
        println!("Simulator: do_command: {:?}", command);
        assert!(self.commands.is_empty()); // TODO:
        let check_result = check(state, &command);
        if check_result.is_ok() {
            let events = self.execute(state, &command);
            self.events.extend(events);
        } else {
            println!("Error: {:?}", check_result)
        }
    }

    pub fn tick(&mut self) -> Option<Event> {
        if self.events.is_empty() {
            None
        } else {
            self.events.pop_front()
        }
    }

    fn execute(&mut self, state: &State, command: &Command) -> Vec<Event> {
        match *command {
            Command::Create(ref command) => self.execute_create(state, command),
            Command::MoveTo(ref command) => self.execute_move_to(state, command),
            Command::Attack(ref command) => self.execute_attack(state, command),
            Command::EndTurn(ref command) => self.execute_end_turn(state, command),
        }
    }

    fn execute_move_to(&mut self, _: &State, command: &command::MoveTo) -> Vec<Event> {
        let active_event = event::ActiveEvent::MoveTo(event::MoveTo {
            id: command.id,
            path: command.path.clone(),
        });
        let event = event::Event {
            active_event,
            effects: HashMap::new(),
        };
        vec![event]
    }

    fn execute_create(&mut self, _: &State, command: &command::Create) -> Vec<Event> {
        let active_event = event::ActiveEvent::Create(event::Create {
            id: command.id,
            unit: command.unit.clone(),
        });
        let event = event::Event {
            active_event,
            effects: HashMap::new(),
        };
        vec![event]
    }

    fn execute_attack(&mut self, _: &State, command: &command::Attack) -> Vec<Event> {
        let active_event = event::ActiveEvent::Attack(event::Attack {
            attacker_id: command.attacker_id,
            target_id: command.target_id,
        });
        let mut effects = HashMap::new();
        let effect = if thread_rng().gen_range(0, 6) > 3 {
            Effect::Kill
        } else {
            Effect::Miss
        };
        effects.insert(command.target_id, vec![effect]);
        let event = event::Event {
            active_event,
            effects,
        };
        vec![event]
    }

    fn execute_end_turn(&mut self, state: &State, _: &command::EndTurn) -> Vec<Event> {
        let player_id_old = state.player_id();
        let player_id_new = next_player_id(state);
        let event_end_turn = event::ActiveEvent::EndTurn(event::EndTurn {
            player_id: player_id_old,
        });
        let event_end_turn = event::Event {
            active_event: event_end_turn,
            effects: HashMap::new(),
        };
        let event_begin_turn = event::ActiveEvent::BeginTurn(event::BeginTurn {
            player_id: player_id_new,
        });
        let event_begin_turn = event::Event {
            active_event: event_begin_turn,
            effects: HashMap::new(),
        };
        vec![event_end_turn, event_begin_turn]
    }
}

fn next_player_id(state: &State) -> PlayerId {
    let current_player_id = PlayerId(state.player_id().0 + 1);
    if current_player_id.0 < state.players_count {
        current_player_id
    } else {
        PlayerId(0)
    }
}

pub fn create_objects(state: &mut State, simulator: &mut Simulator) {
    for &player_index in &[0, 1] {
        for i in 0..6 {
            let id = state.alloc_id();
            let active_event = event::ActiveEvent::Create(event::Create {
                id,
                unit: Unit {
                    pos: PosHex {
                        q: player_index,
                        r: -3 + i,
                    },
                    player_id: PlayerId(player_index),
                    move_points: MovePoints(6), // TODO: remove code duplication
                },
            });
            let event = event::Event {
                active_event,
                effects: HashMap::new(),
            };
            simulator.events.push_back(event);
        }
    }
}
