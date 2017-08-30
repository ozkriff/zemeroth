use cgmath::vec2;
use hate::{Context, Sprite, Time};
use hate::scene::Action;
use hate::scene::action;
use hate::geom::Point;
use hate::gui;
use core::{ObjId, PlayerId, State};
use core::event::{ActiveEvent, Event};
use core::map::PosHex;
use core::event;
use core::effect::{self, Effect};
use core::execute::Phase;
use game_view::GameView;
use map;

pub fn message(view: &mut GameView, context: &mut Context, pos: PosHex, text: &str) -> Box<Action> {
    let visible = [0.0, 0.0, 0.0, 1.0];
    let invisible = [0.0, 0.0, 0.0, 0.0];
    let mut sprite = gui::text_sprite(context, text, 0.1);
    let point = map::hex_to_point(view.tile_size(), pos);
    let point = Point(point.0 + vec2(0.0, view.tile_size()));
    sprite.set_pos(point);
    sprite.set_color(invisible);
    let action_show_hide = Box::new(action::Sequence::new(vec![
        Box::new(action::Show::new(&view.layers().text, &sprite)),
        Box::new(action::ChangeColorTo::new(&sprite, visible, Time(0.3))),
        Box::new(action::Sleep::new(Time(1.0))),
        // TODO: read the time from Config:
        Box::new(action::ChangeColorTo::new(&sprite, invisible, Time(1.0))),
        Box::new(action::Hide::new(&view.layers().text, &sprite)),
    ]));
    let time = action_show_hide.duration();
    let delta = Point(vec2(0.0, 0.3));
    let action_move = Box::new(action::MoveBy::new(&sprite, delta, time));
    Box::new(action::Fork::new(Box::new(action::Sequence::new(vec![
        Box::new(action::Fork::new(action_move)),
        action_show_hide,
    ]))))
}

pub fn show_blood_spot(view: &mut GameView, context: &mut Context, at: PosHex) -> Box<Action> {
    let mut blood = Sprite::from_path(context, "blood.png", view.tile_size() * 2.0);
    blood.set_color([1.0, 1.0, 1.0, 0.0]);
    let mut point = map::hex_to_point(view.tile_size(), at);
    point.0.y -= view.tile_size() * 0.5;
    blood.set_pos(point);
    let color_final = [1.0, 1.0, 1.0, 0.3];
    Box::new(action::Sequence::new(vec![
        Box::new(action::Show::new(&view.layers().blood, &blood)),
        Box::new(action::ChangeColorTo::new(&blood, color_final, Time(0.3))),
    ]))
}

fn remove_brief_unit_info(view: &mut GameView, id: ObjId) -> Box<Action> {
    let mut actions: Vec<Box<Action>> = Vec::new();
    let sprites = view.unit_info_get(id);
    for sprite in sprites {
        let mut color = sprite.color();
        color[3] = 0.0;
        actions.push(Box::new(
            action::Fork::new(Box::new(action::Sequence::new(vec![
                Box::new(action::ChangeColorTo::new(&sprite, color, Time(0.4))),
                Box::new(action::Hide::new(&view.layers().text, &sprite)),
            ]))),
        ));
    }
    let action = Box::new(action::Sequence::new(actions));
    action
}

fn generate_brief_unit_info(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    id: ObjId,
) -> Box<Action> {
    let mut actions: Vec<Box<Action>> = Vec::new();
    let unit = state.unit(id);
    let size = 0.02;
    let mut point = map::hex_to_point(view.tile_size(), unit.pos);
    point.0.x += view.tile_size() * 0.8;
    point.0.y += view.tile_size() * 0.6;
    let mut dots = Vec::new();
    let base_x = point.0.x;
    for &(color, n) in &[
        ([0.0, 0.4, 0.0, 1.0], unit.strength.0),
        ([1.0, 0.1, 1.0, 1.0], unit.jokers.0),
        ([1.0, 0.0, 0.0, 1.0], unit.attacks.0),
        ([0.0, 0.0, 1.0, 1.0], unit.moves.0),
    ] {
        for _ in 0..n {
            dots.push((color, point));
            point.0.x -= size;
        }
        point.0.x = base_x;
        point.0.y -= size;
    }
    let mut sprites = Vec::new();
    for &(color, point) in &dots {
        let mut sprite = Sprite::from_path(context, "white_hex.png", size);
        sprite.set_pos(point);
        sprite.set_color([color[0], color[1], color[2], 0.0]);
        let action = Box::new(action::Fork::new(Box::new(action::Sequence::new(vec![
            Box::new(action::Show::new(&view.layers().text, &sprite)),
            Box::new(action::ChangeColorTo::new(&sprite, color, Time(0.1))),
        ]))));
        sprites.push(sprite);
        actions.push(action);
    }
    view.unit_info_set(id, sprites);
    let action = Box::new(action::Sequence::new(actions));
    action
}

