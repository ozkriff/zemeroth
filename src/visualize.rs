use std::time::Duration;
use cgmath::{InnerSpace, vec2};
use hate::{Context, Sprite};
use hate::scene::Action;
use hate::scene::action;
use hate::geom::Point;
use hate::gui;
use core::{ObjId, PlayerId, State};
use core::event::{ActiveEvent, Event};
use core::map::PosHex;
use core::event;
use core::effect::{self, Effect, LastingEffect, TimedEffect};
use core::execute::ApplyPhase;
use core::ability::Ability;
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
        Box::new(action::ChangeColorTo::new(&sprite, visible, Duration::from_millis(300))),
        Box::new(action::Sleep::new(Duration::from_millis(1_000))),
        // TODO: read the time from Config:
        Box::new(action::ChangeColorTo::new(&sprite, invisible, Duration::from_millis(1_000))),
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

fn show_blood_spot(view: &mut GameView, context: &mut Context, at: PosHex) -> Box<Action> {
    let mut blood = Sprite::from_path(context, "blood.png", view.tile_size() * 2.0);
    blood.set_color([1.0, 1.0, 1.0, 0.0]);
    let mut point = map::hex_to_point(view.tile_size(), at);
    point.0.y -= view.tile_size() * 0.5;
    blood.set_pos(point);
    let color_final = [1.0, 1.0, 1.0, 0.3];
    Box::new(action::Sequence::new(vec![
        Box::new(action::Show::new(&view.layers().blood, &blood)),
        Box::new(action::ChangeColorTo::new(&blood, color_final, Duration::from_millis(300))),
    ]))
}

fn show_flare_scale(
    view: &mut GameView,
    context: &mut Context,
    at: PosHex,
    color: [f32; 4],
    scale: f32,
) -> Box<Action> {
    let visible = color;
    let mut invisible = visible;
    invisible[3] = 0.0;
    let size = view.tile_size() * 2.0 * scale;
    let mut flare = Sprite::from_path(context, "white_hex.png", size);
    let point = map::hex_to_point(view.tile_size(), at);
    flare.set_pos(point);
    flare.set_color(invisible);
    Box::new(action::Sequence::new(vec![
        Box::new(action::Show::new(&view.layers().flares, &flare)),
        Box::new(action::ChangeColorTo::new(&flare, visible, Duration::from_millis(100))),
        Box::new(action::ChangeColorTo::new(&flare, invisible, Duration::from_millis(300))),
        Box::new(action::Hide::new(&view.layers().flares, &flare)),
    ]))
}

fn show_flare(
    view: &mut GameView,
    context: &mut Context,
    at: PosHex,
    color: [f32; 4],
) -> Box<Action> {
    show_flare_scale(view, context, at, color, 1.0)
}

fn up_and_down_move(_: &mut GameView, sprite: &Sprite, height: f32, time: Duration) -> Box<Action> {
    let duration_0_25 = time / 4;
    let up_fast = Point(vec2(0.0, height * 0.75));
    let up_slow = Point(vec2(0.0, height * 0.25));
    let down_slow = Point(vec2(0.0, -height * 0.25));
    let down_fast = Point(vec2(0.0, -height * 0.75));
    Box::new(action::Sequence::new(vec![
        Box::new(action::MoveBy::new(sprite, up_fast, duration_0_25)),
        Box::new(action::MoveBy::new(sprite, up_slow, duration_0_25)),
        Box::new(action::MoveBy::new(sprite, down_slow, duration_0_25)),
        Box::new(action::MoveBy::new(sprite, down_fast, duration_0_25)),
    ]))
}

fn arc_move(view: &mut GameView, sprite: &Sprite, diff: Point) -> Box<Action> {
    let len = diff.0.magnitude();
    let min_height = view.tile_size() * 0.5;
    let base_height = view.tile_size() * 2.0;
    let min_time = 0.2;
    let base_time = 0.3;
    let height = min_height + base_height * (len / 1.0);
    let time_f = min_time + base_time * (len / 1.0);
    let time = Duration::from_millis((time_f * 1_000.0) as _);
    let up_and_down = up_and_down_move(view, sprite, height, time);
    let main_move = Box::new(action::MoveBy::new(sprite, diff, time));
    Box::new(action::Sequence::new(vec![
        Box::new(action::Fork::new(main_move)),
        up_and_down,
    ]))
}

