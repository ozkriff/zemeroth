use std::time::Duration;

use ggez::graphics::{Color, Text, Vector2};
use ggez::nalgebra;
use ggez::Context;
use scene::action;
use scene::{Action, Boxed, Sprite};

use core::map::PosHex;
use core::tactical_map::ability::Ability;
use core::tactical_map::effect::{self, Effect, LastingEffect, TimedEffect};
use core::tactical_map::event::{self, ActiveEvent, Event};
use core::tactical_map::execute::ApplyPhase;
use core::tactical_map::{ObjId, PlayerId, State};
use geom;
use screen::battle::view::BattleView;
use utils::{fork, seq, time_s};
use ZResult;

// TODO: why is this public?
pub fn message(
    view: &mut BattleView,
    context: &mut Context,
    pos: PosHex,
    text: &str,
) -> ZResult<Box<dyn Action>> {
    let visible = [0.0, 0.0, 0.0, 1.0].into();
    let invisible = Color { a: 0.0, ..visible };
    let image = Text::new(context, text, view.font())?.into_inner();
    let mut sprite = Sprite::from_image(image, 0.1);
    sprite.set_centered(true);
    let point = geom::hex_to_point(view.tile_size(), pos);
    let point = point - Vector2::new(0.0, view.tile_size() * 1.5);
    sprite.set_pos(point);
    sprite.set_color(invisible);
    let action_show_hide = seq(vec![
        action::Show::new(&view.layers().text, &sprite).boxed(),
        action::ChangeColorTo::new(&sprite, visible, time_s(0.3)).boxed(),
        action::Sleep::new(time_s(1.0)).boxed(),
        // TODO: read the time from Config:
        action::ChangeColorTo::new(&sprite, invisible, time_s(1.0)).boxed(),
        action::Hide::new(&view.layers().text, &sprite).boxed(),
    ]);
    let time = action_show_hide.duration();
    let delta = -Vector2::new(0.0, 0.3);
    let action_move = action::MoveBy::new(&sprite, delta, time).boxed();
    Ok(fork(seq(vec![fork(action_move), action_show_hide])))
}

fn show_blood_spot(view: &mut BattleView, at: PosHex) -> ZResult<Box<dyn Action>> {
    let mut sprite = Sprite::from_image(view.images().blood.clone(), view.tile_size() * 2.0);
    sprite.set_centered(true);
    sprite.set_color([1.0, 1.0, 1.0, 0.0].into());
    let mut point = geom::hex_to_point(view.tile_size(), at);
    point.y += view.tile_size() * 0.1;
    sprite.set_pos(point);
    let color_final = [1.0, 1.0, 1.0, 1.0].into();
    let time = time_s(0.3);
    Ok(seq(vec![
        action::Show::new(&view.layers().blood, &sprite).boxed(),
        action::ChangeColorTo::new(&sprite, color_final, time).boxed(),
    ]))
}

fn show_flare_scale(
    view: &mut BattleView,
    at: PosHex,
    color: Color,
    scale: f32,
) -> ZResult<Box<dyn Action>> {
    let visible = color;
    let invisible = Color { a: 0.0, ..visible };
    let size = view.tile_size() * 2.0 * scale;
    let mut sprite = Sprite::from_image(view.images().white_hex.clone(), size);
    let point = geom::hex_to_point(view.tile_size(), at);
    sprite.set_centered(true);
    sprite.set_pos(point);
    sprite.set_color(invisible);
    Ok(seq(vec![
        action::Show::new(&view.layers().flares, &sprite).boxed(),
        action::ChangeColorTo::new(&sprite, visible, time_s(0.1)).boxed(),
        action::ChangeColorTo::new(&sprite, invisible, time_s(0.3)).boxed(),
        action::Hide::new(&view.layers().flares, &sprite).boxed(),
    ]))
}

fn show_flare(view: &mut BattleView, at: PosHex, color: Color) -> ZResult<Box<dyn Action>> {
    let scale = 1.0;
    show_flare_scale(view, at, color, scale)
}

