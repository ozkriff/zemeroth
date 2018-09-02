use core::{
    map::PosHex,
    strategy_map::{Id, State},
};

#[derive(Debug, Clone)]
pub enum Command {
    Create(Create),
    MoveTo(MoveTo),
    EndTurn(EndTurn),
}

// TODO: Create what?
#[derive(Debug, Clone)]
pub struct Create {
    // pub owner: Option<PlayerId>,
    pub pos: PosHex,
    pub prototype: String,
}

#[derive(Debug, Clone)]
pub struct MoveTo {
    pub id: Id,
    pub to: PosHex,
}

#[derive(Debug, Clone)]
pub struct EndTurn;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    SomeError, // TODO: Add some meaningful errors
}

pub fn check(state: &State, command: &Command) -> Result<(), Error> {
    match *command {
        Command::Create(ref command) => check_command_create(state, command),
        Command::MoveTo(ref command) => check_command_move_to(state, command),
        Command::EndTurn(ref command) => check_command_end_turn(state, command),
    }
}

fn check_command_create(_state: &State, _command: &Create) -> Result<(), Error> {
    unimplemented!() // TODO:
}

fn check_command_move_to(_state: &State, _command: &MoveTo) -> Result<(), Error> {
    // let agent = try_get_actor(state, command.id)?;
    // let agent_player_id = state.parts().belongs_to.get(command.id).0;
    // if agent_player_id != state.player_id() {
    //     return Err(Error::CanNotCommandEnemyAgents);
    // }
    // check_agent_can_move(state, command.id)?;
    // for step in command.path.steps() {
    //     check_is_inboard(state, step.to)?;
    //     check_not_blocked(state, step.to)?;
    // }
    // let cost = command.path.cost_for(state, command.id);
    // if cost > agent.move_points {
    //     return Err(Error::NotEnoughMovePoints);
    // }
    // Err(Error::SomeError)
    Ok(())
}

fn check_command_end_turn(_: &State, _: &EndTurn) -> Result<(), Error> {
    // Ok(())
    Err(Error::SomeError)
}
