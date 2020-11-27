use std::time::Duration;

use log::{info, trace};
use mq::{
    color::Color,
    math::glam::{Mat2, Vec2},
    texture::Texture2D,
};
use zscene::{action, Action, Boxed, Facing, Sprite};

use crate::{
    assets,
    core::{
        battle::{
            ability::Ability,
            component::{Component, WeaponType},
            effect::{self, Effect},
            event::{self, ActiveEvent, Event},
            execute::{hit_chance, ApplyPhase},
            state, Id, PlayerId, State, Turns,
        },
        map::PosHex,
        utils::roll_dice,
    },
    geom,
    screen::battle::view::BattleView,
    utils::{font_size, time_s},
    ZResult,
};

pub mod color {
    use mq::color::Color;

    pub const STRENGTH: Color = Color::new(0.0, 0.7, 0.0, 1.0);
    pub const DAMAGE: Color = Color::new(0.3, 0.5, 0.3, 0.5);
    pub const ARMOR: Color = Color::new(1.0, 1.0, 0.5, 1.0);
    pub const JOKERS: Color = Color::new(1.0, 1.0, 1.0, 1.0);
    pub const ATTACKS: Color = Color::new(1.0, 0.0, 0.0, 1.0);
    pub const MOVES: Color = Color::new(0.2, 0.2, 1.0, 1.0);
}

const BLOOD_SPRITE_DURATION_TURNS: Turns = Turns(6);
const TIME_LUNGE_TO: f32 = 0.1;
const TIME_LUNGE_FROM: f32 = 0.15;
const TIME_DEFAULT_FLARE: f32 = 0.4;

pub fn seq(actions: impl Into<Vec<Box<dyn Action>>>) -> Box<dyn Action> {
    action::Sequence::new(actions.into()).boxed()
}

pub fn fork(action: Box<dyn Action>) -> Box<dyn Action> {
    action::Fork::new(action).boxed()
}

fn action_set_z(layer: &zscene::Layer, sprite: &Sprite, z: f32) -> Box<dyn Action> {
    let sprite = sprite.clone();
    let mut layer = layer.clone();
    let closure = Box::new(move || {
        layer.set_z(&sprite, z);
    });
    action::Custom::new(closure).boxed()
}

fn hex_pos_to_z(pos: PosHex) -> f32 {
    pos.r as _
}

fn textures() -> &'static assets::Textures {
    &assets::get().textures
}

pub fn message(view: &mut BattleView, pos: PosHex, text: &str) -> ZResult<Box<dyn Action>> {
    let visible = [0.0, 0.0, 0.0, 1.0].into();
    let invisible = Color::new(0.0, 0.0, 0.0, 0.0);
    let font_size = font_size();
    let font = assets::get().font;
    let mut sprite = Sprite::from_text((text, font, font_size), 0.1);
    sprite.set_centered(true);
    let point = view.hex_to_point(pos);
    let point = point - Vec2::new(0.0, view.tile_size() * 1.5);
    sprite.set_pos(point);
    sprite.set_color(invisible);
    let action_show_hide = seq([
        action::Show::new(&view.layers().text, &sprite).boxed(),
        action::ChangeColorTo::new(&sprite, visible, time_s(0.4)).boxed(),
        action::Sleep::new(time_s(0.4)).boxed(),
        // TODO: read the time from Config:
        action::ChangeColorTo::new(&sprite, invisible, time_s(0.6)).boxed(),
        action::Hide::new(&view.layers().text, &sprite).boxed(),
    ]);
    let duration = action_show_hide.duration();
    let delta = -Vec2::new(0.0, 0.15);
    let action_move = action::MoveBy::new(&sprite, delta, duration).boxed();
    let mut actions = Vec::new();
    if let Some(delay) = view.messages_map().delay_at(pos) {
        actions.push(action::Sleep::new(delay).boxed());
    }
    view.messages_map_mut().register_message_at(pos, duration);
    actions.push(fork(action_move));
    actions.push(action_show_hide);
    Ok(fork(seq(actions)))
}

fn announce(view: &mut BattleView, text: &str, time: Duration) -> ZResult<Box<dyn Action>> {
    let height_text = 0.2;
    let height_bg = height_text * 5.0;
    let time_appear = time.mul_f32(0.25);
    let time_wait = time.mul_f32(0.5);
    let time_disappear = time.mul_f32(0.35);
    let action_show_and_hide = |sprite, color: Color| {
        let color_invisible = Color { a: 0.0, ..color };
        seq([
            action::SetColor::new(&sprite, color_invisible).boxed(),
            action::Show::new(&view.layers().text, &sprite).boxed(),
            action::ChangeColorTo::new(&sprite, color, time_appear).boxed(),
            action::Sleep::new(time_wait).boxed(),
            action::ChangeColorTo::new(&sprite, color_invisible, time_disappear).boxed(),
            action::Hide::new(&view.layers().text, &sprite).boxed(),
        ])
    };
    let actions_text = {
        let color = [0.0, 0.0, 0.0, 1.0].into();
        let font = assets::get().font;
        let mut sprite = Sprite::from_text((text, font, font_size()), height_text);
        sprite.set_centered(true);
        action_show_and_hide(sprite, color)
    };
    let actions_bg = {
        let color = [1.0, 1.0, 1.0, 0.5].into();
        let texture = textures().map.white_hex;
        let mut sprite = Sprite::from_texture(texture, height_bg);
        sprite.set_centered(true);
        action_show_and_hide(sprite, color)
    };
    Ok(seq([
        fork(actions_bg),
        action::Sleep::new(time_s(0.01)).boxed(), // delay the text a little
        actions_text,
    ]))
}