fn up_and_down_move(
    _: &mut BattleView,
    sprite: &Sprite,
    height: f32,
    time: Duration,
) -> Box<dyn Action> {
    let duration_0_25 = time / 4;
    let up_fast = Vector2::new(0.0, -height * 0.75);
    let up_slow = Vector2::new(0.0, -height * 0.25);
    let down_slow = -up_slow;
    let down_fast = -up_fast;
    seq(vec![
        action::MoveBy::new(sprite, up_fast, duration_0_25).boxed(),
        action::MoveBy::new(sprite, up_slow, duration_0_25).boxed(),
        action::MoveBy::new(sprite, down_slow, duration_0_25).boxed(),
        action::MoveBy::new(sprite, down_fast, duration_0_25).boxed(),
    ])
}

fn arc_move(view: &mut BattleView, sprite: &Sprite, diff: Vector2) -> Box<dyn Action> {
    let len = nalgebra::norm(&diff);
    let min_height = view.tile_size() * 0.5;
    let base_height = view.tile_size() * 2.0;
    let min_time = 0.2;
    let base_time = 0.3;
    let height = min_height + base_height * (len / 1.0);
    let time = time_s(min_time + base_time * (len / 1.0));
    let up_and_down = up_and_down_move(view, sprite, height, time);
    let main_move = action::MoveBy::new(sprite, diff, time).boxed();
    seq(vec![fork(main_move), up_and_down])
}

fn vanish(view: &mut BattleView, target_id: ObjId) -> Box<dyn Action> {
    debug!("vanish target_id={:?}", target_id);
    let sprite = view.id_to_sprite(target_id).clone();
    let sprite_shadow = view.id_to_shadow_sprite(target_id).clone();
    view.remove_object(target_id);
    let dark = [0.1, 0.1, 0.1, 1.0].into();
    let invisible = [0.1, 0.1, 0.1, 0.0].into();
    seq(vec![
        action::Sleep::new(time_s(0.25)).boxed(),
        action::ChangeColorTo::new(&sprite, dark, time_s(0.2)).boxed(),
        action::ChangeColorTo::new(&sprite, invisible, time_s(0.2)).boxed(),
        action::Hide::new(&view.layers().objects, &sprite).boxed(),
        action::ChangeColorTo::new(&sprite_shadow, invisible, time_s(0.2)).boxed(),
        action::Hide::new(&view.layers().shadows, &sprite_shadow).boxed(),
    ])
}

fn remove_brief_agent_info(view: &mut BattleView, id: ObjId) -> ZResult<Box<dyn Action>> {
    let mut actions = Vec::new();
    let sprites = view.agent_info_get(id);
    for sprite in sprites {
        let color = Color {
            a: 0.0,
            ..sprite.color()
        };
        actions.push(fork(seq(vec![
            action::ChangeColorTo::new(&sprite, color, time_s(0.4)).boxed(),
            action::Hide::new(&view.layers().dots, &sprite).boxed(),
        ])));
    }
    Ok(seq(actions))
}

fn generate_brief_obj_info(
    state: &State,
    view: &mut BattleView,
    id: ObjId,
) -> ZResult<Box<dyn Action>> {
    let image = view.images().dot.clone();
    let mut actions = Vec::new();
    let agent = state.parts().agent.get(id);
    let obj_pos = state.parts().pos.get(id).0;
    let strength = state.parts().strength.get(id);
    let size = 0.2 * view.tile_size();
    let mut point = geom::hex_to_point(view.tile_size(), obj_pos);
    point.x += view.tile_size() * 0.8;
    point.y -= view.tile_size() * 1.6;
    let mut dots = Vec::new();
    let base_x = point.x;
    // TODO: draw missing health as transparent "ghosty" dots
    for &(color, n) in &[
        ([0.0, 0.7, 0.0, 1.0], strength.strength.0),
        ([0.9, 0.1, 0.9, 1.0], agent.jokers.0),
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
        let mut sprite = Sprite::from_image(image.clone(), size);
        sprite.set_centered(true);
        sprite.set_pos(point);
        sprite.set_color(Color { a: 0.0, ..color });
        let action = fork(seq(vec![
            action::Show::new(&view.layers().dots, &sprite).boxed(),
            action::ChangeColorTo::new(&sprite, color, time_s(0.1)).boxed(),
        ]));
        sprites.push(sprite);
        actions.push(action);
    }
    view.agent_info_set(id, sprites);
    Ok(seq(actions))
}

