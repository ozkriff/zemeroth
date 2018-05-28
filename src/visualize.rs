use std::time::Duration;

use ggez::graphics::{Color, Point2, Text, Vector2};
use ggez::nalgebra;
use ggez::Context;
use scene::action;
use scene::{Action, Sprite};

use battle_view::BattleView;
use core::ability::Ability;
use core::effect::{self, Effect, LastingEffect, TimedEffect};
use core::event::{self, ActiveEvent, Event};
use core::execute::ApplyPhase;
use core::map::PosHex;
use core::{ObjId, PlayerId, State};
use geom;
use ZResult;

// TODO: Move to some other module
fn time_s(s: f32) -> Duration {
    let ms = s * 1000.0;
    Duration::from_millis(ms as u64)
}

pub fn message(
    view: &mut BattleView,
    context: &mut Context,
    pos: PosHex,
    text: &str,
) -> ZResult<Box<Action>> {
    let visible = [0.0, 0.0, 0.0, 1.0].into();
    let invisible = Color { a: 0.0, ..visible };
    let image = Text::new(context, text, view.font())?.into_inner();
    let mut sprite = Sprite::from_image(image, 0.1);
    sprite.set_centered(true);
    let point = geom::hex_to_point(view.tile_size(), pos);
    let point = point - Vector2::new(0.0, view.tile_size());
    sprite.set_pos(point);
    sprite.set_color(invisible);
    let action_show_hide = Box::new(action::Sequence::new(vec![
        Box::new(action::Show::new(&view.layers().text, &sprite)),
        Box::new(action::ChangeColorTo::new(&sprite, visible, time_s(0.3))),
        Box::new(action::Sleep::new(time_s(1.0))),
        // TODO: read the time from Config:
        Box::new(action::ChangeColorTo::new(&sprite, invisible, time_s(1.0))),
        Box::new(action::Hide::new(&view.layers().text, &sprite)),
    ]));
    let time = action_show_hide.duration();
    let delta = Point2::new(0.0, -0.3);
    let action_move = Box::new(action::MoveBy::new(&sprite, delta, time));
    Ok(Box::new(action::Fork::new(Box::new(
        action::Sequence::new(vec![
            Box::new(action::Fork::new(action_move)),
            action_show_hide,
        ]),
    ))))
}

fn show_blood_spot(
    view: &mut BattleView,
    context: &mut Context,
    at: PosHex,
) -> ZResult<Box<Action>> {
    let mut sprite = Sprite::from_path(context, "/blood.png", view.tile_size() * 2.0)?;
    sprite.set_centered(true);
    sprite.set_color([1.0, 1.0, 1.0, 0.0].into());
    let mut point = geom::hex_to_point(view.tile_size(), at);
    point.y += view.tile_size() * 0.5;
    sprite.set_pos(point);
    let color_final = [1.0, 1.0, 1.0, 0.3].into();
    let time = time_s(0.3);
    Ok(Box::new(action::Sequence::new(vec![
        Box::new(action::Show::new(&view.layers().blood, &sprite)),
        Box::new(action::ChangeColorTo::new(&sprite, color_final, time)),
    ])))
}

fn show_flare_scale(
    view: &mut BattleView,
    context: &mut Context,
    at: PosHex,
    color: Color,
    scale: f32,
) -> ZResult<Box<Action>> {
    let visible = color;
    let invisible = Color { a: 0.0, ..visible };
    let size = view.tile_size() * 2.0 * scale;
    let mut sprite = Sprite::from_path(context, "/white_hex.png", size)?;
    let point = geom::hex_to_point(view.tile_size(), at);
    sprite.set_centered(true);
    sprite.set_pos(point);
    sprite.set_color(invisible);
    Ok(Box::new(action::Sequence::new(vec![
        Box::new(action::Show::new(&view.layers().flares, &sprite)),
        Box::new(action::ChangeColorTo::new(&sprite, visible, time_s(0.1))),
        Box::new(action::ChangeColorTo::new(&sprite, invisible, time_s(0.3))),
        Box::new(action::Hide::new(&view.layers().flares, &sprite)),
    ])))
}

