use std::{collections::HashMap, default::Default, time::Duration};

use ggez::{
    graphics::{Color, Font, Image, Text},
    nalgebra::Point2,
    Context,
};
use rand::{thread_rng, Rng};
use scene::{action, Action, Boxed, Layer, Scene, Sprite};

use crate::{
    core::{
        battle::{
            self, ability::Ability, command, component::ObjType, execute::hit_chance, movement, Id,
            Jokers, Moves, State, TileType,
        },
        map::{self, Distance, HexMap, PosHex},
    },
    geom::{self, hex_to_point},
    screen::battle::visualize,
    sprite_info::SpriteInfo,
    utils::{self, font_size, time_s},
    ZResult,
};

#[derive(Debug, Clone, PartialEq)]
pub enum SelectionMode {
    Normal,
    Ability(Ability),
}

const TILE_COLOR_WALKABLE: [f32; 4] = [0.1, 0.6, 0.1, 0.5];
const TILE_COLOR_ATTACKABLE: [f32; 4] = [0.8, 0.0, 0.0, 0.6];
const TILE_COLOR_ABILITY: [f32; 4] = [0.0, 0.0, 0.9, 0.5];

#[derive(Debug, Clone, Default)]
pub struct Layers {
    pub bg: Layer,
    pub blood: Layer,
    pub shadows: Layer,
    pub grass: Layer,
    pub highlighted_tiles: Layer,
    pub selection_marker: Layer,
    pub current_tile_marker: Layer,
    pub particles: Layer,
    pub objects: Layer,
    pub dots: Layer,
    pub flares: Layer,
    pub text: Layer,
}

impl Layers {
    fn sorted(self) -> Vec<Layer> {
        vec![
            self.bg,
            self.blood,
            self.shadows,
            self.grass,
            self.highlighted_tiles,
            self.selection_marker,
            self.current_tile_marker,
            self.particles,
            self.objects,
            self.dots,
            self.flares,
            self.text,
        ]
    }
}

pub fn tile_size(map_height: Distance) -> f32 {
    1.0 / (map_height.0 as f32 * 0.75)
}

#[derive(Debug)]
struct DisappearingSprite {
    sprite: Sprite,
    layer: Layer,
    // TODO: use a special type instead of i32!
    turns_total: i32,
    turns_left: i32,
    initial_alpha: f32,
}

#[derive(Debug)]
struct Sprites {
    selection_marker: Sprite,
    current_tile_marker: Sprite,
    highlighted_tiles: Vec<Sprite>,
    labels: Vec<Sprite>,
    id_to_sprite_map: HashMap<Id, Sprite>,
    id_to_shadow_map: HashMap<Id, Sprite>,
    agent_info: HashMap<Id, Vec<Sprite>>,
    disappearing_sprites: Vec<DisappearingSprite>,
}

#[derive(Debug)]
pub struct Images {
    pub selection: Image,
    pub white_hex: Image,
    pub tile: Image,
    pub tile_rocks: Image,
    pub grass: Image,
    pub dot: Image,
    pub blood: Image,
    pub explosion_ground_mark: Image,
    pub shadow: Image,
    pub attack_slash: Image,
    pub attack_smash: Image,
    pub attack_pierce: Image,
    pub attack_claws: Image,
    pub effect_stun: Image,
    pub effect_poison: Image,
    pub effect_bloodlust: Image,
}

impl Images {
    fn new(context: &mut Context) -> ZResult<Self> {
        Ok(Self {
            selection: Image::new(context, "/selection.png")?,
            white_hex: Image::new(context, "/white_hex.png")?,
            tile: Image::new(context, "/tile.png")?,
            tile_rocks: Image::new(context, "/tile_rocks.png")?,
            grass: Image::new(context, "/grass.png")?,
            dot: Image::new(context, "/dot.png")?,
            blood: Image::new(context, "/blood.png")?,
            explosion_ground_mark: Image::new(context, "/explosion_ground_mark.png")?,
            shadow: Image::new(context, "/shadow.png")?,
            attack_slash: Image::new(context, "/slash.png")?,
            attack_smash: Image::new(context, "/smash.png")?,
            attack_pierce: Image::new(context, "/pierce.png")?,
            attack_claws: Image::new(context, "/claw.png")?,
            effect_stun: Image::new(context, "/effect_stun.png")?,
            effect_poison: Image::new(context, "/effect_poison.png")?,
            effect_bloodlust: Image::new(context, "/effect_bloodlust.png")?,
        })
    }
}

#[derive(Debug)]
pub struct BattleView {
    font: Font,
    tile_size: f32,
    layers: Layers,
    scene: Scene,
    sprites: Sprites,
    images: Images,
    sprite_info: HashMap<ObjType, SpriteInfo>,
}