fn vanish(view: &mut GameView, target_id: ObjId) -> Box<Action> {
    debug!("vanish target_id={:?}", target_id);
    let sprite = view.id_to_sprite(target_id).clone();
    view.remove_object(target_id);
    let dark = [0.1, 0.1, 0.1, 1.0];
    let invisible = [0.1, 0.1, 0.1, 0.0];
    Box::new(action::Sequence::new(vec![
        Box::new(action::Sleep::new(Duration::from_millis(250))),
        Box::new(action::ChangeColorTo::new(&sprite, dark, Duration::from_millis(200))),
        Box::new(action::ChangeColorTo::new(&sprite, invisible, Duration::from_millis(200))),
        Box::new(action::Hide::new(&view.layers().units, &sprite)),
    ]))
}

fn remove_brief_unit_info(view: &mut GameView, id: ObjId) -> Box<Action> {
    let mut actions: Vec<Box<Action>> = Vec::new();
    let sprites = view.unit_info_get(id);
    for sprite in sprites {
        let mut color = sprite.color();
        color[3] = 0.0;
        actions.push(Box::new(action::Fork::new(Box::new(
            action::Sequence::new(vec![
                Box::new(action::ChangeColorTo::new(&sprite, color, Duration::from_millis(400))),
                Box::new(action::Hide::new(&view.layers().dots, &sprite)),
            ]),
        ))));
    }
    Box::new(action::Sequence::new(actions))
}

fn generate_brief_obj_info(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    id: ObjId,
) -> Box<Action> {
    let mut actions: Vec<Box<Action>> = Vec::new();
    let agent = state.parts().agent.get(id);
    let obj_pos = state.parts().pos.get(id).0;
    let strength = state.parts().strength.get(id);
    let size = 0.2 * view.tile_size();
    let mut point = map::hex_to_point(view.tile_size(), obj_pos);
    point.0.x += view.tile_size() * 0.8;
    point.0.y += view.tile_size() * 0.6;
    let mut dots = Vec::new();
    let base_x = point.0.x;
    for &(color, n) in &[
        ([0.0, 0.4, 0.0, 1.0], strength.strength.0),
        ([1.0, 0.1, 1.0, 1.0], agent.jokers.0),
        ([1.0, 0.0, 0.0, 1.0], agent.attacks.0),
        ([0.0, 0.0, 1.0, 1.0], agent.moves.0),
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
            Box::new(action::Show::new(&view.layers().dots, &sprite)),
            Box::new(action::ChangeColorTo::new(&sprite, color, Duration::from_millis(100))),
        ]))));
        sprites.push(sprite);
        actions.push(action);
    }
    view.unit_info_set(id, sprites);
    Box::new(action::Sequence::new(actions))
}

pub fn refresh_brief_unit_info(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    id: ObjId,
) -> Box<Action> {
    let mut actions = Vec::new();
    if view.unit_info_check(id) {
        actions.push(remove_brief_unit_info(view, id));
    }
    if state.parts().agent.get_opt(id).is_some() {
        actions.push(generate_brief_obj_info(state, view, context, id));
    }
    Box::new(action::Sequence::new(actions))
}

pub fn visualize(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    event: &Event,
    phase: ApplyPhase,
) -> Box<Action> {
    debug!("visualize: phase={:?} event={:?}", phase, event);
    match phase {
        ApplyPhase::Pre => visualize_pre(state, view, context, event),
        ApplyPhase::Post => visualize_post(state, view, context, event),
    }
}