fn show_flare(
    view: &mut BattleView,
    context: &mut Context,
    at: PosHex,
    color: Color,
) -> ZResult<Box<Action>> {
    let scale = 1.0;
    show_flare_scale(view, context, at, color, scale)
}

fn up_and_down_move(
    _: &mut BattleView,
    sprite: &Sprite,
    height: f32,
    time: Duration,
) -> Box<Action> {
    let duration_0_25 = time / 4;
    let up_fast = Point2::new(0.0, -height * 0.75);
    let up_slow = Point2::new(0.0, -height * 0.25);
    let down_slow = -up_slow;
    let down_fast = -up_fast;
    Box::new(action::Sequence::new(vec![
        Box::new(action::MoveBy::new(sprite, up_fast, duration_0_25)),
        Box::new(action::MoveBy::new(sprite, up_slow, duration_0_25)),
        Box::new(action::MoveBy::new(sprite, down_slow, duration_0_25)),
        Box::new(action::MoveBy::new(sprite, down_fast, duration_0_25)),
    ]))
}

// TODO: diff2 -> Vector2
fn arc_move(view: &mut BattleView, sprite: &Sprite, diff: Point2) -> Box<Action> {
    let len = nalgebra::distance(&Point2::origin(), &diff);
    let min_height = view.tile_size() * 0.5;
    let base_height = view.tile_size() * 2.0;
    let min_time = 0.2;
    let base_time = 0.3;
    let height = min_height + base_height * (len / 1.0);
    let time = time_s(min_time + base_time * (len / 1.0));
    let up_and_down = up_and_down_move(view, sprite, height, time);
    let main_move = Box::new(action::MoveBy::new(sprite, diff, time));
    Box::new(action::Sequence::new(vec![
        Box::new(action::Fork::new(main_move)),
        up_and_down,
    ]))
}

fn vanish(view: &mut BattleView, target_id: ObjId) -> Box<Action> {
    debug!("vanish target_id={:?}", target_id);
    let sprite = view.id_to_sprite(target_id).clone();
    view.remove_object(target_id);
    let dark = [0.1, 0.1, 0.1, 1.0].into();
    let invisible = [0.1, 0.1, 0.1, 0.0].into();
    Box::new(action::Sequence::new(vec![
        Box::new(action::Sleep::new(time_s(0.25))),
        Box::new(action::ChangeColorTo::new(&sprite, dark, time_s(0.2))),
        Box::new(action::ChangeColorTo::new(&sprite, invisible, time_s(0.2))),
        Box::new(action::Hide::new(&view.layers().units, &sprite)),
    ]))
}

fn remove_brief_unit_info(view: &mut BattleView, id: ObjId) -> ZResult<Box<Action>> {
    let mut actions: Vec<Box<Action>> = Vec::new();
    let sprites = view.unit_info_get(id);
    for sprite in sprites {
        let color = Color {
            a: 0.0,
            ..sprite.color()
        };
        actions.push(Box::new(action::Fork::new(Box::new(
            action::Sequence::new(vec![
                Box::new(action::ChangeColorTo::new(&sprite, color, time_s(0.4))),
                Box::new(action::Hide::new(&view.layers().dots, &sprite)),
            ]),
        ))));
    }
    Ok(Box::new(action::Sequence::new(actions)))
}

