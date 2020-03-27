use std::time::Duration;

use gwg::{
    graphics::{Color, Text},
    Context,
};
use log::{debug, info};
use nalgebra::{self, Point2, Vector2};
use rand::Rng;
use scene::{action, Action, Boxed, Facing, Sprite};

use crate::{
    core::{
        battle::{
            ability::Ability,
            component::{Component, WeaponType},
            effect::{self, Effect},
            event::{self, ActiveEvent, Event},
            execute::{hit_chance, ApplyPhase},
            state, Id, PlayerId, State,
        },
        map::PosHex,
        utils::zrng,
    },
    geom,
    screen::battle::view::BattleView,
    sprite_info::SpriteInfo,
    utils::{font_size, time_s},
    ZResult,
};

const BLOOD_SPRITE_DURATION_TURNS: i32 = 6; // TODO: i32 -> Turns, Rounds, etc
const TIME_LUNGE_TO: f32 = 0.1;
const TIME_LUNGE_FROM: f32 = 0.15;

pub fn seq(actions: Vec<Box<dyn Action>>) -> Box<dyn Action> {
    action::Sequence::new(actions).boxed()
}

pub fn fork(action: Box<dyn Action>) -> Box<dyn Action> {
    action::Fork::new(action).boxed()
}

pub fn message(
    view: &mut BattleView,
    context: &mut Context,
    pos: PosHex,
    text: &str,
) -> ZResult<Box<dyn Action>> {
    let visible = [0.0, 0.0, 0.0, 1.0].into();
    let invisible = Color { a: 0.0, ..visible };
    let font_size = font_size();
    let text = Box::new(Text::new((text, view.font(), font_size)));
    let mut sprite = Sprite::from_drawable(context, text, 0.1)?;
    sprite.set_centered(true);
    let point = view.hex_to_point(pos);
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

// TODO: Add some bg? Text is not that readable atm.
fn announce(
    view: &mut BattleView,
    context: &mut Context,
    text: &str,
    time: Duration,
) -> ZResult<Box<dyn Action>> {
    let text_height = 0.2;
    let visible = [0.0, 0.0, 0.0, 1.0].into();
    let invisible = Color { a: 0.0, ..visible };
    let text = Box::new(Text::new((text, view.font(), font_size())));
    let mut sprite = Sprite::from_drawable(context, text, text_height)?;
    sprite.set_centered(true);
    sprite.set_color(invisible);
    let time_8 = time / 8;
    Ok(seq(vec![
        action::Show::new(&view.layers().text, &sprite).boxed(),
        action::ChangeColorTo::new(&sprite, visible, time_8).boxed(),
        action::Sleep::new(time_8 * 5).boxed(),
        action::ChangeColorTo::new(&sprite, invisible, time_8 * 2).boxed(),
        action::Hide::new(&view.layers().text, &sprite).boxed(),
    ]))
}

pub fn attack_message(
    view: &mut BattleView,
    context: &mut Context,
    pos: Point2<f32>,
    text: &str,
) -> ZResult<Box<dyn Action>> {
    let visible = [0.0, 0.0, 0.0, 1.0].into();
    let invisible = Color { a: 0.0, ..visible };
    let font_size = font_size();
    let text = Box::new(Text::new((text, view.font(), font_size)));
    let mut sprite = Sprite::from_drawable(context, text, 0.1)?;
    sprite.set_centered(true);
    let point = pos + Vector2::new(0.0, view.tile_size() * 0.5);
    sprite.set_pos(point);
    sprite.set_color(invisible);
    let action_show_hide = seq(vec![
        action::Show::new(&view.layers().text, &sprite).boxed(),
        // TODO: read the time from Config:
        action::ChangeColorTo::new(&sprite, visible, time_s(0.3)).boxed(),
        action::ChangeColorTo::new(&sprite, invisible, time_s(0.3)).boxed(),
        action::Hide::new(&view.layers().text, &sprite).boxed(),
    ]);
    Ok(fork(action_show_hide))
}

fn show_blood_particles(
    view: &mut BattleView,
    context: &mut Context,
    pos: PosHex,
    from: Option<PosHex>,
    particles_count: i32,
) -> ZResult<Box<dyn Action>> {
    let point_origin = view.hex_to_point(pos);
    let mut actions = Vec::new();
    for _ in 0..particles_count {
        let offset = if let Some(from) = from {
            let from = view.hex_to_point(from);
            let diff = (point_origin - from).normalize() * view.tile_size();
            diff + geom::rand_tile_offset(view.tile_size(), 0.8)
        } else {
            geom::rand_tile_offset(view.tile_size(), 1.7)
        };
        let point = point_origin + offset;
        let color = [0.7, 0.0, 0.0, 0.6].into();
        let visible = color;
        let invisible = Color { a: 0.0, ..visible };
        let scale = zrng().gen_range(0.05, 0.15);
        let size = view.tile_size() * 2.0 * scale;
        let mut sprite = Sprite::from_image(context, view.images().white_hex.clone(), size)?;
        sprite.set_centered(true);
        sprite.set_pos(point_origin);
        sprite.set_color(invisible);
        let vector = point - point_origin;
        let layer = view.layers().blood.clone();
        actions.push(fork(seq(vec![
            action::Show::new(&view.layers().flares, &sprite).boxed(),
            fork(action::ChangeColorTo::new(&sprite, visible, time_s(0.2)).boxed()),
            arc_move(view, &sprite, vector),
            action::Hide::new(&view.layers().flares, &sprite).boxed(),
            action::Show::new(&layer, &sprite).boxed(),
        ])));
        view.add_disappearing_sprite(&layer, &sprite, BLOOD_SPRITE_DURATION_TURNS, visible.a);
    }
    Ok(fork(seq(actions)))
}

fn show_blood_spot(
    view: &mut BattleView,
    context: &mut Context,
    at: PosHex,
) -> ZResult<Box<dyn Action>> {
    let mut sprite =
        Sprite::from_image(context, view.images().blood.clone(), view.tile_size() * 2.0)?;
    sprite.set_centered(true);
    sprite.set_color([1.0, 1.0, 1.0, 0.0].into());
    let mut point = view.hex_to_point(at);
    point.y += view.tile_size() * 0.1;
    sprite.set_pos(point);
    let color_final: Color = [1.0, 1.0, 1.0, 1.0].into();
    let time = time_s(0.3);
    let layer = view.layers().blood.clone();
    view.add_disappearing_sprite(&layer, &sprite, BLOOD_SPRITE_DURATION_TURNS, color_final.a);
    Ok(seq(vec![
        action::Show::new(&layer, &sprite).boxed(),
        action::ChangeColorTo::new(&sprite, color_final, time).boxed(),
    ]))
}

fn show_explosion_ground_mark(
    view: &mut BattleView,
    context: &mut Context,
    at: PosHex,
) -> ZResult<Box<dyn Action>> {
    let mut sprite = Sprite::from_image(
        context,
        view.images().explosion_ground_mark.clone(),
        view.tile_size() * 2.0,
    )?;
    sprite.set_centered(true);
    sprite.set_color([1.0, 1.0, 1.0, 1.0].into());
    sprite.set_pos(view.hex_to_point(at));
    let layer = view.layers().blood.clone();
    let duration = BLOOD_SPRITE_DURATION_TURNS;
    view.add_disappearing_sprite(&layer, &sprite, duration, sprite.color().a);
    Ok(action::Show::new(&layer, &sprite).boxed())
}

fn show_dust_at_pos(
    view: &mut BattleView,
    context: &mut Context,
    at: PosHex,
) -> ZResult<Box<dyn Action>> {
    let point = view.hex_to_point(at);
    let count = 9;
    show_dust(view, context, point, count)
}

fn show_dust(
    view: &mut BattleView,
    context: &mut Context,
    at: Point2<f32>,
    count: i32,
) -> ZResult<Box<dyn Action>> {
    let mut actions = Vec::new();
    for i in 0..count {
        let k = zrng().gen_range(0.8, 1.2);
        let visible = [0.8 * k, 0.8 * k, 0.7 * k, 0.8 * k].into();
        let invisible = Color { a: 0.0, ..visible };
        let scale = zrng().gen_range(0.2, 0.4);
        let size = view.tile_size() * 2.0 * scale;
        let vector = {
            let max = std::f32::consts::PI * 2.0;
            let rot = nalgebra::Rotation2::new((max / count as f32) * i as f32);
            let n = zrng().gen_range(0.4, 0.6);
            let mut vector = rot * Vector2::new(view.tile_size() * n, 0.0);
            vector.y *= geom::FLATNESS_COEFFICIENT;
            vector
        };
        let point = at + vector;
        let sprite = {
            let mut sprite = Sprite::from_image(context, view.images().white_hex.clone(), size)?;
            sprite.set_centered(true);
            sprite.set_pos(point);
            sprite.set_color(invisible);
            sprite
        };
        let layer = &view.layers().particles;
        let action_show_hide = seq(vec![
            action::Show::new(layer, &sprite).boxed(),
            action::ChangeColorTo::new(&sprite, visible, time_s(0.2)).boxed(),
            action::ChangeColorTo::new(&sprite, invisible, time_s(0.7)).boxed(),
            action::Hide::new(layer, &sprite).boxed(),
        ]);
        let time = action_show_hide.duration();
        let action_move = action::MoveBy::new(&sprite, vector, time).boxed();
        let action = seq(vec![fork(action_move), fork(action_show_hide)]);
        actions.push(action);
    }
    Ok(seq(actions))
}

fn show_flare_scale(
    view: &mut BattleView,
    context: &mut Context,
    at: PosHex,
    color: Color,
    scale: f32,
) -> ZResult<Box<dyn Action>> {
    let visible = color;
    let invisible = Color { a: 0.0, ..visible };
    let size = view.tile_size() * 2.0 * scale;
    let mut sprite = Sprite::from_image(context, view.images().white_hex.clone(), size)?;
    let point = view.hex_to_point(at);
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

fn show_weapon_flash(
    view: &mut BattleView,
    context: &mut Context,
    at: PosHex,
    weapon_type: WeaponType,
    facing_opt: Option<geom::Facing>,
) -> ZResult<Box<dyn Action>> {
    let visible = [1.0, 1.0, 1.0, 0.8].into();
    let invisible = Color { a: 0.1, ..visible };
    let tile_size = view.tile_size();
    let sprite_size = tile_size * 2.0;
    let image = match weapon_type {
        WeaponType::Slash => view.images().attack_slash.clone(),
        WeaponType::Smash => view.images().attack_smash.clone(),
        WeaponType::Pierce => view.images().attack_pierce.clone(),
        WeaponType::Claw => view.images().attack_claws.clone(),
    };
    let mut sprite = Sprite::from_image(context, image, sprite_size)?;
    let point = view.hex_to_point(at) - Vector2::new(0.0, tile_size * 0.3);
    sprite.set_centered(true);
    sprite.set_pos(point);
    sprite.set_color(invisible);
    let mut actions = Vec::new();
    actions.push(action::Show::new(&view.layers().flares, &sprite).boxed());
    if let Some(facing) = facing_opt {
        actions.push(action::SetFacing::new(&sprite, facing.to_scene_facing()).boxed());
    }
    actions.push(action::ChangeColorTo::new(&sprite, visible, time_s(0.1)).boxed());
    actions.push(action::ChangeColorTo::new(&sprite, invisible, time_s(0.4)).boxed());
    actions.push(action::Hide::new(&view.layers().flares, &sprite).boxed());
    Ok(fork(seq(actions)))
}

fn show_flare(
    view: &mut BattleView,
    context: &mut Context,
    at: PosHex,
    color: Color,
) -> ZResult<Box<dyn Action>> {
    let scale = 1.0;
    show_flare_scale(view, context, at, color, scale)
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

fn move_object_with_shadow(
    view: &mut BattleView,
    id: Id,
    diff: Vector2<f32>,
    time_to: Duration,
) -> Box<dyn Action> {
    let sprite = view.id_to_sprite(id).clone();
    let sprite_shadow = view.id_to_shadow_sprite(id).clone();
    let action_sprite_move_to = action::MoveBy::new(&sprite, diff, time_to).boxed();
    let action_shadow_move_to = action::MoveBy::new(&sprite_shadow, diff, time_to).boxed();
    seq(vec![fork(action_shadow_move_to), action_sprite_move_to])
}

fn arc_move(view: &mut BattleView, sprite: &Sprite, diff: Vector2<f32>) -> Box<dyn Action> {
    let len = nalgebra::Matrix::norm(&diff);
    let min_height = view.tile_size() * 0.5;
    let base_height = view.tile_size() * 2.0;
    let min_time = 0.25;
    let base_time = 0.3;
    let height = min_height + base_height * (len / 1.0);
    let time = time_s(min_time + base_time * (len / 1.0));
    let up_and_down = up_and_down_move(view, sprite, height, time);
    let main_move = action::MoveBy::new(sprite, diff, time).boxed();
    seq(vec![fork(main_move), up_and_down])
}

fn vanish(view: &mut BattleView, target_id: Id) -> Box<dyn Action> {
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

fn remove_brief_agent_info(view: &mut BattleView, id: Id) -> ZResult<Box<dyn Action>> {
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
    context: &mut Context,
    id: Id,
) -> ZResult<Box<dyn Action>> {
    let dot_image = view.images().dot.clone();
    let mut actions = Vec::new();
    let parts = state.parts();
    let agent = parts.agent.get(id);
    let obj_pos = parts.pos.get(id).0;
    let strength = parts.strength.get(id);
    let damage = strength.base_strength.0 - strength.strength.0;
    let armor = state::get_armor(state, id);
    let size = 0.2 * view.tile_size();
    let mut point = view.hex_to_point(obj_pos);
    point.x += view.tile_size() * 0.8;
    point.y -= view.tile_size() * 1.6;
    let mut dots = Vec::new();
    let base = point;
    let rows: &[&[_]] = &[
        &[
            ([0.0, 0.7, 0.0, 1.0], strength.strength.0),
            ([0.3, 0.5, 0.3, 0.5], damage),
            ([1.0, 1.0, 0.0, 1.0], armor.0),
        ],
        &[([0.9, 0.1, 0.9, 1.0], agent.jokers.0)],
        &[([1.0, 0.0, 0.0, 1.0], agent.attacks.0)],
        &[([0.0, 0.0, 1.0, 1.0], agent.moves.0)],
    ];
    for &row in rows {
        for &(color, n) in row {
            for _ in 0..n {
                dots.push((color, point));
                point.x -= size;
            }
        }
        point.x = base.x;
        point.y += size;
    }
    let mut sprites = Vec::new();
    for &(color, point) in &dots {
        let color = color.into();
        let mut sprite = Sprite::from_image(context, dot_image.clone(), size)?;
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
    {
        let health_points = strength.strength.0 + damage + armor.0;
        let health_bar_width = health_points as f32 * size;
        if let Some(effects) = parts.effects.get_opt(id) {
            let icon_size = size * 2.0;
            let mut icon_point = base;
            icon_point.y -= icon_size;
            icon_point.x -= health_bar_width + icon_size * 0.3;
            for timed_effect in &effects.0 {
                icon_point.y += icon_size;
                let effect = &timed_effect.effect;
                let image = match effect {
                    effect::Lasting::Poison => view.images().effect_poison.clone(),
                    effect::Lasting::Stun => view.images().effect_stun.clone(),
                    effect::Lasting::Bloodlust => view.images().effect_bloodlust.clone(),
                };
                let mut sprite = Sprite::from_image(context, image, icon_size)?;
                sprite.set_pos(icon_point);
                sprite.set_centered(true);
                actions.push(action::Show::new(&view.layers().dots, &sprite).boxed());
                sprites.push(sprite);
            }
        }
    }
    view.agent_info_set(id, sprites);
    Ok(seq(actions))
}

pub fn refresh_brief_agent_info(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    id: Id,
) -> ZResult<Box<dyn Action>> {
    let mut actions = Vec::new();
    if view.agent_info_check(id) {
        actions.push(remove_brief_agent_info(view, id)?);
    }
    if state.parts().agent.get_opt(id).is_some() {
        actions.push(generate_brief_obj_info(state, view, context, id)?);
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
        ApplyPhase::Post => visualize_post(state, view, context, event),
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
    for &(id, ref effects) in &event.instant_effects {
        for effect in effects {
            actions.push(visualize_instant_effect(state, view, context, id, &effect)?);
        }
    }
    for &(id, ref effects) in &event.timed_effects {
        for effect in effects {
            actions.push(visualize_lasting_effect(state, view, context, id, &effect)?);
        }
    }
    Ok(seq(actions))
}

fn visualize_post(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &Event,
) -> ZResult<Box<dyn Action>> {
    let mut actions = Vec::new();
    for &id in &event.actor_ids {
        actions.push(refresh_brief_agent_info(state, view, context, id)?);
    }
    for &(id, _) in &event.instant_effects {
        actions.push(refresh_brief_agent_info(state, view, context, id)?);
    }
    for &(id, _) in &event.timed_effects {
        actions.push(refresh_brief_agent_info(state, view, context, id)?);
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
        ActiveEvent::EndBattle(ref ev) => visualize_event_end_battle(state, view, context, ev)?,
        ActiveEvent::EndTurn(ref ev) => visualize_event_end_turn(state, view, context, ev),
        ActiveEvent::BeginTurn(ref ev) => visualize_event_begin_turn(state, view, context, ev)?,
        ActiveEvent::EffectTick(ref ev) => visualize_event_effect_tick(state, view, context, ev)?,
        ActiveEvent::EffectEnd(ref ev) => visualize_event_effect_end(state, view, context, ev)?,
        ActiveEvent::UseAbility(ref ev) => visualize_event_use_ability(state, view, context, ev)?,
    };
    Ok(action)
}

fn visualize_event_move_to(
    _: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &event::MoveTo,
) -> ZResult<Box<dyn Action>> {
    let sprite = view.id_to_sprite(event.id).clone();
    let mut actions = Vec::new();
    if let [pos] = event.path.tiles() {
        let action = fork(seq(vec![
            action::Sleep::new(time_s(0.4)).boxed(), // to avoid overlapping with a damage msg
            message(view, context, *pos, "move interrupted")?,
        ]));
        actions.push(action);
    }
    for step in event.path.steps() {
        let from = view.hex_to_point(step.from);
        let to = view.hex_to_point(step.to);
        let facing = geom::Facing::from_positions(view.tile_size(), step.from, step.to)
            .expect("Bad path step");
        let diff = to - from;
        let step_height = view.tile_size() * 0.25;
        let step_time = time_s(0.13);
        let move_time = time_s(0.3);
        let action = seq(vec![
            action::SetFacing::new(&sprite, facing.to_scene_facing()).boxed(),
            fork(move_object_with_shadow(view, event.id, diff, move_time)),
            up_and_down_move(view, &sprite, step_height, step_time),
            up_and_down_move(view, &sprite, step_height, step_time),
        ]);
        actions.push(action);
    }
    Ok(seq(actions))
}

fn lunge(state: &State, view: &mut BattleView, id: Id, to: PosHex) -> ZResult<Box<dyn Action>> {
    let from = state.parts().pos.get(id).0;
    let diff = (view.hex_to_point(to) - view.hex_to_point(from)) / 2.0;
    let mut actions = Vec::new();
    if let Some(facing) = geom::Facing::from_positions(view.tile_size(), from, to) {
        let sprite = view.id_to_sprite(id).clone();
        actions.push(action::SetFacing::new(&sprite, facing.to_scene_facing()).boxed());
    }
    let time_to = time_s(TIME_LUNGE_TO);
    let time_from = time_s(TIME_LUNGE_FROM);
    actions.push(move_object_with_shadow(view, id, diff, time_to));
    actions.push(move_object_with_shadow(view, id, -diff, time_from));
    Ok(seq(actions))
}

fn visualize_event_attack(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &event::Attack,
) -> ZResult<Box<dyn Action>> {
    let id = event.attacker_id;
    let sprite = view.id_to_sprite(id).clone();
    let map_to = state.parts().pos.get(event.target_id).0;
    let to = view.hex_to_point(map_to);
    let map_from = state.parts().pos.get(id).0;
    let from = view.hex_to_point(map_from);
    let diff = (to - from) / 2.0;
    let mut actions = Vec::new();
    let chances = hit_chance(state, id, event.target_id);
    let attack_msg = format!("{}%", chances.1 * 10);
    actions.push(attack_message(view, context, from, &attack_msg)?);
    actions.push(action::Sleep::new(time_s(0.1)).boxed());
    if event.mode == event::AttackMode::Reactive {
        actions.push(action::Sleep::new(time_s(0.3)).boxed());
        actions.push(message(view, context, map_from, "reaction")?);
    }
    let time_to = time_s(TIME_LUNGE_TO);
    let time_from = time_s(TIME_LUNGE_FROM);
    let facing_opt = geom::Facing::from_positions(view.tile_size(), map_from, map_to);
    if let Some(facing) = facing_opt {
        actions.push(action::SetFacing::new(&sprite, facing.to_scene_facing()).boxed());
    }
    if sprite.has_frame("attack") {
        actions.push(action::SetFrame::new(&sprite, "attack").boxed());
    }
    actions.push(move_object_with_shadow(view, id, diff, time_to));
    actions.push(show_weapon_flash(
        view,
        context,
        map_to,
        event.weapon_type,
        facing_opt,
    )?);
    actions.push(move_object_with_shadow(view, id, -diff, time_from));
    if sprite.has_frame("attack") {
        actions.push(action::SetFrame::new(&sprite, "").boxed());
    }
    Ok(seq(actions))
}

fn visualize_event_end_battle(
    _: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &event::EndBattle,
) -> ZResult<Box<dyn Action>> {
    let text = match event.result.winner_id {
        PlayerId(0) => "YOU WON!",
        PlayerId(1) => "YOU LOSE!",
        _ => unreachable!(),
    };
    Ok(seq(vec![
        action::Sleep::new(time_s(1.0)).boxed(),
        announce(view, context, text, time_s(4.0))?,
        action::Sleep::new(time_s(1.0)).boxed(),
    ]))
}

fn visualize_event_end_turn(
    _: &State,
    view: &mut BattleView,
    _: &mut Context,
    _: &event::EndTurn,
) -> Box<dyn Action> {
    seq(vec![
        view.update_disappearing_sprites(),
        action::Sleep::new(time_s(0.2)).boxed(),
    ])
}

fn visualize_event_begin_turn(
    _: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &event::BeginTurn,
) -> ZResult<Box<dyn Action>> {
    let text = match event.player_id {
        PlayerId(0) => "YOUR TURN",
        PlayerId(1) => "ENEMY TURN",
        _ => unreachable!(),
    };
    announce(view, context, text, time_s(1.5))
}

fn visualize_event_use_ability_jump(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    let sprite_object = view.id_to_sprite(event.id).clone();
    let sprite_shadow = view.id_to_shadow_sprite(event.id).clone();
    let from = state.parts().pos.get(event.id).0;
    let from = view.hex_to_point(from);
    let to = view.hex_to_point(event.pos);
    let diff = to - from;
    let action_arc_move = arc_move(view, &sprite_object, diff);
    let time = action_arc_move.duration();
    let action_move_shadow = action::MoveBy::new(&sprite_shadow, diff, time).boxed();
    let action_dust = show_dust_at_pos(view, context, event.pos)?;
    let mut actions = Vec::new();
    if sprite_object.has_frame("jump") {
        actions.push(action::SetFrame::new(&sprite_object, "jump").boxed());
    }
    actions.push(fork(action_move_shadow));
    actions.push(action_arc_move);
    if sprite_object.has_frame("jump") {
        actions.push(action::SetFrame::new(&sprite_object, "").boxed());
    }
    actions.push(action_dust);
    Ok(seq(actions))
}

fn visualize_event_use_ability_dash(
    state: &State,
    view: &mut BattleView,
    _: &mut Context,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    let from = state.parts().pos.get(event.id).0;
    let point_from = view.hex_to_point(from);
    let point_to = view.hex_to_point(event.pos);
    let diff = point_to - point_from;
    let time = time_s(0.1);
    Ok(move_object_with_shadow(view, event.id, diff, time))
}

fn visualize_event_use_ability_heal(
    _: &State,
    view: &mut BattleView,
    _: &mut Context,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    let sprite = view.id_to_sprite(event.id).clone();
    let frame_name = "heal";
    assert!(sprite.has_frame(frame_name));
    Ok(fork(seq(vec![
        action::SetFrame::new(&sprite, frame_name).boxed(),
        action::Sleep::new(time_s(1.0)).boxed(),
        action::SetFrame::new(&sprite, "").boxed(),
    ])))
}

fn visualize_event_use_ability_rage(
    _: &State,
    view: &mut BattleView,
    _: &mut Context,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    let sprite = view.id_to_sprite(event.id).clone();
    let frame_name = "rage";
    assert!(sprite.has_frame(frame_name));
    Ok(fork(seq(vec![
        action::SetFrame::new(&sprite, frame_name).boxed(),
        action::Sleep::new(time_s(1.0)).boxed(),
        action::SetFrame::new(&sprite, "").boxed(),
    ])))
}

fn visualize_event_use_ability_knockback(
    state: &State,
    view: &mut BattleView,
    _: &mut Context,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    lunge(state, view, event.id, event.pos)
}

fn visualize_event_use_ability_club(
    state: &State,
    view: &mut BattleView,
    _: &mut Context,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    lunge(state, view, event.id, event.pos)
}

fn visualize_event_use_ability_explode(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    let pos = state.parts().pos.get(event.id).0;
    let scale = 2.5;
    let action_dust = show_dust_at_pos(view, context, pos)?;
    let action_flare = show_flare_scale(view, context, pos, [1.0, 0.0, 0.0, 0.7].into(), scale)?;
    let action_ground_mark = show_explosion_ground_mark(view, context, pos)?;
    Ok(seq(vec![
        fork(action_flare),
        fork(action_ground_mark),
        action_dust,
    ]))
}

fn visualize_event_use_ability_summon(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    let sprite = view.id_to_sprite(event.id).clone();
    let frame_name = "summon";
    assert!(sprite.has_frame(frame_name));
    let pos = state.parts().pos.get(event.id).0;
    let scale = 2.0;
    let action_flare = show_flare_scale(view, context, pos, [1.0, 1.0, 1.0, 0.7].into(), scale)?;
    Ok(seq(vec![
        action::SetFrame::new(&sprite, frame_name).boxed(),
        action::Sleep::new(time_s(0.3)).boxed(),
        fork(seq(vec![
            action_flare,
            action::SetFrame::new(&sprite, "").boxed(),
        ])),
    ]))
}

fn visualize_event_use_ability_bloodlust(
    _: &State,
    view: &mut BattleView,
    _: &mut Context,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    let sprite = view.id_to_sprite(event.id).clone();
    Ok(fork(seq(vec![
        action::SetFrame::new(&sprite, "bloodlust").boxed(),
        action::Sleep::new(time_s(0.5)).boxed(),
        action::SetFrame::new(&sprite, "").boxed(),
    ])))
}

fn visualize_event_use_ability_throw_bomb(
    _: &State,
    view: &mut BattleView,
    _: &mut Context,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    let sprite = view.id_to_sprite(event.id).clone();
    Ok(fork(seq(vec![
        action::SetFrame::new(&sprite, "throw").boxed(),
        action::Sleep::new(time_s(0.5)).boxed(),
        action::SetFrame::new(&sprite, "").boxed(),
    ])))
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
        Ability::Summon => visualize_event_use_ability_summon(state, view, context, event)?,
        Ability::Bloodlust => visualize_event_use_ability_bloodlust(state, view, context, event)?,
        Ability::Heal(_) => visualize_event_use_ability_heal(state, view, context, event)?,
        Ability::Rage(_) => visualize_event_use_ability_rage(state, view, context, event)?,
        Ability::Knockback(_) => {
            visualize_event_use_ability_knockback(state, view, context, event)?
        }
        Ability::Club => visualize_event_use_ability_club(state, view, context, event)?,
        Ability::ExplodePush
        | Ability::ExplodeDamage
        | Ability::ExplodeFire
        | Ability::ExplodePoison => {
            visualize_event_use_ability_explode(state, view, context, event)?
        }
        Ability::BombPush(_)
        | Ability::BombDemonic(_)
        | Ability::BombFire(_)
        | Ability::BombPoison(_)
        | Ability::Bomb(_) => visualize_event_use_ability_throw_bomb(state, view, context, event)?,
        _ => action::Empty::new().boxed(),
    };
    let pos = state.parts().pos.get(event.id).0;
    let text = event.ability.title();
    let mut actions = Vec::new();
    if let Some(facing) = geom::Facing::from_positions(view.tile_size(), pos, event.pos) {
        let sprite = view.id_to_sprite(event.id).clone();
        actions.push(action::SetFacing::new(&sprite, facing.to_scene_facing()).boxed());
    }
    actions.push(action_main);
    actions.push(message(view, context, pos, &format!("<{}>", text))?);
    Ok(seq(actions))
}

fn visualize_event_effect_tick(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    event: &event::EffectTick,
) -> ZResult<Box<dyn Action>> {
    let pos = state.parts().pos.get(event.id).0;
    match event.effect {
        effect::Lasting::Poison => show_flare(view, context, pos, [0.0, 0.8, 0.0, 0.7].into()),
        effect::Lasting::Stun => show_flare(view, context, pos, [1.0, 1.0, 1.0, 0.7].into()),
        effect::Lasting::Bloodlust => show_flare(view, context, pos, [1.0, 0.0, 0.0, 0.5].into()),
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
    target_id: Id,
    timed_effect: &effect::Timed,
) -> ZResult<Box<dyn Action>> {
    let pos = state.parts().pos.get(target_id).0;
    let action_flare = match timed_effect.effect {
        effect::Lasting::Poison => show_flare(view, context, pos, [0.0, 0.8, 0.0, 0.7].into())?,
        effect::Lasting::Stun => show_flare(view, context, pos, [1.0, 1.0, 1.0, 0.7].into())?,
        effect::Lasting::Bloodlust => show_flare(view, context, pos, [1.0, 0.0, 0.0, 0.5].into())?,
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
    target_id: Id,
    effect: &Effect,
) -> ZResult<Box<dyn Action>> {
    debug!("visualize_instant_effect: {:?}", effect);
    let action = match *effect {
        Effect::Create(ref e) => visualize_effect_create(state, view, context, target_id, e)?,
        Effect::Kill(ref e) => visualize_effect_kill(state, view, context, target_id, e)?,
        Effect::Vanish => visualize_effect_vanish(state, view, context, target_id),
        Effect::Stun => visualize_effect_stun(state, view, context, target_id)?,
        Effect::Heal(ref e) => visualize_effect_heal(state, view, context, target_id, e)?,
        Effect::Wound(ref e) => visualize_effect_wound(state, view, context, target_id, e)?,
        Effect::Knockback(ref e) => visualize_effect_knockback(state, view, context, target_id, e)?,
        Effect::FlyOff(ref e) => visualize_effect_fly_off(state, view, context, target_id, e)?,
        Effect::Throw(ref e) => visualize_effect_throw(state, view, context, target_id, e)?,
        Effect::Dodge(ref e) => visualize_effect_dodge(state, view, context, target_id, e)?,
        Effect::Bloodlust => action::Empty.boxed(),
    };
    Ok(action)
}

fn visualize_effect_create(
    _: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: Id,
    effect: &effect::Create,
) -> ZResult<Box<dyn Action>> {
    let SpriteInfo {
        paths,
        offset_x,
        offset_y,
        shadow_size_coefficient,
    } = view.sprite_info(&effect.prototype);
    let point = view.hex_to_point(effect.pos);
    let color = [1.0, 1.0, 1.0, 1.0].into();
    let size = view.tile_size() * 2.0;
    let sprite_object = {
        let mut sprite = Sprite::from_paths(context, &paths, size)?;
        sprite.set_color(Color { a: 0.0, ..color });
        sprite.set_offset(Vector2::new(0.5 - offset_x, 1.0 - offset_y));
        sprite.set_pos(point);
        for component in &effect.components {
            if let Component::BelongsTo(belongs_to) = component {
                if belongs_to.0 == PlayerId(1) {
                    sprite.set_facing(Facing::Left);
                }
            }
        }
        sprite
    };
    let sprite_shadow = {
        let image_shadow = view.images().shadow.clone();
        let mut sprite = Sprite::from_image(context, image_shadow, size * shadow_size_coefficient)?;
        sprite.set_centered(true);
        sprite.set_color(Color { a: 0.0, ..color });
        sprite.set_pos(point);
        sprite
    };
    view.add_object(target_id, &sprite_object, &sprite_shadow);
    let action_change_shadow_color =
        action::ChangeColorTo::new(&sprite_shadow, color, time_s(0.2)).boxed();
    Ok(fork(seq(vec![
        action::Show::new(&view.layers().shadows, &sprite_shadow).boxed(),
        action::Show::new(&view.layers().objects, &sprite_object).boxed(),
        fork(action_change_shadow_color),
        action::ChangeColorTo::new(&sprite_object, color, time_s(0.25)).boxed(),
    ])))
}

fn visualize_effect_kill(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: Id,
    effect: &effect::Kill,
) -> ZResult<Box<dyn Action>> {
    let particles_count = 6;
    let pos = state.parts().pos.get(target_id).0;
    Ok(fork(seq(vec![
        show_blood_particles(view, context, pos, effect.attacker_pos, particles_count)?,
        message(view, context, pos, "killed")?,
        vanish(view, target_id),
        show_blood_spot(view, context, pos)?,
    ])))
}

fn visualize_effect_vanish(
    _: &State,
    view: &mut BattleView,
    _: &mut Context,
    target_id: Id,
) -> Box<dyn Action> {
    debug!("visualize_effect_vanish!");
    fork(vanish(view, target_id))
}

fn visualize_effect_stun(
    _state: &State,
    _view: &mut BattleView,
    _context: &mut Context,
    _target_id: Id,
) -> ZResult<Box<dyn Action>> {
    Ok(fork(action::Sleep::new(time_s(1.0)).boxed()))
}

fn visualize_effect_heal(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: Id,
    effect: &effect::Heal,
) -> ZResult<Box<dyn Action>> {
    let pos = state.parts().pos.get(target_id).0;
    let s = format!("healed +{}", effect.strength.0);
    Ok(seq(vec![
        action::Sleep::new(time_s(0.5)).boxed(),
        message(view, context, pos, &s)?,
        show_flare(view, context, pos, [0.0, 0.0, 0.9, 0.7].into())?,
    ]))
}

fn wound_msg(effect: &effect::Wound) -> String {
    let damage = effect.damage.0;
    let armor_break = effect.armor_break.0;
    if damage > 0 || armor_break > 0 {
        if armor_break == 0 {
            format!("-{} strength", damage)
        } else if damage == 0 {
            format!("-{} armor", armor_break)
        } else {
            format!("-{} strength -{} armor", damage, armor_break)
        }
    } else {
        "no damage".into()
    }
}

fn visualize_effect_wound(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: Id,
    effect: &effect::Wound,
) -> ZResult<Box<dyn Action>> {
    let id = target_id;
    let parts = state.parts();
    let pos = parts.pos.get(id).0;
    let sprite = view.id_to_sprite(id).clone();
    let c_normal = [1.0, 1.0, 1.0, 1.0].into();
    let c_dark = [0.1, 0.1, 0.1, 1.0].into();
    let time = time_s(0.2);
    let mut actions = Vec::new();
    let msg = wound_msg(effect);
    if effect.damage.0 > 0 || effect.armor_break.0 > 0 {
        let particles_count = effect.damage.0 * 4;
        actions.push(show_blood_particles(
            view,
            context,
            pos,
            effect.attacker_pos,
            particles_count,
        )?);
    }
    actions.push(message(view, context, pos, &msg)?);
    actions.push(action::ChangeColorTo::new(&sprite, c_dark, time).boxed());
    actions.push(action::ChangeColorTo::new(&sprite, c_normal, time).boxed());
    Ok(fork(seq(actions)))
}

fn visualize_effect_knockback(
    _: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: Id,
    effect: &effect::Knockback,
) -> ZResult<Box<dyn Action>> {
    if effect.from == effect.to {
        return message(view, context, effect.from, "Resisted knockback");
    }
    let from = view.hex_to_point(effect.from);
    let to = view.hex_to_point(effect.to);
    let diff = to - from;
    let time = time_s(0.15);
    Ok(fork(seq(vec![
        message(view, context, effect.to, "bump")?,
        move_object_with_shadow(view, target_id, diff, time),
    ])))
}

fn visualize_effect_fly_off(
    _: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: Id,
    effect: &effect::FlyOff,
) -> ZResult<Box<dyn Action>> {
    if effect.from == effect.to {
        return message(view, context, effect.from, "Resisted fly off");
    }
    let sprite_object = view.id_to_sprite(target_id).clone();
    let sprite_shadow = view.id_to_shadow_sprite(target_id).clone();
    let from = view.hex_to_point(effect.from);
    let to = view.hex_to_point(effect.to);
    let diff = to - from;
    let action_main_move = arc_move(view, &sprite_object, diff);
    let time = action_main_move.duration();
    let action_move_shadow = action::MoveBy::new(&sprite_shadow, diff, time).boxed();
    let action_dust = show_dust_at_pos(view, context, effect.to)?;
    Ok(fork(seq(vec![
        message(view, context, effect.to, "fly off")?,
        fork(action_move_shadow),
        action_main_move,
        action_dust,
    ])))
}

fn visualize_effect_throw(
    _: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: Id,
    effect: &effect::Throw,
) -> ZResult<Box<dyn Action>> {
    let sprite = view.id_to_sprite(target_id).clone();
    let sprite_shadow = view.id_to_shadow_sprite(target_id).clone();
    let from = view.hex_to_point(effect.from);
    let to = view.hex_to_point(effect.to);
    let diff = to - from;
    let arc_move = arc_move(view, &sprite, diff);
    let action_move_shadow = action::MoveBy::new(&sprite_shadow, diff, arc_move.duration()).boxed();
    let action_dust = show_dust_at_pos(view, context, effect.to)?;
    Ok(seq(vec![fork(action_move_shadow), arc_move, action_dust]))
}

fn visualize_effect_dodge(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: Id,
    effect: &effect::Dodge,
) -> ZResult<Box<dyn Action>> {
    let pos = state.parts().pos.get(target_id).0;
    let time_to = time_s(0.05);
    let time_from = time_s(0.3);
    let mut actions = Vec::new();
    actions.push(message(view, context, pos, "dodged")?);
    let point_a = view.hex_to_point(pos);
    let point_b = view.hex_to_point(effect.attacker_pos);
    let diff = (point_a - point_b).normalize() * view.tile_size() * 0.5;
    actions.push(move_object_with_shadow(view, target_id, diff, time_to));
    actions.push(move_object_with_shadow(view, target_id, -diff, time_from));
    Ok(seq(actions))
}