fn visualize_pre(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    event: &Event,
) -> Box<Action> {
    let mut actions = Vec::new();
    actions.push(visualize_event(state, view, context, &event.active_event));
    for (&id, effects) in &event.instant_effects {
        for effect in effects {
            let action = visualize_instant_effect(state, view, context, id, effect);
            actions.push(Box::new(action::Fork::new(action)));
        }
    }
    for (&id, effects) in &event.timed_effects {
        for effect in effects {
            actions.push(visualize_lasting_effect(state, view, context, id, effect));
        }
    }
    Box::new(action::Sequence::new(actions))
}

fn visualize_post(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    event: &Event,
) -> Box<Action> {
    let mut actions = Vec::new();
    for &id in &event.actor_ids {
        actions.push(refresh_brief_unit_info(state, view, context, id));
    }
    for &id in event.instant_effects.keys() {
        actions.push(refresh_brief_unit_info(state, view, context, id));
    }
    for &id in event.timed_effects.keys() {
        actions.push(refresh_brief_unit_info(state, view, context, id));
    }
    Box::new(action::Sequence::new(actions))
}

fn visualize_event(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    event: &ActiveEvent,
) -> Box<Action> {
    match *event {
        ActiveEvent::Create => Box::new(action::Sleep::new(Duration::from_millis(0))),
        ActiveEvent::MoveTo(ref ev) => visualize_event_move_to(state, view, context, ev),
        ActiveEvent::Attack(ref ev) => visualize_event_attack(state, view, context, ev),
        ActiveEvent::EndTurn(ref ev) => visualize_event_end_turn(state, view, context, ev),
        ActiveEvent::BeginTurn(ref ev) => visualize_event_begin_turn(state, view, context, ev),
        ActiveEvent::EffectTick(ref ev) => visualize_event_effect_tick(state, view, context, ev),
        ActiveEvent::EffectEnd(ref ev) => visualize_event_effect_end(state, view, context, ev),
        ActiveEvent::UseAbility(ref ev) => visualize_event_use_ability(state, view, context, ev),
        ActiveEvent::UsePassiveAbility(ref ev) => {
            visualize_event_use_passive_ability(state, view, context, ev)
        }
    }
}

fn visualize_create(
    view: &mut GameView,
    context: &mut Context,
    id: ObjId,
    pos: PosHex,
    prototype: &str,
) -> Box<Action> {
    let point = map::hex_to_point(view.tile_size(), pos);
    // TODO: Move to some .ron config:
    let sprite_name = match prototype {
        "swordsman" => "swordsman.png",
        "spearman" => "spearman.png",
        "hammerman" => "hammerman.png",
        "alchemist" => "alchemist.png",
        "imp" => "imp.png",
        "imp_toxic" => "imp_toxic.png",
        "imp_bomber" => "imp_bomber.png",
        "imp_summoner" => "imp_summoner.png",
        "boulder" => "boulder.png",
        "bomb" => "bomb.png",
        "bomb_fire" => "bomb_fire.png",
        "bomb_poison" => "bomb_poison.png",
        "fire" => "fire.png",
        "poison_cloud" => "poison_cloud.png",
        _ => unimplemented!(),
    };
    let size = view.tile_size() * 2.0;
    let mut sprite = Sprite::from_path(context, sprite_name, size);
    sprite.set_color([1.0, 1.0, 1.0, 0.0]);
    sprite.set_pos(point);
    view.add_object(id, &sprite);
    let final_color = [1.0, 1.0, 1.0, 1.0];
    Box::new(action::Sequence::new(vec![
        Box::new(action::Show::new(&view.layers().units, &sprite)),
        Box::new(action::ChangeColorTo::new(&sprite, final_color, Duration::from_millis(250))),
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
        let step_height = 0.025;
        let step_time = Duration::from_millis(130);
        let main_move = Box::new(action::MoveBy::new(&sprite, diff, Duration::from_millis(300)));
        let action = Box::new(action::Sequence::new(vec![
            Box::new(action::Fork::new(main_move)),
            up_and_down_move(view, &sprite, step_height, step_time),
            up_and_down_move(view, &sprite, step_height, step_time),
        ]));
        actions.push(action);
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
    let map_to = state.parts().pos.get(event.target_id).0;
    let to = map::hex_to_point(view.tile_size(), map_to);
    let map_from = state.parts().pos.get(event.attacker_id).0;
    let from = map::hex_to_point(view.tile_size(), map_from);
    let diff = Point((to.0 - from.0) / 2.0);
    let mut actions: Vec<Box<Action>> = Vec::new();
    actions.push(Box::new(action::Sleep::new(Duration::from_millis(100))));
    if event.mode == event::AttackMode::Reactive {
        actions.push(Box::new(action::Sleep::new(Duration::from_millis(300))));
        actions.push(message(view, context, map_from, "reaction"));
    }
    actions.push(Box::new(action::MoveBy::new(&sprite, diff, Duration::from_millis(100))));
    actions.push(Box::new(action::MoveBy::new(
        &sprite,
        Point(-diff.0),
        Duration::from_millis(150),
    )));
    actions.push(Box::new(action::Sleep::new(Duration::from_millis(100))));
    Box::new(action::Sequence::new(actions))
}

fn visualize_event_end_turn(
    _: &State,
    _: &mut GameView,
    _: &mut Context,
    _: &event::EndTurn,
) -> Box<Action> {
    Box::new(action::Sleep::new(Duration::from_millis(200)))
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
        Box::new(action::ChangeColorTo::new(&sprite, visible, Duration::from_millis(200))),
        Box::new(action::Sleep::new(Duration::from_millis(1_000))),
        Box::new(action::ChangeColorTo::new(&sprite, invisible, Duration::from_millis(300))),
        Box::new(action::Hide::new(&view.layers().text, &sprite)),
    ]))
}

