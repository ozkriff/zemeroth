use core::{ObjId, State, Strength};

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
    Wound(Wound),
    Miss,
}

#[derive(Clone, Debug)]
pub struct Wound(pub Strength);

pub fn apply(state: &mut State, id: ObjId, effect: &Effect) {
    debug!("effect::apply: {:?}", effect);
    match *effect {
        Effect::Kill => apply_kill(state, id),
        Effect::Wound(ref effect) => apply_wound(state, id, effect),
        Effect::Miss => apply_miss(state, id),
    }
}

pub fn apply_kill(state: &mut State, id: ObjId) {
    state.units.remove(&id).unwrap();
}

pub fn apply_wound(state: &mut State, id: ObjId, effect: &Wound) {
    let damage = effect.0;
    let unit = state.units.get_mut(&id).unwrap();
    unit.strength.0 -= damage.0;
    assert!(unit.strength.0 > 0);
}

pub fn apply_miss(_: &mut State, _: ObjId) {
    // TODO: ?
}
