use hate::{Sprite, Context, Time};
use hate::scene::Action;
use hate::scene::action;
use hate::geom::Point;
use core::{State, PlayerId, ObjId};
use core::event::{Event, ActiveEvent};
use core::event;
use core::effect::Effect;
use game_view::GameView;
use map;

pub fn visualize(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    event: &Event,
) -> (Box<Action>, Time) {
    let mut total_duration = Time(0.0);
    let mut actions: Vec<Box<Action>> = Vec::new();
    let (event_action, event_time) = visualize_event(state, view, context, &event.active_event);
    total_duration.0 += event_time.0;
    actions.push(event_action);
    for (&target_id, effects) in &event.effects {
        for effect in effects {
            let (action, duration) = visualize_effect(state, view, context, target_id, effect);
            total_duration.0 += duration.0;
            actions.push(action);
        }
    }
    let mut forked_actions: Vec<Box<Action>> = Vec::new();
    for action in actions {
        forked_actions.push(Box::new(action::Fork::new(action)));
    }
    let final_action = Box::new(action::Sequence::new(forked_actions));
    (final_action, total_duration)
}

pub fn visualize_event(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    event: &ActiveEvent,
) -> (Box<Action>, Time) {
    match *event {
        ActiveEvent::Create(ref event) => visualize_event_create(state, view, context, event),
        ActiveEvent::MoveTo(ref event) => visualize_event_move_to(state, view, context, event),
        ActiveEvent::Attack(ref event) => visualize_event_attack(state, view, context, event),
    }
}

fn visualize_event_create(
    _: &State,
    view: &mut GameView,
    context: &mut Context,
    event: &event::Create,
) -> (Box<Action>, Time) {
    let point = map::hex_to_point(view.tile_size(), event.unit.pos);
    let sprite_name = match event.unit.player_id {
        PlayerId(0) => "swordsman.png",
        PlayerId(1) => "imp.png",
        _ => unimplemented!(),
    };
    let mut sprite = Sprite::from_path(context, sprite_name, 0.2);
    sprite.set_color([1.0, 1.0, 1.0, 0.0]);
    sprite.set_pos(point);
    view.add_object(event.id, &sprite);
    let action = Box::new(action::Sequence::new(vec![
        Box::new(action::Show::new(&view.layers().fg, &sprite)),
        Box::new(action::ChangeColorTo::new(
            &sprite,
            [1.0, 1.0, 1.0, 1.0],
            Time(0.5),
        )),
    ]));
    let time = action.duration();
    (action, time)
}

fn visualize_event_move_to(
    _: &State,
    view: &mut GameView,
    _: &mut Context,
    event: &event::MoveTo,
) -> (Box<Action>, Time) {
    let sprite = view.id_to_sprite(event.id).clone();
    let mut actions: Vec<Box<Action>> = Vec::new();
    // TODO: add Path struct with `iter` method returning
    // special `Edge{from, to}` iterator
    for window in event.path.windows(2) {
        let from = map::hex_to_point(view.tile_size(), window[0]);
        let to = map::hex_to_point(view.tile_size(), window[1]);
        let diff = Point(to.0 - from.0);
        actions.push(Box::new(action::MoveBy::new(&sprite, diff, Time(0.3))));
    }
    let action = Box::new(action::Sequence::new(actions));
    let time = action.duration();
    (action, time)
}

fn visualize_event_attack(
    state: &State,
    view: &mut GameView,
    _: &mut Context,
    event: &event::Attack,
) -> (Box<Action>, Time) {
    let sprite = view.id_to_sprite(event.attacker_id).clone();
    let map_to = state.unit(event.target_id).pos;
    let to = map::hex_to_point(view.tile_size(), map_to);
    let from = sprite.pos();
    let diff = Point((to.0 - from.0) / 2.0);
    let action = Box::new(action::Sequence::new(vec![
        Box::new(action::MoveBy::new(&sprite, diff, Time(0.15))),
        Box::new(action::MoveBy::new(&sprite, Point(-diff.0), Time(0.15))),
    ]));
    let time = action.duration();
    (action, time)
}

pub fn visualize_effect(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    target_id: ObjId,
    effect: &Effect,
) -> (Box<Action>, Time) {
    match *effect {
        Effect::Kill => visualize_effect_kill(state, view, context, target_id),
    }
}

fn visualize_effect_kill(
    _: &State,
    view: &mut GameView,
    _: &mut Context,
    target_id: ObjId,
) -> (Box<Action>, Time) {
    let sprite = view.id_to_sprite(target_id).clone();
    view.remove_object(target_id);
    let color = [1.0, 1.0, 1.0, 0.0];
    let action = Box::new(action::Sequence::new(vec![
        Box::new(action::Sleep::new(Time(0.25))),
        Box::new(action::ChangeColorTo::new(&sprite, color, Time(0.10))),
        Box::new(action::Hide::new(&view.layers().fg, &sprite)),
    ]));
    let time = action.duration();
    (action, time)
}