// TODO: why is this public?
// pub fn refresh_brief_agent_info(
fn refresh_brief_agent_info(
    state: &State,
    view: &mut BattleView,
    id: ObjId,
) -> ZResult<Box<dyn Action>> {
    let mut actions = Vec::new();
    if view.agent_info_check(id) {
        actions.push(remove_brief_agent_info(view, id)?);
    }
    if state.parts().agent.get_opt(id).is_some() {
        actions.push(generate_brief_obj_info(state, view, id)?);
    }
    Ok(seq(actions))
}

pub fn visualize(
    state: &State,
    view: &mut BattleView,
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
    view: &mut BattleView,
    context: &mut Context,
    event: &Event,
) -> ZResult<Box<dyn Action>> {
    let mut actions = Vec::new();
    actions.push(visualize_event(state, view, context, &event.active_event)?);
    for (&id, effects) in &event.instant_effects {
        for effect in effects {
            actions.push(visualize_instant_effect(state, view, context, id, effect)?);
        }
    }
    for (&id, effects) in &event.timed_effects {
        for effect in effects {
            actions.push(visualize_lasting_effect(state, view, context, id, effect)?);
        }
    }
    Ok(seq(actions))
}

fn visualize_post(state: &State, view: &mut BattleView, event: &Event) -> ZResult<Box<dyn Action>> {
    let mut actions = Vec::new();
    for &id in &event.actor_ids {
        actions.push(refresh_brief_agent_info(state, view, id)?);
    }
    for &id in event.instant_effects.keys() {
        actions.push(refresh_brief_agent_info(state, view, id)?);
    }
    for &id in event.timed_effects.keys() {
        actions.push(refresh_brief_agent_info(state, view, id)?);
    }
    Ok(seq(actions))
}

fn visualize_event(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &ActiveEvent,
) -> ZResult<Box<dyn Action>> {
    info!("{:?}", event);
    let action = match *event {
        ActiveEvent::UsePassiveAbility(_) | ActiveEvent::Create => action::Empty::new().boxed(),
        ActiveEvent::MoveTo(ref ev) => visualize_event_move_to(state, view, context, ev)?,
        ActiveEvent::Attack(ref ev) => visualize_event_attack(state, view, context, ev)?,
        ActiveEvent::EndTurn(ref ev) => visualize_event_end_turn(state, view, context, ev),
        ActiveEvent::BeginTurn(ref ev) => visualize_event_begin_turn(state, view, context, ev)?,
        ActiveEvent::EffectTick(ref ev) => visualize_event_effect_tick(state, view, ev)?,
        ActiveEvent::EffectEnd(ref ev) => visualize_event_effect_end(state, view, context, ev)?,
        ActiveEvent::UseAbility(ref ev) => visualize_event_use_ability(state, view, context, ev)?,
    };
    Ok(action)
}

