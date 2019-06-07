use std::time::Duration;

use ggez::{
    graphics::{Color, Text},
    nalgebra,
    nalgebra::{Point2, Vector2},
    Context,
};
use log::{debug, info};
use rand::{thread_rng, Rng};
use scene::{action, Action, Boxed, Facing, Sprite};

use crate::{
    core::{
        battle::{
            ability::Ability,
            component::{Component, WeaponType},
            effect::{self, Effect},
            event::{self, ActiveEvent, Event},
            execute::{hit_chance, ApplyPhase},
            state, ObjId, PlayerId, State,
        },
        map::{Dir, PosHex},
    },
    geom,
    screen::battle::view::BattleView,
    utils::{font_size, time_s},
    ZResult,
};

const BLOOD_SPRITE_DURATION: i32 = 6;

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
    from: Option<Dir>,
    particles_count: i32,
) -> ZResult<Box<dyn Action>> {
    let point_origin = geom::hex_to_point(view.tile_size(), pos);
    let mut actions = Vec::new();
    for _ in 0..particles_count {
        let point = if let Some(dir) = from {
            let pos_neighbor = Dir::get_neighbor_pos(pos, dir);
            geom::hex_to_point(view.tile_size(), pos_neighbor)
                + geom::rand_tile_offset(view.tile_size(), 0.8)
        } else {
            point_origin + geom::rand_tile_offset(view.tile_size(), 1.7)
        };
        let color = [0.7, 0.0, 0.0, 0.6].into();
        let visible = color;
        let invisible = Color { a: 0.0, ..visible };
        let scale = thread_rng().gen_range(0.05, 0.15);
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
        view.add_disappearing_sprite(&layer, &sprite, BLOOD_SPRITE_DURATION, visible.a);
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
    let mut point = geom::hex_to_point(view.tile_size(), at);
    point.y += view.tile_size() * 0.1;
    sprite.set_pos(point);
    let color_final: Color = [1.0, 1.0, 1.0, 1.0].into();
    let time = time_s(0.3);
    let layer = view.layers().blood.clone();
    view.add_disappearing_sprite(&layer, &sprite, BLOOD_SPRITE_DURATION, color_final.a);
    Ok(seq(vec![
        action::Show::new(&layer, &sprite).boxed(),
        action::ChangeColorTo::new(&sprite, color_final, time).boxed(),
    ]))
}

fn show_dust_at_pos(
    view: &mut BattleView,
    context: &mut Context,
    at: PosHex,
) -> ZResult<Box<dyn Action>> {
    let point = geom::hex_to_point(view.tile_size(), at);
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
        let k = thread_rng().gen_range(0.8, 1.2);
        let visible = [0.8 * k, 0.8 * k, 0.7 * k, 0.8 * k].into();
        let invisible = Color { a: 0.0, ..visible };
        let scale = thread_rng().gen_range(0.2, 0.4);
        let size = view.tile_size() * 2.0 * scale;
        let vector = {
            let max = std::f32::consts::PI * 2.0;
            let rot = nalgebra::Rotation2::new((max / count as f32) * i as f32);
            let n = thread_rng().gen_range(0.4, 0.6);
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
    let point = geom::hex_to_point(tile_size, at) - Vector2::new(0.0, tile_size * 0.3);
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
    context: &mut Context,
    id: ObjId,
) -> ZResult<Box<dyn Action>> {
    let image = view.images().dot.clone();
    let mut actions = Vec::new();
    let parts = state.parts();
    let agent = parts.agent.get(id);
    let obj_pos = parts.pos.get(id).0;
    let strength = parts.strength.get(id);
    let damage = strength.base_strength.0 - strength.strength.0;
    let armor = state::get_armor(state, id);
    let size = 0.2 * view.tile_size();
    let mut point = geom::hex_to_point(view.tile_size(), obj_pos);
    point.x += view.tile_size() * 0.8;
    point.y -= view.tile_size() * 1.6;
    let mut dots = Vec::new();
    let base_x = point.x;
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
        point.x = base_x;
        point.y += size;
    }
    let mut sprites = Vec::new();
    for &(color, point) in &dots {
        let color = color.into();
        let mut sprite = Sprite::from_image(context, image.clone(), size)?;
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

struct SpriteInfo {
    paths: Vec<(&'static str, &'static str)>,
    offset_x: f32,
    offset_y: f32,
    shadow_size_coefficient: f32,
}

fn sprite_params(name: &str) -> SpriteInfo {
    // TODO: Move this table to a `.ron` config
    let (paths, offset_x, offset_y, shadow_size_coefficient) = match name {
        "swordsman" => (
            vec![("", "/swordsman.png"), ("rage", "/swordsman_rage.png")],
            0.15,
            0.1,
            1.0,
        ),
        "spearman" => (vec![("", "/spearman.png")], 0.2, 0.05, 1.0),
        "hammerman" => (vec![("", "/hammerman.png")], 0.05, 0.1, 1.0),
        "alchemist" => (
            vec![
                ("", "/alchemist.png"),
                ("throw", "/alchemist_throw.png"),
                ("heal", "/alchemist_heal.png"),
            ],
            0.05,
            0.1,
            1.0,
        ),
        "imp" => (vec![("", "/imp.png")], 0.0, 0.15, 1.3),
        "imp_toxic" => (vec![("", "/imp_toxic.png")], 0.0, 0.15, 1.2),
        "imp_bomber" => (
            vec![("", "/imp_bomber.png"), ("throw", "/imp_bomber_throw.png")],
            0.0,
            0.15,
            1.2,
        ),
        "imp_summoner" => (
            vec![
                ("", "/imp_summoner.png"),
                ("summon", "/imp_summoner_summon.png"),
            ],
            0.0,
            0.15,
            1.3,
        ),
        "boulder" => (vec![("", "/boulder.png")], 0.0, 0.4, 2.5),
        "bomb_damage" => (vec![("", "/bomb.png")], 0.0, 0.2, 0.7),
        "bomb_push" => (vec![("", "/bomb.png")], 0.0, 0.2, 0.7),
        "bomb_fire" => (vec![("", "/bomb_fire.png")], 0.0, 0.2, 0.7),
        "bomb_poison" => (vec![("", "/bomb_poison.png")], 0.0, 0.2, 0.7),
        "bomb_demonic" => (vec![("", "/bomb_demonic.png")], 0.0, 0.2, 0.7),
        "fire" => (vec![("", "/fire.png")], 0.0, 0.2, 0.001),
        "poison_cloud" => (vec![("", "/poison_cloud.png")], 0.0, 0.2, 2.0),
        "spike_trap" => (vec![("", "/spike_trap.png")], 0.0, 0.5, 1.4),
        _ => unimplemented!("Don't know such object type: {}", name),
    };
    SpriteInfo {
        paths,
        offset_x,
        offset_y,
        shadow_size_coefficient,
    }
}

pub fn refresh_brief_agent_info(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    id: ObjId,
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
    let sprite_shadow = view.id_to_shadow_sprite(event.id).clone();
    let mut actions = Vec::new();
    if let [pos] = event.path.tiles() {
        let action = fork(seq(vec![
            action::Sleep::new(time_s(0.4)).boxed(), // to avoid overlapping with a damage msg
            message(view, context, *pos, "move interrupted")?,
        ]));
        actions.push(action);
    }
    for step in event.path.steps() {
        let from = geom::hex_to_point(view.tile_size(), step.from);
        let to = geom::hex_to_point(view.tile_size(), step.to);
        let facing = geom::Facing::from_positions(view.tile_size(), step.from, step.to)
            .expect("Bad path step");
        let diff = to - from;
        let step_height = view.tile_size() * 0.25;
        let step_time = time_s(0.13);
        let move_time = time_s(0.3);
        let main_move = action::MoveBy::new(&sprite, diff, move_time).boxed();
        let shadow_move = action::MoveBy::new(&sprite_shadow, diff, move_time).boxed();
        let action = seq(vec![
            action::SetFacing::new(&sprite, facing.to_scene_facing()).boxed(),
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
    let chances = hit_chance(state, event.attacker_id, event.target_id);
    let attack_msg = format!("{}%", chances.1 * 10);
    actions.push(attack_message(view, context, from, &attack_msg)?);
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
    let facing_opt = geom::Facing::from_positions(view.tile_size(), map_from, map_to);
    if let Some(facing) = facing_opt {
        actions.push(action::SetFacing::new(&sprite, facing.to_scene_facing()).boxed());
    }
    if sprite.has_frame("attack") {
        actions.push(action::SetFrame::new(&sprite, "attack").boxed());
    }
    actions.push(fork(action_shadow_move_to));
    actions.push(action_sprite_move_to);
    actions.push(show_weapon_flash(
        view,
        context,
        map_to,
        event.weapon_type,
        facing_opt,
    )?);
    actions.push(fork(action_shadow_move_from));
    actions.push(action_sprite_move_from);
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
    let from = geom::hex_to_point(view.tile_size(), from);
    let to = geom::hex_to_point(view.tile_size(), event.pos);
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
    Ok(seq(vec![fork(action_flare), action_dust]))
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
        Ability::Heal(_) => visualize_event_use_ability_heal(state, view, context, event)?,
        Ability::Rage(_) => visualize_event_use_ability_rage(state, view, context, event)?,
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
    let text = event.ability.to_string();
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
    timed_effect: &effect::Timed,
) -> ZResult<Box<dyn Action>> {
    let pos = state.parts().pos.get(target_id).0;
    let action_flare = match timed_effect.effect {
        effect::Lasting::Poison => show_flare(view, context, pos, [0.0, 0.8, 0.0, 0.7].into())?,
        effect::Lasting::Stun => show_flare(view, context, pos, [1.0, 1.0, 1.0, 0.7].into())?,
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
        Effect::Kill(ref e) => visualize_effect_kill(state, view, context, target_id, e)?,
        Effect::Vanish => visualize_effect_vanish(state, view, context, target_id),
        Effect::Stun => visualize_effect_stun(state, view, context, target_id)?,
        Effect::Heal(ref e) => visualize_effect_heal(state, view, context, target_id, e)?,
        Effect::Wound(ref e) => visualize_effect_wound(state, view, context, target_id, e)?,
        Effect::Knockback(ref e) => visualize_effect_knockback(state, view, context, target_id, e)?,
        Effect::FlyOff(ref e) => visualize_effect_fly_off(state, view, context, target_id, e)?,
        Effect::Throw(ref e) => visualize_effect_throw(state, view, context, target_id, e)?,
        Effect::Dodge(ref e) => visualize_effect_dodge(state, view, context, target_id, e)?,
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
    let SpriteInfo {
        paths,
        offset_x,
        offset_y,
        shadow_size_coefficient,
    } = sprite_params(effect.prototype.0.as_str());
    let point = geom::hex_to_point(view.tile_size(), effect.pos);
    let color = [1.0, 1.0, 1.0, 1.0].into();
    let size = view.tile_size() * 2.0;
    let sprite_object = {
        let mut sprite = if let [path] = paths.as_slice() {
            Sprite::from_path(context, path.1, size)?
        } else {
            Sprite::from_paths(context, &paths, size)?
        };
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
    target_id: ObjId,
    effect: &effect::Kill,
) -> ZResult<Box<dyn Action>> {
    let particles_count = 6;
    let pos = state.parts().pos.get(target_id).0;
    Ok(fork(seq(vec![
        show_blood_particles(view, context, pos, effect.dir, particles_count)?,
        message(view, context, pos, "killed")?,
        vanish(view, target_id),
        show_blood_spot(view, context, pos)?,
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
    target_id: ObjId,
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
            effect.dir,
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
    let action_dust = show_dust_at_pos(view, context, effect.to)?;
    Ok(seq(vec![fork(action_move_shadow), arc_move, action_dust]))
}

fn visualize_effect_dodge(
    state: &State,
    view: &mut BattleView,
    context: &mut Context,
    target_id: ObjId,
    effect: &effect::Dodge,
) -> ZResult<Box<dyn Action>> {
    let pos = state.parts().pos.get(target_id).0;
    let time_to = time_s(0.05);
    let time_from = time_s(0.3);
    let mut actions = Vec::new();
    actions.push(message(view, context, pos, "dodged")?);
    if let Some(dir) = effect.dir {
        // TODO: code duplication with visualize_event_attack?
        let sprite = view.id_to_sprite(target_id).clone();
        let sprite_shadow = view.id_to_shadow_sprite(target_id).clone();
        let pos_b = Dir::get_neighbor_pos(pos, dir);
        let point_a = geom::hex_to_point(view.tile_size(), pos);
        let point_b = geom::hex_to_point(view.tile_size(), pos_b);
        let diff = (point_b - point_a) * 0.3;
        let action_sprite_move_to = action::MoveBy::new(&sprite, diff, time_to).boxed();
        let action_shadow_move_to = action::MoveBy::new(&sprite_shadow, diff, time_to).boxed();
        let action_sprite_move_from = action::MoveBy::new(&sprite, -diff, time_from).boxed();
        let action_shadow_move_from = action::MoveBy::new(&sprite_shadow, -diff, time_from).boxed();
        actions.push(fork(action_shadow_move_to));
        actions.push(action_sprite_move_to);
        actions.push(fork(action_shadow_move_from));
        actions.push(action_sprite_move_from);
    }
    Ok(seq(actions))
}