fn generate_brief_obj_info(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    id: ObjId,
) -> ZResult<Box<Action>> {
    let mut actions: Vec<Box<Action>> = Vec::new();
    let agent = state.parts().agent.get(id);
    let obj_pos = state.parts().pos.get(id).0;
    let strength = state.parts().strength.get(id);
    let size = 0.2 * view.tile_size();
    let mut point = geom::hex_to_point(view.tile_size(), obj_pos);
    point.x += view.tile_size() * 0.8;
    point.y -= view.tile_size() * 0.6;
    let mut dots = Vec::new();
    let base_x = point.x;
    for &(color, n) in &[
        ([0.0, 0.6, 0.0, 1.0], strength.strength.0),
        ([0.2, 0.2, 0.5, 1.0], agent.jokers.0),
        ([1.0, 0.0, 0.0, 1.0], agent.attacks.0),
        ([0.0, 0.0, 1.0, 1.0], agent.moves.0),
    ] {
        for _ in 0..n {
            dots.push((color, point));
            point.x -= size;
        }
        point.x = base_x;
        point.y += size;
    }
    let mut sprites = Vec::new();
    for &(color, point) in &dots {
        let color = color.into();
        let mut sprite = Sprite::from_path(context, "/white_hex.png", size)?;
        sprite.set_centered(true);
        sprite.set_pos(point);
        sprite.set_color(Color { a: 0.0, ..color });
        let action = Box::new(action::Fork::new(Box::new(action::Sequence::new(vec![
            Box::new(action::Show::new(&view.layers().dots, &sprite)),
            Box::new(action::ChangeColorTo::new(&sprite, color, time_s(0.1))),
        ]))));
        sprites.push(sprite);
        actions.push(action);
    }
    view.unit_info_set(id, sprites);
    Ok(Box::new(action::Sequence::new(actions)))
}

pub fn refresh_brief_unit_info(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    id: ObjId,
) -> ZResult<Box<Action>> {
    let mut actions = Vec::new();
    if view.unit_info_check(id) {
        actions.push(remove_brief_unit_info(view, id)?);
    }
    if state.parts().agent.get_opt(id).is_some() {
        actions.push(generate_brief_obj_info(state, view, context, id)?);
    }
    Ok(Box::new(action::Sequence::new(actions)))
}

pub fn visualize(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &Event,
    phase: ApplyPhase,
) -> ZResult<Box<Action>> {
    debug!("visualize: phase={:?} event={:?}", phase, event);
    match phase {
        ApplyPhase::Pre => visualize_pre(state, view, context, event),
        ApplyPhase::Post => visualize_post(state, view, context, event),
    }
}

fn visualize_pre(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &Event,
) -> ZResult<Box<Action>> {
    let mut actions: Vec<Box<Action>> = Vec::new();
    actions.push(visualize_event(state, view, context, &event.active_event)?);
    for (&id, effects) in &event.instant_effects {
        for effect in effects {
            let action = visualize_instant_effect(state, view, context, id, effect)?;
            actions.push(Box::new(action::Fork::new(action)));
        }
    }
    for (&id, effects) in &event.timed_effects {
        for effect in effects {
            actions.push(visualize_lasting_effect(state, view, context, id, effect)?);
        }
    }
    Ok(Box::new(action::Sequence::new(actions)))
}

fn visualize_post(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &Event,
) -> ZResult<Box<Action>> {
    let mut actions = Vec::new();
    for &id in &event.actor_ids {
        actions.push(refresh_brief_unit_info(state, view, context, id)?);
    }
    for &id in event.instant_effects.keys() {
        actions.push(refresh_brief_unit_info(state, view, context, id)?);
    }
    for &id in event.timed_effects.keys() {
        actions.push(refresh_brief_unit_info(state, view, context, id)?);
    }
    Ok(Box::new(action::Sequence::new(actions)))
}

fn visualize_event(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &ActiveEvent,
) -> ZResult<Box<Action>> {
    info!("{:?}", event);
    let action = match *event {
        ActiveEvent::Create => Box::new(action::Sleep::new(time_s(0.0))),
        ActiveEvent::MoveTo(ref ev) => visualize_event_move_to(state, view, context, ev)?,
        ActiveEvent::Attack(ref ev) => visualize_event_attack(state, view, context, ev)?,
        ActiveEvent::EndTurn(ref ev) => visualize_event_end_turn(state, view, context, ev),
        ActiveEvent::BeginTurn(ref ev) => visualize_event_begin_turn(state, view, context, ev)?,
        ActiveEvent::EffectTick(ref ev) => visualize_event_effect_tick(state, view, context, ev)?,
        ActiveEvent::EffectEnd(ref ev) => visualize_event_effect_end(state, view, context, ev)?,
        ActiveEvent::UseAbility(ref ev) => visualize_event_use_ability(state, view, context, ev)?,
        ActiveEvent::UsePassiveAbility(ref ev) => {
            visualize_event_use_passive_ability(state, view, context, ev)
        }
    };
    Ok(action)
}

