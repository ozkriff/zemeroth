use core::strategy_map::{
    command::{self, check, Command, Error},
    // component::{self/*, Component*/},
    event::Event,
    /*Id,*/ State,
};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ApplyPhase {
    Pre,
    Post,
}

/// A callback for visualization of the events.
type Cb<'c> = &'c mut dyn FnMut(&State, &Event, ApplyPhase);

pub fn execute(state: &mut State, command: &Command, cb: Cb) -> Result<(), Error> {
    debug!("execute: {:?}", command);
    if let Err(err) = check(state, command) {
        error!("Check failed: {:?}", err);
        return Err(err);
    }
    match *command {
        Command::Create(ref command) => execute_create(state, cb, command),
        Command::MoveTo(ref command) => execute_move_to(state, cb, command),
        Command::EndTurn(ref command) => execute_end_turn(state, cb, command),
    }
    Ok(())
}

fn do_event(state: &mut State, cb: Cb, event: &Event) {
    cb(state, event, ApplyPhase::Pre);
    apply(state, event);
    cb(state, event, ApplyPhase::Post);
}

fn execute_create(_state: &mut State, _cb: Cb, _command: &command::Create) {
    // let mut components = state.prototype_for(&command.prototype);
    // if let Some(player_id) = command.owner {
    //     components.push(Component::BelongsTo(component::BelongsTo(player_id)));
    // }
    // let name = command.prototype.clone();
    // components.extend_from_slice(&[
    //     Component::Pos(component::Pos(command.pos)),
    //     Component::Meta(component::Meta { name }),
    // ]);
    // let id = state.alloc_id();

    // let mut instant_effects = HashMap::new();
    // let effect_create = Effect::Create(effect::Create {
    //     pos: command.pos,
    //     prototype: command.prototype.clone(),
    //     components,
    // });
    // instant_effects.insert(id, vec![effect_create]);

    // let event = Event {
    //     active_event: ActiveEvent::Create,
    //     actor_ids: vec![id],
    //     instant_effects,
    //     timed_effects: HashMap::new(),
    // };
    // do_event(state, cb, &event);

    unimplemented!() // TODO
}

fn execute_move_to(_state: &mut State, _cb: Cb, _command: &command::MoveTo) {
    unimplemented!() // TODO:
}

fn execute_end_turn(_state: &mut State, _cb: Cb, _command: &command::EndTurn) {
    unimplemented!() // TODO:
}