fn attack_message(view: &mut BattleView, pos: Vec2, text: &str) -> ZResult<Box<dyn Action>> {
    let visible = [0.0, 0.0, 0.0, 1.0].into();
    let invisible = [0.0, 0.0, 0.0, 0.0].into();
    let font_size = font_size();
    let font = assets::get().font;
    let mut sprite = Sprite::from_text((text, font, font_size), 0.1);
    sprite.set_centered(true);
    let point = pos + Vec2::new(0.0, view.tile_size() * 0.5);
    sprite.set_pos(point);
    sprite.set_color(invisible);
    let action_show_hide = seq([
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
        let visible = [0.7, 0.0, 0.0, 0.6].into();
        let invisible = Color { a: 0.0, ..visible };
        let scale = roll_dice(0.05, 0.15);
        let size = view.tile_size() * 2.0 * scale;
        let mut sprite = Sprite::from_texture(textures().map.white_hex, size);
        sprite.set_centered(true);
        sprite.set_pos(point_origin);
        sprite.set_color(invisible);
        let vector = point - point_origin;
        let layer = view.layers().blood.clone();
        actions.push(fork(seq([
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

fn show_blood_spot(view: &mut BattleView, at: PosHex) -> ZResult<Box<dyn Action>> {
    let mut sprite = Sprite::from_texture(textures().map.blood, view.tile_size() * 2.0);
    sprite.set_centered(true);
    sprite.set_color([1.0, 1.0, 1.0, 0.0].into());
    sprite.set_pos(view.hex_to_point(at) + Vec2::new(0.0, view.tile_size() * 0.1));
    let color_final: Color = [1.0, 1.0, 1.0, 1.0].into();
    let time = time_s(0.6);
    let layer = view.layers().blood.clone();
    let duration = BLOOD_SPRITE_DURATION_TURNS;
    view.add_disappearing_sprite(&layer, &sprite, duration, color_final.a);
    Ok(seq([
        action::Show::new(&layer, &sprite).boxed(),
        action::ChangeColorTo::new(&sprite, color_final, time).boxed(),
    ]))
}

fn show_explosion_ground_mark(view: &mut BattleView, at: PosHex) -> ZResult<Box<dyn Action>> {
    let tex = textures().map.explosion_ground_mark;
    let mut sprite = Sprite::from_texture(tex, view.tile_size() * 2.0);
    sprite.set_centered(true);
    sprite.set_color([1.0, 1.0, 1.0, 1.0].into());
    sprite.set_pos(view.hex_to_point(at));
    let layer = view.layers().blood.clone();
    let duration = BLOOD_SPRITE_DURATION_TURNS;
    view.add_disappearing_sprite(&layer, &sprite, duration, sprite.color().a);
    Ok(action::Show::new(&layer, &sprite).boxed())
}

fn show_dust_at_pos(view: &mut BattleView, at: PosHex) -> ZResult<Box<dyn Action>> {
    let point = view.hex_to_point(at);
    let count = 9;
    show_dust(view, point, count)
}

fn show_dust(view: &mut BattleView, at: Vec2, count: i32) -> ZResult<Box<dyn Action>> {
    let mut actions = Vec::new();
    for i in 0..count {
        let k = roll_dice(0.8, 1.2);
        let visible = [0.8 * k, 0.8 * k, 0.7 * k, 0.8 * k].into();
        let invisible = [0.8 * k, 0.8 * k, 0.7 * k, 0.0].into();
        let scale = roll_dice(0.2, 0.4);
        let size = view.tile_size() * 2.0 * scale;
        let vector = {
            let max = std::f32::consts::PI * 2.0;
            let rot = Mat2::from_angle((max / count as f32) * i as f32);
            let n = roll_dice(0.3, 0.6);
            let mut vector = rot * Vec2::new(view.tile_size() * n, 0.0);
            vector.set_y(vector.y() * geom::FLATNESS_COEFFICIENT);
            vector
        };
        let point = at + vector;
        let sprite = {
            let mut sprite = Sprite::from_texture(textures().map.white_hex, size);
            sprite.set_centered(true);
            sprite.set_pos(point);
            sprite.set_color(invisible);
            sprite
        };
        let layer = &view.layers().particles;
        let action_show_hide = seq([
            action::Show::new(layer, &sprite).boxed(),
            action::ChangeColorTo::new(&sprite, visible, time_s(0.2)).boxed(),
            action::ChangeColorTo::new(&sprite, invisible, time_s(0.7)).boxed(),
            action::Hide::new(layer, &sprite).boxed(),
        ]);
        let time = action_show_hide.duration();
        let action_move = action::MoveBy::new(&sprite, vector, time).boxed();
        let action = seq([fork(action_move), fork(action_show_hide)]);
        actions.push(action);
    }
    Ok(seq(actions))
}

fn show_flare_scale_time(
    view: &mut BattleView,
    at: PosHex,
    color: Color,
    scale: f32,
    time: Duration,
) -> ZResult<Box<dyn Action>> {
    let visible = color;
    let invisible = Color { a: 0.0, ..visible };
    let size = view.tile_size() * 2.0 * scale;
    let mut sprite = Sprite::from_texture(textures().map.white_hex, size);
    let point = view.hex_to_point(at);
    sprite.set_centered(true);
    sprite.set_pos(point);
    sprite.set_color(invisible);
    Ok(seq([
        action::Show::new(&view.layers().flares, &sprite).boxed(),
        action::ChangeColorTo::new(&sprite, visible, time.mul_f32(0.25)).boxed(),
        action::ChangeColorTo::new(&sprite, invisible, time.mul_f32(0.75)).boxed(),
        action::Hide::new(&view.layers().flares, &sprite).boxed(),
    ]))
}

fn show_weapon_flash(
    view: &mut BattleView,
    at: PosHex,
    weapon_type: WeaponType,
    facing_opt: Option<geom::Facing>,
) -> ZResult<Box<dyn Action>> {
    let textures = &assets::get().textures.weapon_flashes;
    let visible = [1.0, 1.0, 1.0, 0.8].into();
    let invisible = [1.0, 1.0, 1.0, 0.1].into();
    let tile_size = view.tile_size();
    let sprite_size = tile_size * 2.0;
    let texture = textures.get(&weapon_type).expect("No such attack flash");
    let mut sprite = Sprite::from_texture(*texture, sprite_size);
    let point = view.hex_to_point(at) - Vec2::new(0.0, tile_size * 0.3);
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

fn show_flare(view: &mut BattleView, at: PosHex, color: Color) -> ZResult<Box<dyn Action>> {
    let scale = 1.0;
    show_flare_scale_time(view, at, color, scale, time_s(TIME_DEFAULT_FLARE))
}

fn up_and_down_move(
    _: &mut BattleView,
    sprite: &Sprite,
    height: f32,
    time: Duration,
) -> Box<dyn Action> {
    let duration_0_25 = time.mul_f32(0.25);
    let up_fast = Vec2::new(0.0, -height * 0.75);
    let up_slow = Vec2::new(0.0, -height * 0.25);
    let down_slow = -up_slow;
    let down_fast = -up_fast;
    seq([
        action::MoveBy::new(sprite, up_fast, duration_0_25).boxed(),
        action::MoveBy::new(sprite, up_slow, duration_0_25).boxed(),
        action::MoveBy::new(sprite, down_slow, duration_0_25).boxed(),
        action::MoveBy::new(sprite, down_fast, duration_0_25).boxed(),
    ])
}

fn move_object_with_shadow(
    view: &mut BattleView,
    id: Id,
    diff: Vec2,
    time_to: Duration,
) -> Box<dyn Action> {
    let sprite = view.id_to_sprite(id).clone();
    let sprite_shadow = view.id_to_shadow_sprite(id).clone();
    let action_sprite_move_to = action::MoveBy::new(&sprite, diff, time_to).boxed();
    let action_shadow_move_to = action::MoveBy::new(&sprite_shadow, diff, time_to).boxed();
    seq([fork(action_shadow_move_to), action_sprite_move_to])
}

fn arc_move(view: &mut BattleView, sprite: &Sprite, diff: Vec2) -> Box<dyn Action> {
    let len = diff.length();
    let min_height = view.tile_size() * 0.5;
    let base_height = view.tile_size() * 2.0;
    let min_time = 0.25;
    let base_time = 0.3;
    let height = min_height + base_height * (len / 1.0);
    let time = time_s(min_time + base_time * (len / 1.0));
    let up_and_down = up_and_down_move(view, sprite, height, time);
    let main_move = action::MoveBy::new(sprite, diff, time).boxed();
    seq([fork(main_move), up_and_down])
}

fn vanish_with_duration(view: &mut BattleView, target_id: Id, time: Duration) -> Box<dyn Action> {
    trace!("vanish target_id={:?}", target_id);
    let sprite = view.id_to_sprite(target_id).clone();
    let sprite_shadow = view.id_to_shadow_sprite(target_id).clone();
    view.remove_object(target_id);
    let dark = [0.1, 0.1, 0.1, 1.0].into();
    let invisible = [0.1, 0.1, 0.1, 0.0].into();
    let time_div_3 = time.div_f32(3.0);
    seq([
        action::ChangeColorTo::new(&sprite, dark, time_div_3).boxed(),
        action::ChangeColorTo::new(&sprite, invisible, time_div_3).boxed(),
        action::Hide::new(&view.layers().objects, &sprite).boxed(),
        action::ChangeColorTo::new(&sprite_shadow, invisible, time_div_3).boxed(),
        action::Hide::new(&view.layers().shadows, &sprite_shadow).boxed(),
    ])
}

fn show_frame_for_time(
    view: &mut BattleView,
    id: Id,
    frame_name: &str,
    time: Duration,
) -> ZResult<Box<dyn Action>> {
    let sprite = view.id_to_sprite(id).clone();
    assert!(sprite.has_frame(frame_name));
    Ok(seq([
        action::SetFrame::new(&sprite, frame_name).boxed(),
        action::Sleep::new(time).boxed(),
        action::SetFrame::new(&sprite, "").boxed(),
    ]))
}

fn remove_brief_agent_info(view: &mut BattleView, id: Id) -> ZResult<Box<dyn Action>> {
    let mut actions = Vec::new();
    let sprites = view.agent_info_get(id);
    for sprite in sprites {
        let color = sprite.color();
        let color = Color { a: 0.0, ..color };
        actions.push(fork(seq([
            action::ChangeColorTo::new(&sprite, color, time_s(0.4)).boxed(),
            action::Hide::new(&view.layers().dots, &sprite).boxed(),
        ])));
    }
    Ok(seq(actions))
}

pub fn get_effect_icon(effect: &effect::Lasting) -> Texture2D {
    let effects = &assets::get().textures.icons.lasting_effects;
    *effects.get(&effect).expect("No such effect found")
}

fn generate_brief_obj_info(
    state: &State,
    view: &mut BattleView,
    id: Id,
) -> ZResult<Box<dyn Action>> {
    let mut actions = Vec::new();
    let parts = state.parts();
    let agent = parts.agent.get(id);
    let obj_pos = parts.pos.get(id).0;
    let strength = parts.strength.get(id);
    let damage = strength.base_strength.0 - strength.strength.0;
    let armor = state::get_armor(state, id);
    let size = 0.25 * view.tile_size();
    let mut point = view.hex_to_point(obj_pos);
    point += Vec2::new(view.tile_size() * 0.8, -view.tile_size() * 1.6);
    let mut dots = Vec::new();
    let base = point;
    let rows: &[&[_]] = &[
        &[
            (color::STRENGTH, strength.strength.0),
            (color::DAMAGE, damage),
            (color::ARMOR, armor.0),
        ],
        &[(color::JOKERS, agent.jokers.0)],
        &[(color::ATTACKS, agent.attacks.0)],
        &[(color::MOVES, agent.moves.0)],
    ];
    let actual_dot_size_k = 0.8;
    for &row in rows {
        for &(color, n) in row {
            for _ in 0..n {
                dots.push((color, point));
                *point.x_mut() -= size * actual_dot_size_k;
            }
        }
        *point.x_mut() = base.x();
        *point.y_mut() += size * actual_dot_size_k;
    }
    let mut sprites = Vec::new();
    for &(color, point) in &dots {
        let mut sprite = Sprite::from_texture(textures().dot, size);
        sprite.set_centered(true);
        sprite.set_pos(point);
        sprite.set_color(Color { a: 0.0, ..color });
        let action = fork(seq([
            action::Show::new(&view.layers().dots, &sprite).boxed(),
            action::ChangeColorTo::new(&sprite, color, time_s(0.1)).boxed(),
        ]));
        sprites.push(sprite);
        actions.push(action);
    }
    {
        let health_points = strength.strength.0 + damage + armor.0;
        let health_bar_width = health_points as f32 * size * actual_dot_size_k;
        if let Some(effects) = parts.effects.get_opt(id) {
            let icon_size = size * 1.6;
            let mut icon_point = base - Vec2::new(icon_size, health_bar_width + icon_size * 0.4);
            for timed_effect in &effects.0 {
                *icon_point.y_mut() += icon_size;
                let effect = &timed_effect.effect;
                let texture = get_effect_icon(effect);
                let mut sprite = Sprite::from_texture(texture, icon_size);
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

fn refresh_brief_agent_info(
    state: &State,
    view: &mut BattleView,
    id: Id,
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
    event: &Event,
    phase: ApplyPhase,
) -> ZResult<Box<dyn Action>> {
    trace!("visualize: phase={:?} event={:?}", phase, event);
    match phase {
        ApplyPhase::Pre => visualize_pre(state, view, event),
        ApplyPhase::Post => visualize_post(state, view, event),
    }
}

fn visualize_pre(state: &State, view: &mut BattleView, event: &Event) -> ZResult<Box<dyn Action>> {
    let mut actions = Vec::new();
    actions.push(visualize_event(state, view, &event.active_event)?);
    for &(id, ref effects) in &event.instant_effects {
        for effect in effects {
            actions.push(visualize_instant_effect(state, view, id, &effect)?);
        }
    }
    for &(id, ref effects) in &event.timed_effects {
        for effect in effects {
            actions.push(visualize_lasting_effect(state, view, id, &effect)?);
        }
    }
    Ok(seq(actions))
}

fn visualize_post(state: &State, view: &mut BattleView, event: &Event) -> ZResult<Box<dyn Action>> {
    let mut actions = Vec::new();
    for &id in &event.actor_ids {
        actions.push(refresh_brief_agent_info(state, view, id)?);
    }
    for &(id, _) in &event.instant_effects {
        actions.push(refresh_brief_agent_info(state, view, id)?);
    }
    for &(id, _) in &event.timed_effects {
        actions.push(refresh_brief_agent_info(state, view, id)?);
    }
    Ok(seq(actions))
}

fn visualize_event(
    state: &State,
    view: &mut BattleView,
    event: &ActiveEvent,
) -> ZResult<Box<dyn Action>> {
    info!("{:?}", event);
    let action = match *event {
        ActiveEvent::UsePassiveAbility(_) | ActiveEvent::Create => action::Empty::new().boxed(),
        ActiveEvent::MoveTo(ref ev) => visualize_event_move_to(state, view, ev)?,
        ActiveEvent::Attack(ref ev) => visualize_event_attack(state, view, ev)?,
        ActiveEvent::EndBattle(ref ev) => visualize_event_end_battle(state, view, ev)?,
        ActiveEvent::EndTurn(ref ev) => visualize_event_end_turn(state, view, ev),
        ActiveEvent::BeginTurn(ref ev) => visualize_event_begin_turn(state, view, ev)?,
        ActiveEvent::EffectTick(ref ev) => visualize_event_effect_tick(state, view, ev)?,
        ActiveEvent::EffectEnd(ref ev) => visualize_event_effect_end(state, view, ev)?,
        ActiveEvent::UseAbility(ref ev) => visualize_event_use_ability(state, view, ev)?,
    };
    Ok(action)
}

fn visualize_event_move_to(
    _: &State,
    view: &mut BattleView,
    event: &event::MoveTo,
) -> ZResult<Box<dyn Action>> {
    let sprite = view.id_to_sprite(event.id).clone();
    let mut actions = Vec::new();
    if let [pos] = event.path.tiles() {
        actions.push(message(view, *pos, "move interrupted")?);
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
        let action = seq([
            action::SetFacing::new(&sprite, facing.to_scene_facing()).boxed(),
            action_set_z(&view.layers().objects, &sprite, hex_pos_to_z(step.to)),
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
    actions.push(attack_message(view, from, &attack_msg)?);
    if event.mode == event::AttackMode::Reactive {
        actions.push(message(view, map_from, "reaction")?);
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
    event: &event::EndBattle,
) -> ZResult<Box<dyn Action>> {
    let text = match event.result.winner_id {
        PlayerId(0) => "YOU WON!",
        PlayerId(1) => "YOU LOSE!",
        _ => unreachable!(),
    };
    Ok(seq([
        action::Sleep::new(time_s(1.0)).boxed(),
        announce(view, text, time_s(4.0))?,
        action::Sleep::new(time_s(1.0)).boxed(),
    ]))
}

fn visualize_event_end_turn(
    _: &State,
    view: &mut BattleView,
    _: &event::EndTurn,
) -> Box<dyn Action> {
    seq([
        view.update_disappearing_sprites(),
        action::Sleep::new(time_s(0.2)).boxed(),
    ])
}

fn visualize_event_begin_turn(
    _: &State,
    view: &mut BattleView,
    event: &event::BeginTurn,
) -> ZResult<Box<dyn Action>> {
    let text = match event.player_id {
        PlayerId(0) => "YOUR TURN",
        PlayerId(1) => "ENEMY TURN",
        _ => unreachable!(),
    };
    announce(view, text, time_s(1.5))
}

fn visualize_event_use_ability_jump(
    state: &State,
    view: &mut BattleView,
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
    let z = hex_pos_to_z(event.pos);
    let action_move_shadow = action::MoveBy::new(&sprite_shadow, diff, time).boxed();
    let action_dust = show_dust_at_pos(view, event.pos)?;
    let mut actions = Vec::new();
    actions.push(action_set_z(&view.layers().objects, &sprite_object, 200.0));
    if sprite_object.has_frame("jump") {
        actions.push(action::SetFrame::new(&sprite_object, "jump").boxed());
    }
    actions.push(fork(action_move_shadow));
    actions.push(action_arc_move);
    actions.push(action_set_z(&view.layers().objects, &sprite_object, z));
    if sprite_object.has_frame("jump") {
        actions.push(action::SetFrame::new(&sprite_object, "").boxed());
    }
    actions.push(action_dust);
    Ok(seq(actions))
}

fn visualize_event_use_ability_dash(
    state: &State,
    view: &mut BattleView,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    let sprite = view.id_to_sprite(event.id).clone();
    let z = hex_pos_to_z(event.pos);
    let from = state.parts().pos.get(event.id).0;
    let point_from = view.hex_to_point(from);
    let point_to = view.hex_to_point(event.pos);
    let diff = point_to - point_from;
    let time = time_s(0.1);
    Ok(seq([
        action_set_z(&view.layers().objects, &sprite, z),
        move_object_with_shadow(view, event.id, diff, time),
    ]))
}

fn visualize_event_use_ability_heal(
    _: &State,
    view: &mut BattleView,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    let time = time_s(1.0);
    Ok(fork(show_frame_for_time(view, event.id, "heal", time)?))
}

fn visualize_event_use_ability_rage(
    _: &State,
    view: &mut BattleView,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    let time = time_s(1.0);
    Ok(fork(show_frame_for_time(view, event.id, "rage", time)?))
}

fn visualize_event_use_ability_knockback(
    state: &State,
    view: &mut BattleView,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    lunge(state, view, event.id, event.pos)
}

fn visualize_event_use_ability_club(
    state: &State,
    view: &mut BattleView,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    lunge(state, view, event.id, event.pos)
}

fn visualize_event_use_ability_explode(
    state: &State,
    view: &mut BattleView,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    let pos = state.parts().pos.get(event.id).0;
    let action_dust = show_dust_at_pos(view, pos)?;
    let color = [1.0, 0.0, 0.0, 0.7].into();
    let scale = 2.5;
    let time = time_s(TIME_DEFAULT_FLARE * 0.8);
    let action_flare = show_flare_scale_time(view, pos, color, scale, time)?;
    let action_ground_mark = show_explosion_ground_mark(view, pos)?;
    Ok(seq([
        fork(seq([action_flare, action_dust])),
        action_ground_mark,
    ]))
}

fn visualize_event_use_ability_summon(
    state: &State,
    view: &mut BattleView,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    let sprite = view.id_to_sprite(event.id).clone();
    let frame_name = "summon";
    assert!(sprite.has_frame(frame_name));
    let pos = state.parts().pos.get(event.id).0;
    let color = [1.0, 1.0, 1.0, 0.7].into();
    let scale = 2.0;
    let time = time_s(TIME_DEFAULT_FLARE);
    let action_flare = show_flare_scale_time(view, pos, color, scale, time)?;
    Ok(seq([
        action::SetFrame::new(&sprite, frame_name).boxed(),
        action::Sleep::new(time_s(0.3)).boxed(),
        fork(seq([
            action_flare,
            action::SetFrame::new(&sprite, "").boxed(),
        ])),
    ]))
}

fn visualize_event_use_ability_bloodlust(
    _: &State,
    view: &mut BattleView,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    let time = time_s(0.5);
    show_frame_for_time(view, event.id, "bloodlust", time)
}

fn visualize_event_use_ability_throw_bomb(
    _: &State,
    view: &mut BattleView,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    let time = time_s(0.5);
    Ok(fork(show_frame_for_time(view, event.id, "throw", time)?))
}

fn visualize_event_use_ability(
    state: &State,
    view: &mut BattleView,
    event: &event::UseAbility,
) -> ZResult<Box<dyn Action>> {
    let action_main = match event.ability {
        Ability::Jump | Ability::LongJump => visualize_event_use_ability_jump(state, view, event)?,
        Ability::Dash => visualize_event_use_ability_dash(state, view, event)?,
        Ability::Summon => visualize_event_use_ability_summon(state, view, event)?,
        Ability::Bloodlust => visualize_event_use_ability_bloodlust(state, view, event)?,
        Ability::Heal | Ability::GreatHeal => visualize_event_use_ability_heal(state, view, event)?,
        Ability::Rage => visualize_event_use_ability_rage(state, view, event)?,
        Ability::Knockback => visualize_event_use_ability_knockback(state, view, event)?,
        Ability::Club => visualize_event_use_ability_club(state, view, event)?,
        Ability::ExplodePush
        | Ability::ExplodeDamage
        | Ability::ExplodeFire
        | Ability::ExplodePoison => visualize_event_use_ability_explode(state, view, event)?,
        Ability::BombPush
        | Ability::BombDemonic
        | Ability::BombFire
        | Ability::BombPoison
        | Ability::Bomb => visualize_event_use_ability_throw_bomb(state, view, event)?,
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
    // Don't show messages for not that important abilities.
    match event.ability {
        Ability::Vanish => {}
        _ => actions.push(message(view, pos, &text)?),
    }
    Ok(seq(actions))
}

fn visualize_event_effect_tick(
    state: &State,
    view: &mut BattleView,
    event: &event::EffectTick,
) -> ZResult<Box<dyn Action>> {
    let pos = state.parts().pos.get(event.id).0;
    match event.effect {
        effect::Lasting::Poison => show_flare(view, pos, [0.0, 0.8, 0.0, 0.7].into()),
        effect::Lasting::Stun => show_flare(view, pos, [1.0, 1.0, 1.0, 0.7].into()),
        effect::Lasting::Bloodlust => show_flare(view, pos, [1.0, 0.0, 0.0, 0.5].into()),
    }
}

fn visualize_event_effect_end(
    state: &State,
    view: &mut BattleView,
    event: &event::EffectEnd,
) -> ZResult<Box<dyn Action>> {
    let pos = state.parts().pos.get(event.id).0;
    let s = event.effect.title();
    message(view, pos, &format!("[{}] ended", s))
}

fn visualize_lasting_effect(
    state: &State,
    view: &mut BattleView,
    target_id: Id,
    timed_effect: &effect::Timed,
) -> ZResult<Box<dyn Action>> {
    let pos = state.parts().pos.get(target_id).0;
    let action_flare = match timed_effect.effect {
        effect::Lasting::Poison => show_flare(view, pos, [0.0, 0.8, 0.0, 0.7].into())?,
        effect::Lasting::Stun => show_flare(view, pos, [1.0, 1.0, 1.0, 0.7].into())?,
        effect::Lasting::Bloodlust => show_flare(view, pos, [1.0, 0.0, 0.0, 0.5].into())?,
    };
    let s = timed_effect.effect.title();
    Ok(seq([
        action_flare,
        message(view, pos, &format!("[{}]", s))?,
    ]))
}

fn visualize_instant_effect(
    state: &State,
    view: &mut BattleView,
    target_id: Id,
    effect: &Effect,
) -> ZResult<Box<dyn Action>> {
    trace!("visualize_instant_effect: {:?}", effect);
    let action = match *effect {
        Effect::Create(ref e) => visualize_effect_create(state, view, target_id, e)?,
        Effect::Kill(ref e) => visualize_effect_kill(state, view, target_id, e)?,
        Effect::Vanish => visualize_effect_vanish(state, view, target_id),
        Effect::Stun => visualize_effect_stun(state, view, target_id)?,
        Effect::Heal(ref e) => visualize_effect_heal(state, view, target_id, e)?,
        Effect::Wound(ref e) => visualize_effect_wound(state, view, target_id, e)?,
        Effect::Knockback(ref e) => visualize_effect_knockback(state, view, target_id, e)?,
        Effect::FlyOff(ref e) => visualize_effect_fly_off(state, view, target_id, e)?,
        Effect::Throw(ref e) => visualize_effect_throw(state, view, target_id, e)?,
        Effect::Dodge(ref e) => visualize_effect_dodge(state, view, target_id, e)?,
        Effect::Bloodlust => action::Empty.boxed(),
    };
    Ok(action)
}

fn visualize_effect_create(
    _: &State,
    view: &mut BattleView,
    target_id: Id,
    effect: &effect::Create,
) -> ZResult<Box<dyn Action>> {
    let info = &assets::get().sprites_info[&effect.prototype];
    let z = hex_pos_to_z(effect.pos) + info.sub_tile_z;
    let point = view.hex_to_point(effect.pos);
    let color = Color::new(1.0, 1.0, 1.0, 1.0);
    let size = view.tile_size() * 2.0;
    let sprite_object = {
        let mut sprite = view.object_sprite(&effect.prototype);
        sprite.set_color(Color { a: 0.0, ..color });
        sprite.set_pos(point);
        // Turn enemies left.
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
        let tex = textures().map.shadow;
        let mut sprite = Sprite::from_texture(tex, size * info.shadow_size_coefficient);
        sprite.set_centered(true);
        sprite.set_color(Color { a: 0.0, ..color });
        sprite.set_pos(point);
        sprite
    };
    view.add_object(target_id, &sprite_object, &sprite_shadow);
    let time_appear = time_s(0.2);
    let action_change_shadow_color =
        action::ChangeColorTo::new(&sprite_shadow, color, time_appear).boxed();
    let mut actions = Vec::new();
    if effect.is_teleported {
        let white = [1.0, 1.0, 1.0, 0.9].into();
        let scale = 0.9;
        let mut teleportation_flare =
            |time| show_flare_scale_time(view, effect.pos, white, scale, time);
        actions.push(fork(teleportation_flare(time_s(0.2))?));
        actions.push(fork(teleportation_flare(time_s(1.0))?));
    }
    actions.push(action::Show::new(&view.layers().shadows, &sprite_shadow).boxed());
    actions.push(action::Show::new(&view.layers().objects, &sprite_object).boxed());
    actions.push(action_set_z(&view.layers().objects, &sprite_object, z));
    actions.push(fork(action_change_shadow_color));
    actions.push(action::ChangeColorTo::new(&sprite_object, color, time_appear).boxed());
    Ok(fork(seq(actions)))
}

fn visualize_effect_kill(
    state: &State,
    view: &mut BattleView,
    target_id: Id,
    effect: &effect::Kill,
) -> ZResult<Box<dyn Action>> {
    let particles_count = 6;
    let pos = state.parts().pos.get(target_id).0;
    Ok(fork(seq([
        show_blood_particles(view, pos, effect.attacker_pos, particles_count)?,
        message(view, pos, "killed")?,
        fork(show_blood_spot(view, pos)?),
        vanish_with_duration(view, target_id, time_s(1.5)),
    ])))
}

fn visualize_effect_vanish(_: &State, view: &mut BattleView, target_id: Id) -> Box<dyn Action> {
    fork(vanish_with_duration(view, target_id, time_s(1.2)))
}

fn visualize_effect_stun(
    _state: &State,
    _view: &mut BattleView,
    _target_id: Id,
) -> ZResult<Box<dyn Action>> {
    Ok(fork(action::Sleep::new(time_s(1.0)).boxed()))
}

fn visualize_effect_heal(
    state: &State,
    view: &mut BattleView,
    target_id: Id,
    effect: &effect::Heal,
) -> ZResult<Box<dyn Action>> {
    let pos = state.parts().pos.get(target_id).0;
    let s = format!("healed +{}", effect.strength.0);
    Ok(seq([
        action::Sleep::new(time_s(0.5)).boxed(),
        message(view, pos, &s)?,
        show_flare(view, pos, [0.0, 0.0, 0.9, 0.7].into())?,
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
            format!("-{} strength & {} armor", damage, armor_break)
        }
    } else {
        "no damage".into()
    }
}

fn visualize_effect_wound(
    state: &State,
    view: &mut BattleView,
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
        let count = effect.damage.0 * 3;
        let from = effect.attacker_pos;
        actions.push(show_blood_particles(view, pos, from, count)?);
    }
    actions.push(message(view, pos, &msg)?);
    actions.push(action::ChangeColorTo::new(&sprite, c_dark, time).boxed());
    actions.push(action::ChangeColorTo::new(&sprite, c_normal, time).boxed());
    Ok(fork(seq(actions)))
}

fn visualize_effect_knockback(
    _: &State,
    view: &mut BattleView,
    target_id: Id,
    effect: &effect::Knockback,
) -> ZResult<Box<dyn Action>> {
    if effect.from == effect.to {
        return message(view, effect.from, "Resisted knockback");
    }
    let sprite = view.id_to_sprite(target_id).clone();
    let z = hex_pos_to_z(effect.to);
    let from = view.hex_to_point(effect.from);
    let to = view.hex_to_point(effect.to);
    let diff = to - from;
    let time = time_s(0.15);
    Ok(fork(seq([
        message(view, effect.to, "bump")?,
        action_set_z(&view.layers().objects, &sprite, z),
        move_object_with_shadow(view, target_id, diff, time),
    ])))
}

fn visualize_effect_fly_off(
    _: &State,
    view: &mut BattleView,
    target_id: Id,
    effect: &effect::FlyOff,
) -> ZResult<Box<dyn Action>> {
    if effect.from == effect.to {
        return message(view, effect.from, "Resisted fly off");
    }
    let sprite_object = view.id_to_sprite(target_id).clone();
    let sprite_shadow = view.id_to_shadow_sprite(target_id).clone();
    let z = hex_pos_to_z(effect.to);
    let from = view.hex_to_point(effect.from);
    let to = view.hex_to_point(effect.to);
    let diff = to - from;
    let action_main_move = arc_move(view, &sprite_object, diff);
    let time = action_main_move.duration();
    let action_move_shadow = action::MoveBy::new(&sprite_shadow, diff, time).boxed();
    let action_dust = show_dust_at_pos(view, effect.to)?;
    Ok(fork(seq([
        fork(action_move_shadow),
        action_set_z(&view.layers().objects, &sprite_object, z),
        action_main_move,
        message(view, effect.to, "fly off")?,
        action_dust,
    ])))
}

fn visualize_effect_throw(
    _: &State,
    view: &mut BattleView,
    target_id: Id,
    effect: &effect::Throw,
) -> ZResult<Box<dyn Action>> {
    let sprite = view.id_to_sprite(target_id).clone();
    let sprite_shadow = view.id_to_shadow_sprite(target_id).clone();
    let z = hex_pos_to_z(effect.to);
    let from = view.hex_to_point(effect.from);
    let to = view.hex_to_point(effect.to);
    let diff = to - from;
    let arc_move = arc_move(view, &sprite, diff);
    let action_move_shadow = action::MoveBy::new(&sprite_shadow, diff, arc_move.duration()).boxed();
    Ok(seq([
        fork(action_move_shadow),
        arc_move,
        action_set_z(&view.layers().objects, &sprite, z),
        show_dust_at_pos(view, effect.to)?,
    ]))
}

fn visualize_effect_dodge(
    state: &State,
    view: &mut BattleView,
    target_id: Id,
    effect: &effect::Dodge,
) -> ZResult<Box<dyn Action>> {
    let pos = state.parts().pos.get(target_id).0;
    let time_to = time_s(0.05);
    let time_from = time_s(0.3);
    let mut actions = Vec::new();
    actions.push(message(view, pos, "dodged")?);
    let point_a = view.hex_to_point(pos);
    let point_b = view.hex_to_point(effect.attacker_pos);
    let diff = (point_a - point_b).normalize() * view.tile_size() * 0.5;
    actions.push(move_object_with_shadow(view, target_id, diff, time_to));
    actions.push(move_object_with_shadow(view, target_id, -diff, time_from));
    Ok(seq(actions))
}