fn visualize_event_use_ability_jump(
    state: &State,
    view: &mut GameView,
    _: &mut Context,
    event: &event::UseAbility,
) -> Box<Action> {
    let sprite = view.id_to_sprite(event.id).clone();
    let from = state.parts().pos.get(event.id).0;
    let from = map::hex_to_point(view.tile_size(), from);
    let to = map::hex_to_point(view.tile_size(), event.pos);
    let diff = Point(to.0 - from.0);
    arc_move(view, &sprite, diff)
}

fn visualize_event_use_ability_dash(
    state: &State,
    view: &mut GameView,
    _: &mut Context,
    event: &event::UseAbility,
) -> Box<Action> {
    let sprite = view.id_to_sprite(event.id).clone();
    let from = state.parts().pos.get(event.id).0;
    let from = map::hex_to_point(view.tile_size(), from);
    let to = map::hex_to_point(view.tile_size(), event.pos);
    let diff = Point(to.0 - from.0);
    Box::new(action::MoveBy::new(&sprite, diff, Duration::from_millis(100)))
}

fn visualize_event_use_ability_explode(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    event: &event::UseAbility,
) -> Box<Action> {
    let pos = state.parts().pos.get(event.id).0;
    show_flare_scale(view, context, pos, [1.0, 0.0, 0.0, 0.7], 2.5)
}

fn visualize_event_use_ability_explode_fire(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    event: &event::UseAbility,
) -> Box<Action> {
    let pos = state.parts().pos.get(event.id).0;
    show_flare_scale(view, context, pos, [1.0, 0.0, 0.0, 0.7], 2.5)
}

fn visualize_event_use_ability_explode_poison(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    event: &event::UseAbility,
) -> Box<Action> {
    let pos = state.parts().pos.get(event.id).0;
    show_flare_scale(view, context, pos, [0.0, 1.0, 0.0, 0.7], 2.5)
}

fn visualize_event_use_ability_summon(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    event: &event::UseAbility,
) -> Box<Action> {
    let pos = state.parts().pos.get(event.id).0;
    show_flare_scale(view, context, pos, [1.0, 1.0, 1.0, 0.7], 2.0)
}