impl BattleView {
    pub fn new(map_radius: Distance, context: &mut Context) -> ZResult<Self> {
        let font = utils::default_font(context);
        let images = Images::new(context)?;
        let layers = Layers::default();
        let scene = Scene::new(layers.clone().sorted());
        let map_diameter = map::radius_to_diameter(map_radius);
        let tile_size = tile_size(map_diameter);
        let mut make_marker_sprite = |color: Color| -> ZResult<Sprite> {
            let h = tile_size * 2.0 * geom::FLATNESS_COEFFICIENT;
            let mut sprite = Sprite::from_image(context, images.selection.clone(), h)?;
            sprite.set_centered(true);
            sprite.set_color(color);
            Ok(sprite)
        };
        let selection_marker = make_marker_sprite([0.0, 0.0, 1.0, 0.8].into())?;
        let current_tile_marker = make_marker_sprite([0.0, 0.0, 0.0, 0.5].into())?;
        let sprite_info = utils::deserialize_from_file(context, "/sprites.ron")?;
        let sprites = Sprites {
            selection_marker,
            current_tile_marker,
            highlighted_tiles: Vec::new(),
            labels: Vec::new(),
            id_to_sprite_map: HashMap::new(),
            id_to_shadow_map: HashMap::new(),
            agent_info: HashMap::new(),
            disappearing_sprites: Vec::new(),
        };
        Ok(Self {
            font,
            sprites,
            scene,
            layers,
            tile_size,
            images,
            sprite_info,
        })
    }

    pub fn font(&self) -> Font {
        self.font
    }

    pub fn images(&self) -> &Images {
        &self.images
    }

    pub fn message(&mut self, context: &mut Context, pos: PosHex, text: &str) -> ZResult {
        let action = visualize::message(self, context, pos, text)?;
        self.add_action(action);
        Ok(())
    }

    pub fn tick(&mut self, dtime: Duration) {
        self.scene.tick(dtime);
    }

    pub fn draw(&self, context: &mut Context) -> ZResult {
        self.scene.draw(context)?;
        Ok(())
    }

    pub fn add_action(&mut self, action: Box<dyn Action>) {
        self.scene.add_action(action);
    }

    // TODO: return `(f32, f32)`? width and height separately?
    pub fn tile_size(&self) -> f32 {
        self.tile_size
    }

    pub fn hex_to_point(&self, hex: PosHex) -> Point2<f32> {
        geom::hex_to_point(self.tile_size, hex)
    }

    pub fn layers(&self) -> &Layers {
        &self.layers
    }

    pub fn add_object(&mut self, id: Id, sprite: &Sprite, sprite_shadow: &Sprite) {
        let sprite_shadow = sprite_shadow.clone();
        let sprite = sprite.clone();
        self.sprites.id_to_sprite_map.insert(id, sprite);
        self.sprites.id_to_shadow_map.insert(id, sprite_shadow);
    }

    pub fn remove_object(&mut self, id: Id) {
        self.sprites.id_to_sprite_map.remove(&id).unwrap();
        self.sprites.id_to_shadow_map.remove(&id).unwrap();
    }

    pub fn add_disappearing_sprite(
        &mut self,
        layer: &Layer,
        sprite: &Sprite,
        turns: i32,
        initial_alpha: f32,
    ) {
        self.sprites.disappearing_sprites.push(DisappearingSprite {
            sprite: sprite.clone(),
            layer: layer.clone(),
            turns_total: turns,
            turns_left: turns,
            initial_alpha,
        });
    }

    pub fn update_disappearing_sprites(&mut self) -> Box<dyn Action> {
        let mut actions = Vec::new();
        for s in &mut self.sprites.disappearing_sprites {
            s.turns_left -= 1;
            let mut color = s.sprite.color();
            color.a = (s.initial_alpha / s.turns_total as f32) * s.turns_left as f32;
            let mut sub_actions = Vec::new();
            sub_actions.push(action::ChangeColorTo::new(&s.sprite, color, time_s(2.0)).boxed());
            if s.turns_left == 0 {
                sub_actions.push(action::Hide::new(&s.layer, &s.sprite).boxed());
            }
            actions.push(visualize::fork(visualize::seq(sub_actions)));
        }
        self.sprites
            .disappearing_sprites
            .retain(|s| s.turns_left > 0);
        visualize::seq(actions)
    }

    pub fn id_to_sprite(&mut self, id: Id) -> &Sprite {
        &self.sprites.id_to_sprite_map[&id]
    }

    pub fn id_to_shadow_sprite(&mut self, id: Id) -> &Sprite {
        &self.sprites.id_to_shadow_map[&id]
    }