fn visualize_create(
    view: &mut BattleView,
    context: &mut Context,
    id: ObjId,
    pos: PosHex,
    prototype: &str,
) -> ZResult<Box<dyn Action>> {
    // TODO: Move to some .ron config:
    // TODO: At lest, extract this to a separate function
    let (sprite_name, offset_x, offset_y, shadow_size_coefficient) = match prototype {
        "swordsman" => ("/swordsman.png", 0.15, 0.1, 1.0),
        "spearman" => ("/spearman.png", 0.2, 0.05, 1.0),
        "hammerman" => ("/hammerman.png", 0.05, 0.1, 1.0),
        "alchemist" => ("/alchemist.png", 0.05, 0.1, 1.0),
        "imp" => ("/imp.png", -0.05, 0.15, 1.3),
        "imp_toxic" => ("/imp_toxic.png", -0.05, 0.15, 1.2),
        "imp_bomber" => ("/imp_bomber.png", -0.05, 0.15, 1.2),
        "imp_summoner" => ("/imp_summoner.png", -0.05, 0.15, 1.3),
        "boulder" => ("/boulder.png", 0.0, 0.4, 2.5),
        "bomb_damage" => ("/bomb.png", 0.0, 0.2, 0.7),
        "bomb_push" => ("/bomb.png", 0.0, 0.2, 0.7),
        "bomb_fire" => ("/bomb_fire.png", 0.0, 0.2, 0.7),
        "bomb_poison" => ("/bomb_poison.png", 0.0, 0.2, 0.7),
        "bomb_demonic" => ("/bomb_demonic.png", 0.0, 0.2, 0.7),
        "fire" => ("/fire.png", 0.0, 0.2, 0.001),
        "poison_cloud" => ("/poison_cloud.png", 0.0, 0.2, 2.0),
        "spike_trap" => ("/spike_trap.png", 0.0, 0.5, 1.4),
        _ => unimplemented!("Don't know such object type: {}", prototype),
    };
    let point = geom::hex_to_point(view.tile_size(), pos);
    let color = [1.0, 1.0, 1.0, 1.0].into();
    let size = view.tile_size() * 2.0;
    let sprite_object = {
        let mut sprite = Sprite::from_path(context, sprite_name, size)?;
        sprite.set_color(Color { a: 0.0, ..color });
        sprite.set_offset(Vector2::new(0.5 - offset_x, 1.0 - offset_y));
        sprite.set_pos(point);
        sprite
    };
    let sprite_shadow = {
        let image_shadow = view.images().shadow.clone();
        let mut sprite = Sprite::from_image(image_shadow, size * shadow_size_coefficient);
        sprite.set_centered(true);
        sprite.set_color(Color { a: 0.0, ..color });
        sprite.set_pos(point);
        sprite
    };
    view.add_object(id, &sprite_object, &sprite_shadow);
    let action_change_shadow_color =
        action::ChangeColorTo::new(&sprite_shadow, color, time_s(0.2)).boxed();
    Ok(seq(vec![
        action::Show::new(&view.layers().shadows, &sprite_shadow).boxed(),
        action::Show::new(&view.layers().objects, &sprite_object).boxed(),
        fork(action_change_shadow_color),
        action::ChangeColorTo::new(&sprite_object, color, time_s(0.25)).boxed(),
    ]))
}

fn visualize_event_move_to(
    _: &State,
    view: &mut BattleView,
    _: &mut Context,
    event: &event::MoveTo,
) -> ZResult<Box<dyn Action>> {
    let sprite = view.id_to_sprite(event.id).clone();
    let sprite_shadow = view.id_to_shadow_sprite(event.id).clone();
    let mut actions = Vec::new();
    for step in event.path.steps() {
        let from = geom::hex_to_point(view.tile_size(), step.from);
        let to = geom::hex_to_point(view.tile_size(), step.to);
        let diff = to - from;
        let step_height = 0.025;
        let step_time = time_s(0.13);
        let move_time = time_s(0.3);
        let main_move = action::MoveBy::new(&sprite, diff, move_time).boxed();
        let shadow_move = action::MoveBy::new(&sprite_shadow, diff, move_time).boxed();
        let action = seq(vec![
            fork(main_move),
            fork(shadow_move),
            up_and_down_move(view, &sprite, step_height, step_time),
            up_and_down_move(view, &sprite, step_height, step_time),
        ]);
        actions.push(action);
    }
    Ok(seq(actions))
}

