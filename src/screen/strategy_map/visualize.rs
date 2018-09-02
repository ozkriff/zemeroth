use ggez::Context;
use scene::action::{self, Action, Boxed};

use core::strategy_map::{
    event::{self, Event},
    execute::ApplyPhase,
    State,
};
use geom;
use screen::strategy_map::view::View;
use utils::{seq, time_s};
use ZResult;

pub fn visualize(
    state: &State,
    view: &mut View,
    context: &mut Context,
    event: &Event,
    phase: ApplyPhase,
) -> ZResult<Box<dyn Action>> {
    debug!("visualize: phase={:?} event={:?}", phase, event);
    match phase {
        ApplyPhase::Pre => visualize_pre(state, view, context, event),
        ApplyPhase::Post => visualize_post(state, view, event),
    }
}

fn visualize_pre(
    state: &State,
    view: &mut View,
    context: &mut Context,
    event: &Event,
) -> ZResult<Box<dyn Action>> {
    let mut actions = Vec::new();
    actions.push(visualize_event(state, view, context, event)?);
    // for (&id, effects) in &event.instant_effects {
    //     for effect in effects {
    //         actions.push(visualize_instant_effect(state, view, context, id, effect)?);
    //     }
    // }
    // for (&id, effects) in &event.timed_effects {
    //     for effect in effects {
    //         actions.push(visualize_lasting_effect(state, view, context, id, effect)?);
    //     }
    // }
    Ok(seq(actions))
}

fn visualize_post(state: &State, view: &mut View, event: &Event) -> ZResult<Box<dyn Action>> {
    let mut actions = Vec::new();
    // for &id in &event.actor_ids {
    //     actions.push(refresh_brief_agent_info(state, view, id)?);
    // }
    // for &id in event.instant_effects.keys() {
    //     actions.push(refresh_brief_agent_info(state, view, id)?);
    // }
    // for &id in event.timed_effects.keys() {
    //     actions.push(refresh_brief_agent_info(state, view, id)?);
    // }
    Ok(seq(actions))
}

fn visualize_event(
    state: &State,
    view: &mut View,
    context: &mut Context,
    event: &Event,
) -> ZResult<Box<dyn Action>> {
    info!("{:?}", event);
    let action = match *event {
        // Event::Create => action::Empty::new().boxed(),
        Event::MoveTo(ref ev) => visualize_event_move_to(state, view, context, ev)?,
        Event::EndTurn(ref ev) => visualize_event_end_turn(state, view, context, ev),
        // Event::Attack(ref ev) => visualize_event_attack(state, view, context, ev)?,
        // Event::BeginTurn(ref ev) => visualize_event_begin_turn(state, view, context, ev)?,
        // Event::EffectTick(ref ev) => visualize_event_effect_tick(state, view, ev)?,
        // Event::EffectEnd(ref ev) => visualize_event_effect_end(state, view, context, ev)?,
        // Event::UseAbility(ref ev) => visualize_event_use_ability(state, view, context, ev)?,
    };
    Ok(action)
}

fn visualize_event_move_to(
    _: &State,
    view: &mut View,
    _: &mut Context,
    event: &event::MoveTo,
) -> ZResult<Box<dyn Action>> {
    let sprite = view.id_to_sprite(event.id).clone();
    // let sprite_shadow = view.id_to_shadow_sprite(event.id).clone();
    // let mut actions = Vec::new();
    // for step in event.path.steps() {}
    let from = geom::hex_to_point(view.tile_size(), event.from);
    let to = geom::hex_to_point(view.tile_size(), event.to);
    let diff = to - from;
    let step_height = 0.025;
    let step_time = time_s(0.13);
    let move_time = time_s(0.3);
    let action = action::MoveBy::new(&sprite, diff, move_time).boxed();
    Ok(action)
}

fn visualize_event_end_turn(
    _: &State,
    _: &mut View,
    _: &mut Context,
    _: &event::EndTurn,
) -> Box<dyn Action> {
    action::Sleep::new(time_s(0.2)).boxed()
}