fn visualize_create(
    view: &mut BattleView,
    context: &mut Context,
    id: ObjId,
    pos: PosHex,
    prototype: &str,
) -> ZResult<Box<Action>> {
    let point = geom::hex_to_point(view.tile_size(), pos);
    // TODO: Move to some .ron config:
    let sprite_name = match prototype {
        "swordsman" => "/swordsman.png",
        "spearman" => "/spearman.png",
        "hammerman" => "/hammerman.png",
        "alchemist" => "/alchemist.png",
        "imp" => "/imp.png",
        "imp_toxic" => "/imp_toxic.png",
        "imp_bomber" => "/imp_bomber.png",
        "imp_summoner" => "/imp_summoner.png",
        "boulder" => "/boulder.png",
        "bomb" => "/bomb.png",
        "bomb_fire" => "/bomb_fire.png",
        "bomb_poison" => "/bomb_poison.png",
        "fire" => "/fire.png",
        "poison_cloud" => "/poison_cloud.png",
        "spike_trap" => "/spike_trap.png",
        _ => unimplemented!("Don't know such object type: {}", prototype),
    };
    let size = view.tile_size() * 2.0;
    let mut sprite = Sprite::from_path(context, sprite_name, size)?;
    let color = [1.0, 1.0, 1.0, 1.0].into();
    sprite.set_color(Color { a: 0.0, ..color });
    sprite.set_centered(true);
    sprite.set_pos(point);
    view.add_object(id, &sprite);
    Ok(Box::new(action::Sequence::new(vec![
        Box::new(action::Show::new(&view.layers().units, &sprite)),
        Box::new(action::ChangeColorTo::new(&sprite, color, time_s(0.25))),
    ])))
}

fn visualize_event_move_to(
    _: &State,
    view: &mut BattleView,
    _: &mut Context,
    event: &event::MoveTo,
) -> ZResult<Box<Action>> {
    let sprite = view.id_to_sprite(event.id).clone();
    let mut actions: Vec<Box<Action>> = Vec::new();
    for step in event.path.steps() {
        let from = geom::hex_to_point(view.tile_size(), step.from);
        let to = geom::hex_to_point(view.tile_size(), step.to);
        let diff = to - from;
        let step_height = 0.025;
        let step_time = time_s(0.13);
        let main_move = Box::new(action::MoveBy::new(
            &sprite,
            Point2::origin() + diff, // TODO: ugly hack
            time_s(0.3),
        ));
        let action = Box::new(action::Sequence::new(vec![
            Box::new(action::Fork::new(main_move)),
            up_and_down_move(view, &sprite, step_height, step_time),
            up_and_down_move(view, &sprite, step_height, step_time),
        ]));
        actions.push(action);
    }
    Ok(Box::new(action::Sequence::new(actions)))
}

fn visualize_event_attack(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &event::Attack,
) -> ZResult<Box<Action>> {
    let sprite = view.id_to_sprite(event.attacker_id).clone();
    let map_to = state.parts().pos.get(event.target_id).0;
    let to = geom::hex_to_point(view.tile_size(), map_to);
    let map_from = state.parts().pos.get(event.attacker_id).0;
    let from = geom::hex_to_point(view.tile_size(), map_from);
    let diff = Point2::origin() + ((to - from) / 2.0); // TODO: na-hack
    let mut actions: Vec<Box<Action>> = Vec::new();
    actions.push(Box::new(action::Sleep::new(time_s(0.1))));
    if event.mode == event::AttackMode::Reactive {
        actions.push(Box::new(action::Sleep::new(time_s(0.3))));
        actions.push(message(view, context, map_from, "reaction")?);
    }
    actions.push(Box::new(action::MoveBy::new(&sprite, diff, time_s(0.1))));
    actions.push(Box::new(action::MoveBy::new(&sprite, -diff, time_s(0.15))));
    actions.push(Box::new(action::Sleep::new(time_s(0.1))));
    Ok(Box::new(action::Sequence::new(actions)))
}