fn visualize_event_attack(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &event::Attack,
) -> ZResult<Box<dyn Action>> {
    let sprite = view.id_to_sprite(event.attacker_id).clone();
    let sprite_shadow = view.id_to_shadow_sprite(event.attacker_id).clone();
    let map_to = state.parts().pos.get(event.target_id).0;
    let to = geom::hex_to_point(view.tile_size(), map_to);
    let map_from = state.parts().pos.get(event.attacker_id).0;
    let from = geom::hex_to_point(view.tile_size(), map_from);
    let diff = (to - from) / 2.0;
    let mut actions = Vec::new();
    actions.push(action::Sleep::new(time_s(0.1)).boxed());
    if event.mode == event::AttackMode::Reactive {
        actions.push(action::Sleep::new(time_s(0.3)).boxed());
        actions.push(message(view, context, map_from, "reaction")?);
    }
    let time_to = time_s(0.1);
    let time_from = time_s(0.15);
    let action_sprite_move_to = action::MoveBy::new(&sprite, diff, time_to).boxed();
    let action_shadow_move_to = action::MoveBy::new(&sprite_shadow, diff, time_to).boxed();
    let action_sprite_move_from = action::MoveBy::new(&sprite, -diff, time_from).boxed();
    let action_shadow_move_from = action::MoveBy::new(&sprite_shadow, -diff, time_from).boxed();
    actions.push(fork(action_shadow_move_to));
    actions.push(action_sprite_move_to);
    actions.push(fork(action_shadow_move_from));
    actions.push(action_sprite_move_from);
    actions.push(action::Sleep::new(time_s(0.1)).boxed());
    Ok(seq(actions))
}

fn visualize_event_end_turn(
    _: &State,
    _: &mut BattleView,
    _: &mut Context,
    _: &event::EndTurn,
) -> Box<dyn Action> {
    action::Sleep::new(time_s(0.2)).boxed()
}

fn visualize_event_begin_turn(
    _: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &event::BeginTurn,
) -> ZResult<Box<dyn Action>> {
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
    Ok(seq(vec![
        action::Show::new(&view.layers().text, &sprite).boxed(),
        action::ChangeColorTo::new(&sprite, visible, time_s(0.2)).boxed(),
        action::Sleep::new(time_s(1.0)).boxed(),
        action::ChangeColorTo::new(&sprite, invisible, time_s(0.3)).boxed(),
        action::Hide::new(&view.layers().text, &sprite).boxed(),
    ]))
}

fn visualize_event_use_ability_jump(
    state: &State,
    view: &mut BattleView,
    _: &mut Context,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    let sprite_object = view.id_to_sprite(event.id).clone();
    let sprite_shadow = view.id_to_shadow_sprite(event.id).clone();
    let from = state.parts().pos.get(event.id).0;
    let from = geom::hex_to_point(view.tile_size(), from);
    let to = geom::hex_to_point(view.tile_size(), event.pos);
    let diff = to - from;
    let action_arc_move = arc_move(view, &sprite_object, diff);
    let time = action_arc_move.duration();
    let action_move_shadow = action::MoveBy::new(&sprite_shadow, diff, time).boxed();
    Ok(seq(vec![fork(action_move_shadow), action_arc_move]))
}

fn visualize_event_use_ability_dash(
    state: &State,
    view: &mut BattleView,
    _: &mut Context,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    let sprite_object = view.id_to_sprite(event.id).clone();
    let sprite_shadow = view.id_to_shadow_sprite(event.id).clone();
    let from = state.parts().pos.get(event.id).0;
    let from = geom::hex_to_point(view.tile_size(), from);
    let to = geom::hex_to_point(view.tile_size(), event.pos);
    let diff = to - from;
    let time = time_s(0.1);
    let main_move = action::MoveBy::new(&sprite_object, diff, time).boxed();
    let action_move_shadow = action::MoveBy::new(&sprite_shadow, diff, time).boxed();
    Ok(seq(vec![fork(action_move_shadow), main_move]))
}

