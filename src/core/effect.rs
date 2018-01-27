use core::Strength;

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