    pub fn agent_info_check(&self, id: Id) -> bool {
        self.sprites.agent_info.get(&id).is_some()
    }

    pub fn agent_info_get(&mut self, id: Id) -> Vec<Sprite> {
        self.sprites.agent_info.remove(&id).unwrap()
    }

    pub fn agent_info_set(&mut self, id: Id, sprites: Vec<Sprite>) {
        self.sprites.agent_info.insert(id, sprites);
    }

    pub fn sprite_info(&self, obj_type: &ObjType) -> SpriteInfo {
        self.sprite_info[&obj_type].clone()
    }

    pub fn set_mode(
        &mut self,
        state: &State,
        context: &mut Context,
        map: &HexMap<movement::Tile>,
        selected_id: Id,
        mode: &SelectionMode,
    ) -> ZResult {
        match mode {
            SelectionMode::Normal => self.select_normal(state, context, map, selected_id),
            SelectionMode::Ability(ref ability) => {
                self.select_ability(state, context, selected_id, ability)
            }
        }
    }

    fn remove_highlights(&mut self) {
        self.clean_highlighted_tiles();
        self.clean_labels();
    }

    fn clean_highlighted_tiles(&mut self) {
        for sprite in self.sprites.highlighted_tiles.split_off(0) {
            let color = Color {
                a: 0.0,
                ..sprite.color()
            };
            let action = {
                let layer = &self.layers().highlighted_tiles;
                let time = time_s(0.3);
                visualize::seq(vec![
                    action::ChangeColorTo::new(&sprite, color, time).boxed(),
                    action::Hide::new(layer, &sprite).boxed(),
                ])
            };
            self.add_action(action);
        }
    }

    fn clean_labels(&mut self) {
        for sprite in self.sprites.labels.split_off(0) {
            let action = action::Hide::new(&self.layers().text, &sprite).boxed();
            self.add_action(action);
        }
    }

    pub fn hide_current_tile_marker(&mut self) {
        let layer = &self.layers.current_tile_marker;
        let sprite = &mut self.sprites.current_tile_marker;
        if layer.has_sprite(sprite) {
            let hide_marker = action::Hide::new(layer, sprite).boxed();
            self.scene.add_action(hide_marker);
        }
    }

    pub fn show_current_tile_marker(&mut self, pos: PosHex) {
        let point = hex_to_point(self.tile_size(), pos);
        let layer = &self.layers.current_tile_marker;
        let sprite = &mut self.sprites.current_tile_marker;
        sprite.set_pos(point);
        if !layer.has_sprite(sprite) {
            let action = action::Show::new(layer, sprite).boxed();
            self.scene.add_action(action);
        }
    }

    pub fn deselect(&mut self) {
        self.hide_selection_marker();
        self.remove_highlights();
    }

    fn select_normal(
        &mut self,
        state: &State,
        context: &mut Context,
        map: &HexMap<movement::Tile>,
        id: Id,
    ) -> ZResult {
        self.show_selection_marker(state, id);
        self.show_walkable_tiles(state, context, map, id)?;
        self.show_attackable_tiles(state, context, id)
    }

    fn select_ability(
        &mut self,
        state: &State,
        context: &mut Context,
        selected_id: Id,
        ability: &Ability,
    ) -> ZResult {
        self.remove_highlights();
        let positions = state.map().iter();
        for pos in positions {
            let id = selected_id;
            let ability = ability.clone();
            let command = command::UseAbility { id, ability, pos }.into();
            if battle::check(state, &command).is_ok() {
                self.highlight_tile(context, pos, TILE_COLOR_ABILITY.into())?;
            }
        }
        Ok(())
    }

    fn show_selection_marker(&mut self, state: &State, id: Id) {
        let pos = state.parts().pos.get(id).0;
        let point = hex_to_point(self.tile_size(), pos);
        let layer = &self.layers.selection_marker;
        let sprite = &mut self.sprites.selection_marker;
        sprite.set_pos(point);
        let action = action::Show::new(layer, sprite).boxed();
        self.scene.add_action(action);
    }

    fn hide_selection_marker(&mut self) {
        let layer = &self.layers.selection_marker;
        let sprite = &self.sprites.selection_marker;
        if layer.has_sprite(sprite) {
            let hide_marker = action::Hide::new(layer, sprite).boxed();
            self.scene.add_action(hide_marker);
        }
    }