fn visualize_event_use_ability(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    event: &event::UseAbility,
) -> Box<Action> {
    let action_main = match event.ability {
        Ability::Jump(_) => visualize_event_use_ability_jump(state, view, context, event),
        Ability::Dash => visualize_event_use_ability_dash(state, view, context, event),
        Ability::Explode => visualize_event_use_ability_explode(state, view, context, event),
        Ability::ExplodeFire => {
            visualize_event_use_ability_explode_fire(state, view, context, event)
        }
        Ability::ExplodePoison => {
            visualize_event_use_ability_explode_poison(state, view, context, event)
        }
        Ability::Summon(_) => visualize_event_use_ability_summon(state, view, context, event),
        _ => Box::new(action::Sleep::new(Duration::from_millis(0))),
    };
    let pos = state.parts().pos.get(event.id).0;
    let text = event.ability.to_str();
    Box::new(action::Sequence::new(vec![
        action_main,
        message(view, context, pos, &format!("<{}>", text)),
    ]))
}

fn visualize_event_use_passive_ability(
    _: &State,
    _: &mut GameView,
    _: &mut Context,
    _: &event::UsePassiveAbility,
) -> Box<Action> {
    Box::new(action::Sleep::new(Duration::from_millis(0)))
}

fn visualize_event_effect_tick(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    event: &event::EffectTick,
) -> Box<Action> {
    let pos = state.parts().pos.get(event.id).0;
    match event.effect {
        LastingEffect::Poison => show_flare(view, context, pos, [0.0, 0.8, 0.0, 0.7]),
        LastingEffect::Stun => show_flare(view, context, pos, [1.0, 1.0, 1.0, 0.7]),
    }
}

fn visualize_event_effect_end(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    event: &event::EffectEnd,
) -> Box<Action> {
    let pos = state.parts().pos.get(event.id).0;
    let s = event.effect.to_str();
    message(view, context, pos, &format!("[{}] ended", s))
}

pub fn visualize_lasting_effect(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    target_id: ObjId,
    timed_effect: &TimedEffect,
) -> Box<Action> {
    let pos = state.parts().pos.get(target_id).0;
    let action_flare = match timed_effect.effect {
        LastingEffect::Poison => show_flare(view, context, pos, [0.0, 0.8, 0.0, 0.7]),
        LastingEffect::Stun => show_flare(view, context, pos, [1.0, 1.0, 1.0, 0.7]),
    };
    let s = timed_effect.effect.to_str();
    Box::new(action::Sequence::new(vec![
        action_flare,
        message(view, context, pos, &format!("[{}]", s)),
    ]))
}

pub fn visualize_instant_effect(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    target_id: ObjId,
    effect: &Effect,
) -> Box<Action> {
    debug!("visualize_instant_effect: {:?}", effect);
    let main_action = match *effect {
        Effect::Create(ref e) => visualize_effect_create(state, view, context, target_id, e),
        Effect::Kill => visualize_effect_kill(state, view, context, target_id),
        Effect::Vanish => visualize_effect_vanish(state, view, context, target_id),
        Effect::Stun => visualize_effect_stun(state, view, context, target_id),
        Effect::Heal(ref e) => visualize_effect_heal(state, view, context, target_id, e),
        Effect::Wound(ref e) => visualize_effect_wound(state, view, context, target_id, e),
        Effect::Knockback(ref e) => visualize_effect_knockback(state, view, context, target_id, e),
        Effect::FlyOff(ref e) => visualize_effect_fly_off(state, view, context, target_id, e),
        Effect::Throw(ref e) => visualize_effect_throw(state, view, context, target_id, e),
        Effect::Miss => visualize_effect_miss(state, view, context, target_id),
    };
    Box::new(action::Sequence::new(vec![main_action]))
}

fn visualize_effect_create(
    _: &State,
    view: &mut GameView,
    context: &mut Context,
    target_id: ObjId,
    effect: &effect::Create,
) -> Box<Action> {
    visualize_create(view, context, target_id, effect.pos, &effect.prototype)
}