fn visualize_event_end_turn(
    _: &State,
    _: &mut BattleView,
    _: &mut Context,
    _: &event::EndTurn,
) -> Box<Action> {
    Box::new(action::Sleep::new(time_s(0.2)))
}

fn visualize_event_begin_turn(
    _: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &event::BeginTurn,
) -> ZResult<Box<Action>> {
    let visible = [0.0, 0.0, 0.0, 1.0].into();
    let invisible = Color { a: 0.0, ..visible };
    let text = match event.player_id {
        PlayerId(0) => "YOUR TURN",
        PlayerId(1) => "ENEMY TURN",
        _ => unreachable!(),
    };
    let text = Text::new(context, text, view.font())?;
    let mut sprite = Sprite::from_image(text.into_inner(), 0.2);
    sprite.set_centered(true);
    sprite.set_color(invisible);
    Ok(Box::new(action::Sequence::new(vec![
        Box::new(action::Show::new(&view.layers().text, &sprite)),
        Box::new(action::ChangeColorTo::new(&sprite, visible, time_s(0.2))),
        Box::new(action::Sleep::new(time_s(1.0))),
        Box::new(action::ChangeColorTo::new(&sprite, invisible, time_s(0.3))),
        Box::new(action::Hide::new(&view.layers().text, &sprite)),
    ])))
}

fn visualize_event_use_ability_jump(
    state: &State,
    view: &mut BattleView,
    _: &mut Context,
    event: &event::UseAbility,
) -> ZResult<Box<Action>> {
    let sprite = view.id_to_sprite(event.id).clone();
    let from = state.parts().pos.get(event.id).0;
    let from = geom::hex_to_point(view.tile_size(), from);
    let to = geom::hex_to_point(view.tile_size(), event.pos);
    let diff = to - from;
    let diff = Point2::origin() + diff; // TODO: na-hack
    Ok(arc_move(view, &sprite, diff))
}

fn visualize_event_use_ability_dash(
    state: &State,
    view: &mut BattleView,
    _: &mut Context,
    event: &event::UseAbility,
) -> ZResult<Box<Action>> {
    let sprite = view.id_to_sprite(event.id).clone();
    let from = state.parts().pos.get(event.id).0;
    let from = geom::hex_to_point(view.tile_size(), from);
    let to = geom::hex_to_point(view.tile_size(), event.pos);
    let diff = to - from;
    let diff = Point2::origin() + diff; // TODO: na-hack
    Ok(Box::new(action::MoveBy::new(&sprite, diff, time_s(0.1))))
}

fn visualize_event_use_ability_explode(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &event::UseAbility,
) -> ZResult<Box<Action>> {
    let pos = state.parts().pos.get(event.id).0;
    let scale = 2.5;
    show_flare_scale(view, context, pos, [1.0, 0.0, 0.0, 0.7].into(), scale)
}

fn visualize_event_use_ability_explode_fire(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &event::UseAbility,
) -> ZResult<Box<Action>> {
    let pos = state.parts().pos.get(event.id).0;
    let scale = 2.5;
    show_flare_scale(view, context, pos, [1.0, 0.0, 0.0, 0.7].into(), scale)
}

fn visualize_event_use_ability_explode_poison(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &event::UseAbility,
) -> ZResult<Box<Action>> {
    let pos = state.parts().pos.get(event.id).0;
    let scale = 2.5;
    show_flare_scale(view, context, pos, [0.0, 1.0, 0.0, 0.7].into(), scale)
}

fn visualize_event_use_ability_summon(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &event::UseAbility,
) -> ZResult<Box<Action>> {
    let pos = state.parts().pos.get(event.id).0;
    let scale = 2.0;
    show_flare_scale(view, context, pos, [1.0, 1.0, 1.0, 0.7].into(), scale)
}

