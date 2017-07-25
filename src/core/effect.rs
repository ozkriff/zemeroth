use core::{ObjId, State};

// TODO: subturns? EffectTime?
//
// #[derive(Clone, Debug, PartialEq)]
// pub enum Time {
//     Forever,
//     Turns(i32),
//     Instant,
// }
//
// #[derive(Clone, Debug)]
// pub struct TimedEffect {
//     pub time: Time,
//     pub effect: Effect,
// }

#[derive(Clone, Debug)]
pub enum Effect {
    Kill,
    Miss,
}

pub fn apply(state: &mut State, id: ObjId, effect: &Effect) {
    println!("effect::apply: {:?}", effect);
    match *effect {
        Effect::Kill => apply_kill(state, id),
        Effect::Miss => apply_miss(state, id),
    }
}

pub fn apply_kill(state: &mut State, id: ObjId) {
    state.units.remove(&id).unwrap();
}

pub fn apply_miss(_: &mut State, _: ObjId) {
    // TODO: ?
}