fn visualize_effect_kill(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    target_id: ObjId,
) -> Box<Action> {
    let pos = state.parts().pos.get(target_id).0;
    Box::new(action::Sequence::new(vec![
        message(view, context, pos, "killed"),
        vanish(view, target_id),
        Box::new(action::Sleep::new(Duration::from_millis(250))),
        show_blood_spot(view, context, pos),
    ]))
}

fn visualize_effect_vanish(
    _: &State,
    view: &mut GameView,
    _: &mut Context,
    target_id: ObjId,
) -> Box<Action> {
    debug!("visualize_effect_vanish!");
    vanish(view, target_id)
}

fn visualize_effect_stun(
    _state: &State,
    _view: &mut GameView,
    _context: &mut Context,
    _target_id: ObjId,
) -> Box<Action> {
    Box::new(action::Sleep::new(Duration::from_millis(1_000)))
}

fn visualize_effect_heal(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    target_id: ObjId,
    effect: &effect::Heal,
) -> Box<Action> {
    let pos = state.parts().pos.get(target_id).0;
    let s = format!("healed +{}", effect.strength.0);
    Box::new(action::Sequence::new(vec![
        Box::new(action::Sleep::new(Duration::from_millis(500))),
        message(view, context, pos, &s),
        show_flare(view, context, pos, [0.0, 0.0, 0.9, 0.7]),
    ]))
}

fn visualize_effect_wound(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    target_id: ObjId,
    effect: &effect::Wound,
) -> Box<Action> {
    let damage = effect.damage;
    let pos = state.parts().pos.get(target_id).0;
    let sprite = view.id_to_sprite(target_id).clone();
    let color_normal = [1.0, 1.0, 1.0, 1.0];
    let color_dark = [0.1, 0.1, 0.1, 1.0];
    Box::new(action::Sequence::new(vec![
        message(view, context, pos, &format!("wounded - {}", damage.0)),
        Box::new(action::ChangeColorTo::new(&sprite, color_dark, Duration::from_millis(200))),
        Box::new(action::ChangeColorTo::new(&sprite, color_normal, Duration::from_millis(200))),
        show_blood_spot(view, context, pos),
    ]))
}

fn visualize_effect_knockback(
    _: &State,
    view: &mut GameView,
    context: &mut Context,
    target_id: ObjId,
    effect: &effect::Knockback,
) -> Box<Action> {
    let sprite = view.id_to_sprite(target_id).clone();
    let from = map::hex_to_point(view.tile_size(), effect.from);
    let to = map::hex_to_point(view.tile_size(), effect.to);
    let diff = Point(to.0 - from.0);
    Box::new(action::Sequence::new(vec![
        message(view, context, effect.to, "bump"),
        Box::new(action::MoveBy::new(&sprite, diff, Duration::from_millis(150))),
    ]))
}

fn visualize_effect_fly_off(
    _: &State,
    view: &mut GameView,
    context: &mut Context,
    target_id: ObjId,
    effect: &effect::FlyOff,
) -> Box<Action> {
    let sprite = view.id_to_sprite(target_id).clone();
    let from = map::hex_to_point(view.tile_size(), effect.from);
    let to = map::hex_to_point(view.tile_size(), effect.to);
    let diff = Point(to.0 - from.0);
    let action_move = arc_move(view, &sprite, diff);
    Box::new(action::Sequence::new(vec![
        message(view, context, effect.to, "fly off"),
        action_move,
    ]))
}

fn visualize_effect_throw(
    _: &State,
    view: &mut GameView,
    _: &mut Context,
    target_id: ObjId,
    effect: &effect::Throw,
) -> Box<Action> {
    let sprite = view.id_to_sprite(target_id).clone();
    let from = map::hex_to_point(view.tile_size(), effect.from);
    let to = map::hex_to_point(view.tile_size(), effect.to);
    let diff = Point(to.0 - from.0);
    arc_move(view, &sprite, diff)
}

fn visualize_effect_miss(
    state: &State,
    view: &mut GameView,
    context: &mut Context,
    target_id: ObjId,
) -> Box<Action> {
    let pos = state.parts().pos.get(target_id).0;
    message(view, context, pos, "missed")
}