fn visualize_event_use_ability(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &event::UseAbility,
) -> ZResult<Box<Action>> {
    let action_main = match event.ability {
        Ability::Jump(_) => visualize_event_use_ability_jump(state, view, context, event)?,
        Ability::Dash => visualize_event_use_ability_dash(state, view, context, event)?,
        Ability::Explode => visualize_event_use_ability_explode(state, view, context, event)?,
        Ability::ExplodeFire => {
            visualize_event_use_ability_explode_fire(state, view, context, event)?
        }
        Ability::ExplodePoison => {
            visualize_event_use_ability_explode_poison(state, view, context, event)?
        }
        Ability::Summon(_) => visualize_event_use_ability_summon(state, view, context, event)?,
        _ => Box::new(action::Sleep::new(time_s(0.0))),
    };
    let pos = state.parts().pos.get(event.id).0;
    let text = event.ability.to_string();
    Ok(Box::new(action::Sequence::new(vec![
        action_main,
        message(view, context, pos, &format!("<{}>", text))?,
    ])))
}

fn visualize_event_use_passive_ability(
    _: &State,
    _: &mut BattleView,
    _: &mut Context,
    _: &event::UsePassiveAbility,
) -> Box<Action> {
    Box::new(action::Sleep::new(time_s(0.0)))
}

fn visualize_event_effect_tick(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &event::EffectTick,
) -> ZResult<Box<Action>> {
    let pos = state.parts().pos.get(event.id).0;
    match event.effect {
        LastingEffect::Poison => show_flare(view, context, pos, [0.0, 0.8, 0.0, 0.7].into()),
        LastingEffect::Stun => show_flare(view, context, pos, [1.0, 1.0, 1.0, 0.7].into()),
    }
}

fn visualize_event_effect_end(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &event::EffectEnd,
) -> ZResult<Box<Action>> {
    let pos = state.parts().pos.get(event.id).0;
    let s = event.effect.to_str();
    message(view, context, pos, &format!("[{}] ended", s))
}

pub fn visualize_lasting_effect(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: ObjId,
    timed_effect: &TimedEffect,
) -> ZResult<Box<Action>> {
    let pos = state.parts().pos.get(target_id).0;
    let action_flare = match timed_effect.effect {
        LastingEffect::Poison => show_flare(view, context, pos, [0.0, 0.8, 0.0, 0.7].into())?,
        LastingEffect::Stun => show_flare(view, context, pos, [1.0, 1.0, 1.0, 0.7].into())?,
    };
    let s = timed_effect.effect.to_str();
    Ok(Box::new(action::Sequence::new(vec![
        action_flare,
        message(view, context, pos, &format!("[{}]", s))?,
    ])))
}

pub fn visualize_instant_effect(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: ObjId,
    effect: &Effect,
) -> ZResult<Box<Action>> {
    debug!("visualize_instant_effect: {:?}", effect);
    let action = match *effect {
        Effect::Create(ref e) => visualize_effect_create(state, view, context, target_id, e)?,
        Effect::Kill => visualize_effect_kill(state, view, context, target_id)?,
        Effect::Vanish => visualize_effect_vanish(state, view, context, target_id),
        Effect::Stun => visualize_effect_stun(state, view, context, target_id)?,
        Effect::Heal(ref e) => visualize_effect_heal(state, view, context, target_id, e)?,
        Effect::Wound(ref e) => visualize_effect_wound(state, view, context, target_id, e)?,
        Effect::Knockback(ref e) => visualize_effect_knockback(state, view, context, target_id, e)?,
        Effect::FlyOff(ref e) => visualize_effect_fly_off(state, view, context, target_id, e)?,
        Effect::Throw(ref e) => visualize_effect_throw(state, view, context, target_id, e)?,
        Effect::Miss => visualize_effect_miss(state, view, context, target_id)?,
    };
    Ok(action)
}

fn visualize_effect_create(
    _: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: ObjId,
    effect: &effect::Create,
) -> ZResult<Box<Action>> {
    visualize_create(view, context, target_id, effect.pos, &effect.prototype)
}

fn visualize_effect_kill(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: ObjId,
) -> ZResult<Box<Action>> {
    let pos = state.parts().pos.get(target_id).0;
    Ok(Box::new(action::Sequence::new(vec![
        message(view, context, pos, "killed")?,
        vanish(view, target_id),
        Box::new(action::Sleep::new(time_s(0.25))),
        show_blood_spot(view, context, pos)?,
    ])))
}