fn visualize_event_use_ability_explode(
    state: &State,
    view: &mut BattleView,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    let pos = state.parts().pos.get(event.id).0;
    let scale = 2.5;
    show_flare_scale(view, pos, [1.0, 0.0, 0.0, 0.7].into(), scale).map(fork)
}

fn visualize_event_use_ability_summon(
    state: &State,
    view: &mut BattleView,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    let pos = state.parts().pos.get(event.id).0;
    let scale = 2.0;
    show_flare_scale(view, pos, [1.0, 1.0, 1.0, 0.7].into(), scale)
}

fn visualize_event_use_ability(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    let action_main = match event.ability {
        Ability::Jump(_) => visualize_event_use_ability_jump(state, view, context, event)?,
        Ability::Dash => visualize_event_use_ability_dash(state, view, context, event)?,
        Ability::ExplodePush => visualize_event_use_ability_explode(state, view, event)?,
        Ability::ExplodeDamage => visualize_event_use_ability_explode(state, view, event)?,
        Ability::ExplodeFire => visualize_event_use_ability_explode(state, view, event)?,
        Ability::ExplodePoison => visualize_event_use_ability_explode(state, view, event)?,
        Ability::Summon(_) => visualize_event_use_ability_summon(state, view, event)?,
        _ => action::Empty::new().boxed(),
    };
    let pos = state.parts().pos.get(event.id).0;
    let text = event.ability.to_string();
    Ok(seq(vec![
        action_main,
        message(view, context, pos, &format!("<{}>", text))?,
    ]))
}

fn visualize_event_effect_tick(
    state: &State,
    view: &mut BattleView,
    event: &event::EffectTick,
) -> ZResult<Box<dyn Action>> {
    let pos = state.parts().pos.get(event.id).0;
    match event.effect {
        LastingEffect::Poison => show_flare(view, pos, [0.0, 0.8, 0.0, 0.7].into()),
        LastingEffect::Stun => show_flare(view, pos, [1.0, 1.0, 1.0, 0.7].into()),
    }
}

fn visualize_event_effect_end(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &event::EffectEnd,
) -> ZResult<Box<dyn Action>> {
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
) -> ZResult<Box<dyn Action>> {
    let pos = state.parts().pos.get(target_id).0;
    let action_flare = match timed_effect.effect {
        LastingEffect::Poison => show_flare(view, pos, [0.0, 0.8, 0.0, 0.7].into())?,
        LastingEffect::Stun => show_flare(view, pos, [1.0, 1.0, 1.0, 0.7].into())?,
    };
    let s = timed_effect.effect.to_str();
    Ok(seq(vec![
        action_flare,
        message(view, context, pos, &format!("[{}]", s))?,
    ]))
}

pub fn visualize_instant_effect(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: ObjId,
    effect: &Effect,
) -> ZResult<Box<dyn Action>> {
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
) -> ZResult<Box<dyn Action>> {
    visualize_create(view, context, target_id, effect.pos, &effect.prototype).map(fork)
}

fn visualize_effect_kill(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: ObjId,
) -> ZResult<Box<dyn Action>> {
    let pos = state.parts().pos.get(target_id).0;
    Ok(fork(seq(vec![
        message(view, context, pos, "killed")?,
        vanish(view, target_id),
        action::Sleep::new(time_s(0.25)).boxed(),
        show_blood_spot(view, pos)?,
    ])))
}

fn visualize_effect_vanish(
    _: &State,
    view: &mut BattleView,
    _: &mut Context,
    target_id: ObjId,
) -> Box<dyn Action> {
    debug!("visualize_effect_vanish!");
    fork(vanish(view, target_id))
}

fn visualize_effect_stun(
    _state: &State,
    _view: &mut BattleView,
    _context: &mut Context,
    _target_id: ObjId,
) -> ZResult<Box<dyn Action>> {
    Ok(fork(action::Sleep::new(time_s(1.0)).boxed()))
}