pub fn showhide_brief_unit_info(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    id: ObjId,
) -> Box<Action> {
    let mut actions = Vec::new();
    if view.unit_info_check(id) {
        actions.push(remove_brief_unit_info(view, id));
    }
    if state.unit_opt(id).is_some() {
        actions.push(generate_brief_unit_info(state, view, context, id));
    }
    Box::new(action::Sequence::new(actions))
}

pub fn visualize(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    event: &Event,
    phase: Phase,
) -> Box<Action> {
    match phase {
        Phase::Pre => visualize_pre(state, view, context, event),
        Phase::Post => visualize_post(state, view, context, event),
    }
}

pub fn visualize_pre(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    event: &Event,
) -> Box<Action> {
    let mut actions = Vec::new();
    actions.push(visualize_event(state, view, context, &event.active_event));
    for (&target_id, effects) in &event.effects {
        for effect in effects {
            actions.push(visualize_effect(state, view, context, target_id, effect));
        }
    }
    Box::new(action::Sequence::new(actions))
}

pub fn visualize_post(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    event: &Event,
) -> Box<Action> {
    let mut actions = Vec::new();
    for &id in &event.actor_ids {
        actions.push(showhide_brief_unit_info(state, view, context, id));
    }
    for &id in event.effects.keys() {
        actions.push(showhide_brief_unit_info(state, view, context, id));
    }
    Box::new(action::Sequence::new(actions))
}

pub fn visualize_event(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    event: &ActiveEvent,
) -> Box<Action> {
    match *event {
        ActiveEvent::Create(ref event) => visualize_event_create(state, view, context, event),
        ActiveEvent::MoveTo(ref event) => visualize_event_move_to(state, view, context, event),
        ActiveEvent::Attack(ref event) => visualize_event_attack(state, view, context, event),
        ActiveEvent::EndTurn(ref event) => visualize_event_end_turn(state, view, context, event),
        ActiveEvent::BeginTurn(ref ev) => visualize_event_begin_turn(state, view, context, ev),
    }
}

fn visualize_event_create(
    _: &State,
    view: &mut GameView,
    context: &mut Context,
    event: &event::Create,
) -> Box<Action> {
    let point = map::hex_to_point(view.tile_size(), event.unit.pos);
    let sprite_name = match event.unit.unit_type.name.as_str() {
        "swordsman" => "swordsman.png",
        "spearman" => "spearman.png",
        "imp" => "imp.png",
        _ => unimplemented!(),
    };
    let size = view.tile_size() * 2.0;
    let mut sprite = Sprite::from_path(context, sprite_name, size);
    sprite.set_color([1.0, 1.0, 1.0, 0.0]);
    sprite.set_pos(point);
    view.add_object(event.id, &sprite);
    Box::new(action::Sequence::new(vec![
        Box::new(action::Show::new(&view.layers().fg, &sprite)),
        Box::new(action::ChangeColorTo::new(
            &sprite,
            [1.0, 1.0, 1.0, 1.0],
            Time(0.25),
        )),
    ]))
}

fn visualize_event_move_to(
    _: &State,
    view: &mut GameView,
    _: &mut Context,
    event: &event::MoveTo,
) -> Box<Action> {
    let sprite = view.id_to_sprite(event.id).clone();
    let mut actions: Vec<Box<Action>> = Vec::new();
    for step in event.path.steps() {
        let from = map::hex_to_point(view.tile_size(), step.from);
        let to = map::hex_to_point(view.tile_size(), step.to);
        let diff = Point(to.0 - from.0);
        actions.push(Box::new(action::MoveBy::new(&sprite, diff, Time(0.3))));
    }
    Box::new(action::Sequence::new(actions))
}