fn visualize_effect_vanish(
    _: &State,
    view: &mut BattleView,
    _: &mut Context,
    target_id: ObjId,
) -> Box<Action> {
    debug!("visualize_effect_vanish!");
    vanish(view, target_id)
}

fn visualize_effect_stun(
    _state: &State,
    _view: &mut BattleView,
    _context: &mut Context,
    _target_id: ObjId,
) -> ZResult<Box<Action>> {
    Ok(Box::new(action::Sleep::new(time_s(1.0))))
}

fn visualize_effect_heal(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: ObjId,
    effect: &effect::Heal,
) -> ZResult<Box<Action>> {
    let pos = state.parts().pos.get(target_id).0;
    let s = format!("healed +{}", effect.strength.0);
    Ok(Box::new(action::Sequence::new(vec![
        Box::new(action::Sleep::new(time_s(0.5))),
        message(view, context, pos, &s)?,
        show_flare(view, context, pos, [0.0, 0.0, 0.9, 0.7].into())?,
    ])))
}

fn visualize_effect_wound(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: ObjId,
    effect: &effect::Wound,
) -> ZResult<Box<Action>> {
    let damage = effect.damage;
    let pos = state.parts().pos.get(target_id).0;
    let sprite = view.id_to_sprite(target_id).clone();
    let c_normal = [1.0, 1.0, 1.0, 1.0].into();
    let c_dark = [0.1, 0.1, 0.1, 1.0].into();
    let time = time_s(0.2);
    Ok(Box::new(action::Sequence::new(vec![
        message(view, context, pos, &format!("wounded - {}", damage.0))?,
        Box::new(action::ChangeColorTo::new(&sprite, c_dark, time)),
        Box::new(action::ChangeColorTo::new(&sprite, c_normal, time)),
        show_blood_spot(view, context, pos)?,
    ])))
}

fn visualize_effect_knockback(
    _: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: ObjId,
    effect: &effect::Knockback,
) -> ZResult<Box<Action>> {
    let sprite = view.id_to_sprite(target_id).clone();
    let from = geom::hex_to_point(view.tile_size(), effect.from);
    let to = geom::hex_to_point(view.tile_size(), effect.to);
    // let diff = Point(to.0 - from.0);
    let diff = Point2::origin() + (to - from); // TODO: na-hack
    Ok(Box::new(action::Sequence::new(vec![
        message(view, context, effect.to, "bump")?,
        Box::new(action::MoveBy::new(&sprite, diff, time_s(0.15))),
    ])))
}

fn visualize_effect_fly_off(
    _: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: ObjId,
    effect: &effect::FlyOff,
) -> ZResult<Box<Action>> {
    let sprite = view.id_to_sprite(target_id).clone();
    let from = geom::hex_to_point(view.tile_size(), effect.from);
    let to = geom::hex_to_point(view.tile_size(), effect.to);
    // let diff = Point(to.0 - from.0);
    let diff = Point2::origin() + (to - from); // TODO: na-hack
    let action_move = arc_move(view, &sprite, diff);
    Ok(Box::new(action::Sequence::new(vec![
        message(view, context, effect.to, "fly off")?,
        action_move,
    ])))
}

fn visualize_effect_throw(
    _: &State,
    view: &mut BattleView,
    _: &mut Context,
    target_id: ObjId,
    effect: &effect::Throw,
) -> ZResult<Box<Action>> {
    let sprite = view.id_to_sprite(target_id).clone();
    let from = geom::hex_to_point(view.tile_size(), effect.from);
    let to = geom::hex_to_point(view.tile_size(), effect.to);
    // let diff = Point(to.0 - from.0);
    let diff = Point2::origin() + (to - from); // TODO: na-hack
    Ok(arc_move(view, &sprite, diff))
}

fn visualize_effect_miss(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: ObjId,
) -> ZResult<Box<Action>> {
    let pos = state.parts().pos.get(target_id).0;
    message(view, context, pos, "missed")
}