fn visualize_effect_heal(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: ObjId,
    effect: &effect::Heal,
) -> ZResult<Box<dyn Action>> {
    let pos = state.parts().pos.get(target_id).0;
    let s = format!("healed +{}", effect.strength.0);
    Ok(seq(vec![
        action::Sleep::new(time_s(0.5)).boxed(),
        message(view, context, pos, &s)?,
        show_flare(view, pos, [0.0, 0.0, 0.9, 0.7].into())?,
    ]))
}

fn visualize_effect_wound(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: ObjId,
    effect: &effect::Wound,
) -> ZResult<Box<dyn Action>> {
    let damage = effect.damage;
    let pos = state.parts().pos.get(target_id).0;
    let sprite = view.id_to_sprite(target_id).clone();
    let c_normal = [1.0, 1.0, 1.0, 1.0].into();
    let c_dark = [0.1, 0.1, 0.1, 1.0].into();
    let time = time_s(0.2);
    Ok(fork(seq(vec![
        message(view, context, pos, &format!("wounded - {}", damage.0))?,
        action::ChangeColorTo::new(&sprite, c_dark, time).boxed(),
        action::ChangeColorTo::new(&sprite, c_normal, time).boxed(),
        show_blood_spot(view, pos)?,
    ])))
}

fn visualize_effect_knockback(
    _: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: ObjId,
    effect: &effect::Knockback,
) -> ZResult<Box<dyn Action>> {
    let sprite = view.id_to_sprite(target_id).clone();
    let sprite_shadow = view.id_to_shadow_sprite(target_id).clone();
    let from = geom::hex_to_point(view.tile_size(), effect.from);
    let to = geom::hex_to_point(view.tile_size(), effect.to);
    let diff = to - from;
    let time = time_s(0.15);
    let action_main_move = action::MoveBy::new(&sprite, diff, time).boxed();
    let action_move_shadow = action::MoveBy::new(&sprite_shadow, diff, time).boxed();
    Ok(fork(seq(vec![
        message(view, context, effect.to, "bump")?,
        fork(action_move_shadow),
        action_main_move,
    ])))
}

fn visualize_effect_fly_off(
    _: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: ObjId,
    effect: &effect::FlyOff,
) -> ZResult<Box<dyn Action>> {
    let sprite_object = view.id_to_sprite(target_id).clone();
    let sprite_shadow = view.id_to_shadow_sprite(target_id).clone();
    let from = geom::hex_to_point(view.tile_size(), effect.from);
    let to = geom::hex_to_point(view.tile_size(), effect.to);
    let diff = to - from;
    let action_main_move = arc_move(view, &sprite_object, diff);
    let time = action_main_move.duration();
    let action_move_shadow = action::MoveBy::new(&sprite_shadow, diff, time).boxed();
    Ok(fork(seq(vec![
        message(view, context, effect.to, "fly off")?,
        fork(action_move_shadow),
        action_main_move,
    ])))
}

fn visualize_effect_throw(
    _: &State,
    view: &mut BattleView,
    _: &mut Context,
    target_id: ObjId,
    effect: &effect::Throw,
) -> ZResult<Box<dyn Action>> {
    let sprite = view.id_to_sprite(target_id).clone();
    let sprite_shadow = view.id_to_shadow_sprite(target_id).clone();
    let from = geom::hex_to_point(view.tile_size(), effect.from);
    let to = geom::hex_to_point(view.tile_size(), effect.to);
    let diff = to - from;
    let arc_move = arc_move(view, &sprite, diff);
    let action_move_shadow = action::MoveBy::new(&sprite_shadow, diff, arc_move.duration()).boxed();
    Ok(seq(vec![fork(action_move_shadow), arc_move]))
}

fn visualize_effect_miss(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: ObjId,
) -> ZResult<Box<dyn Action>> {
    let pos = state.parts().pos.get(target_id).0;
    message(view, context, pos, "missed")
}