fn visualize_event_attack(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    event: &event::Attack,
) -> Box<Action> {
    let sprite = view.id_to_sprite(event.attacker_id).clone();
    let map_to = state.unit(event.target_id).pos;
    let to = map::hex_to_point(view.tile_size(), map_to);
    let map_from = state.unit(event.attacker_id).pos;
    let from = map::hex_to_point(view.tile_size(), map_from);
    let diff = Point((to.0 - from.0) / 2.0);
    let mut actions: Vec<Box<Action>> = Vec::new();
    actions.push(Box::new(action::Sleep::new(Time(0.1)))); // TODO: ??
    if event.mode == event::AttackMode::Reactive {
        actions.push(Box::new(action::Sleep::new(Time(0.3)))); // TODO: ??
        actions.push(message(view, context, map_from, "reaction"));
    }
    actions.push(Box::new(action::MoveBy::new(&sprite, diff, Time(0.15))));
    actions.push(Box::new(
        action::MoveBy::new(&sprite, Point(-diff.0), Time(0.15)),
    ));
    actions.push(Box::new(action::Sleep::new(Time(0.1)))); // TODO: ??
    Box::new(action::Sequence::new(actions))
}

fn visualize_event_end_turn(
    _: &State,
    _: &mut GameView,
    _: &mut Context,
    _: &event::EndTurn,
) -> Box<Action> {
    Box::new(action::Sleep::new(Time(0.2)))
}

fn visualize_event_begin_turn(
    _: &State,
    view: &mut GameView,
    context: &mut Context,
    event: &event::BeginTurn,
) -> Box<Action> {
    let visible = [0.0, 0.0, 0.0, 1.0];
    let invisible = [0.0, 0.0, 0.0, 0.0];
    let text = match event.player_id {
        PlayerId(0) => "YOUR TURN",
        PlayerId(1) => "ENEMY TURN",
        _ => unreachable!(),
    };
    let mut sprite = gui::text_sprite(context, text, 0.2);
    sprite.set_pos(Point(vec2(0.0, 0.0)));
    sprite.set_color(invisible);
    Box::new(action::Sequence::new(vec![
        Box::new(action::Show::new(&view.layers().text, &sprite)),
        Box::new(action::ChangeColorTo::new(&sprite, visible, Time(0.2))),
        Box::new(action::Sleep::new(Time(1.5))),
        Box::new(action::ChangeColorTo::new(&sprite, invisible, Time(0.3))),
        Box::new(action::Hide::new(&view.layers().text, &sprite)),
    ]))
}

pub fn visualize_effect(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    target_id: ObjId,
    effect: &Effect,
) -> Box<Action> {
    match *effect {
        Effect::Kill => visualize_effect_kill(state, view, context, target_id),
        Effect::Wound(ref effect) => {
            visualize_effect_wound(state, view, context, target_id, effect)
        }
        Effect::Miss => visualize_effect_miss(state, view, context, target_id),
    }
}

fn visualize_effect_kill(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    target_id: ObjId,
) -> Box<Action> {
    let pos = state.unit(target_id).pos;
    let sprite = view.id_to_sprite(target_id).clone();
    view.remove_object(target_id);
    let dark = [0.1, 0.1, 0.1, 1.0];
    let invisible = [0.1, 0.1, 0.1, 0.0];
    Box::new(action::Sequence::new(vec![
        message(view, context, pos, "killed"),
        Box::new(action::Sleep::new(Time(0.25))),
        show_blood_spot(view, context, pos),
        Box::new(action::ChangeColorTo::new(&sprite, dark, Time(0.2))),
        Box::new(action::ChangeColorTo::new(&sprite, invisible, Time(0.2))),
        Box::new(action::Hide::new(&view.layers().fg, &sprite)),
    ]))
}

fn visualize_effect_wound(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    target_id: ObjId,
    effect: &effect::Wound,
) -> Box<Action> {
    let pos = state.unit(target_id).pos;
    let damage = effect.0;
    let sprite = view.id_to_sprite(target_id).clone();
    let color_normal = sprite.color();
    let color_dark = [0.1, 0.1, 0.1, 1.0];
    Box::new(action::Sequence::new(vec![
        message(view, context, pos, &format!("wounded - {}", damage.0)),
        Box::new(action::ChangeColorTo::new(&sprite, color_dark, Time(0.2))),
        Box::new(action::ChangeColorTo::new(&sprite, color_normal, Time(0.2))),
        show_blood_spot(view, context, pos),
    ]))
}

fn visualize_effect_miss(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    target_id: ObjId,
) -> Box<Action> {
    let pos = state.unit(target_id).pos;
    message(view, context, pos, "missed")
}