    fn show_attackable_tiles(&mut self, state: &State, context: &mut Context, id: Id) -> ZResult {
        let parts = state.parts();
        let selected_agent_player_id = parts.belongs_to.get(id).0;
        for target_id in parts.agent.ids() {
            let target_pos = parts.pos.get(target_id).0;
            let target_player_id = parts.belongs_to.get(target_id).0;
            if target_player_id == selected_agent_player_id {
                continue;
            }
            let command_attack = command::Attack {
                attacker_id: id,
                target_id,
            }
            .into();
            if battle::check(state, &command_attack).is_err() {
                continue;
            }
            self.show_hit_chance_label(state, context, id, target_id)?;
            self.highlight_tile(context, target_pos, TILE_COLOR_ATTACKABLE.into())?;
        }
        Ok(())
    }

    fn show_walkable_tiles(
        &mut self,
        state: &State,
        context: &mut Context,
        map: &HexMap<movement::Tile>,
        id: Id,
    ) -> ZResult {
        let agent = state.parts().agent.get(id);
        if agent.moves == Moves(0) && agent.jokers == Jokers(0) {
            return Ok(());
        }
        for pos in map.iter() {
            if map.tile(pos).cost() > agent.move_points {
                continue;
            }
            self.highlight_tile(context, pos, TILE_COLOR_WALKABLE.into())?
        }
        Ok(())
    }

    fn highlight_tile(&mut self, context: &mut Context, pos: PosHex, color: Color) -> ZResult {
        let size = self.tile_size() * 2.0 * geom::FLATNESS_COEFFICIENT;
        let mut sprite = Sprite::from_image(context, self.images.white_hex.clone(), size)?;
        let color_from = Color { a: 0.0, ..color };
        sprite.set_centered(true);
        sprite.set_color(color_from);
        sprite.set_pos(hex_to_point(self.tile_size(), pos));
        let time = time_s(0.3);
        let layer = &self.layers.highlighted_tiles;
        let actions = vec![
            action::Show::new(layer, &sprite).boxed(),
            action::ChangeColorTo::new(&sprite, color, time).boxed(),
        ];
        self.scene.add_action(visualize::seq(actions));
        self.sprites.highlighted_tiles.push(sprite);
        Ok(())
    }

    fn show_hit_chance_label(
        &mut self,
        state: &State,
        context: &mut Context,
        attacker_id: Id,
        target_id: Id,
    ) -> ZResult {
        let target_pos = state.parts().pos.get(target_id).0;
        let chances = hit_chance(state, attacker_id, target_id);
        let pos = hex_to_point(self.tile_size(), target_pos);
        let text = format!("{}%", chances.1 * 10);
        let text = Box::new(Text::new((text.as_str(), self.font, font_size())));
        let mut sprite = Sprite::from_drawable(context, text, 0.1)?;
        sprite.set_pos(pos);
        sprite.set_centered(true);
        sprite.set_color([0.0, 0.0, 0.0, 1.0].into());
        let action = action::Show::new(&self.layers.text, &sprite).boxed();
        self.scene.add_action(action);
        self.sprites.labels.push(sprite);
        Ok(())
    }
}

fn make_action_show_tile(
    state: &State,
    context: &mut Context,
    view: &BattleView,
    at: PosHex,
) -> ZResult<Box<dyn Action>> {
    let screen_pos = hex_to_point(view.tile_size(), at);
    let image = match state.map().tile(at) {
        TileType::Plain => view.images.tile.clone(),
        TileType::Rocks => view.images.tile_rocks.clone(),
    };
    let size = view.tile_size() * 2.0 * geom::FLATNESS_COEFFICIENT;
    let mut sprite = Sprite::from_image(context, image, size)?;
    sprite.set_centered(true);
    sprite.set_pos(screen_pos);
    Ok(action::Show::new(&view.layers().bg, &sprite).boxed())
}

fn make_action_grass(
    context: &mut Context,
    view: &BattleView,
    at: PosHex,
) -> ZResult<Box<dyn Action>> {
    let screen_pos = hex_to_point(view.tile_size(), at);
    let mut sprite =
        Sprite::from_image(context, view.images.grass.clone(), view.tile_size() * 2.0)?;
    let v_offset = view.tile_size() * 0.5; // depends on the image
    let mut screen_pos_grass = screen_pos + geom::rand_tile_offset(view.tile_size(), 0.5);
    screen_pos_grass.y -= v_offset;
    sprite.set_centered(true);
    sprite.set_pos(screen_pos_grass);
    Ok(action::Show::new(&view.layers().grass, &sprite).boxed())
}

pub fn make_action_create_map(
    state: &State,
    context: &mut Context,
    view: &BattleView,
) -> ZResult<Box<dyn Action>> {
    let mut actions = Vec::new();
    for hex_pos in state.map().iter() {
        actions.push(make_action_show_tile(state, context, view, hex_pos)?);
        if thread_rng().gen_range(0, 10) < 2 {
            actions.push(make_action_grass(context, view, hex_pos)?);
        }
    }
    Ok(visualize::seq(actions))
}
