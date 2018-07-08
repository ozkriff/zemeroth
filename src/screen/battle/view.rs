use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::time::Duration;

use ggez::graphics::{Color, Font, Image, Point2};
use ggez::Context;
use scene::action;
use scene::{Action, Boxed, Layer, Scene, Sprite};

use core::ability::Ability;
use core::map::{Distance, HexMap, PosHex};
use core::{self, command, movement};
use core::{Jokers, Moves, ObjId, State, TileType};
use geom::{self, hex_to_point};
use screen::battle::visualize;
use ZResult;

#[derive(Debug, PartialEq)]
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
    pub agents: Layer,
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
            self.agents,
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
struct Sprites {
    selection_marker: Sprite,
    highlighted_tiles: Vec<Sprite>,
    id_to_sprite_map: HashMap<ObjId, Sprite>,
    id_to_shadow_map: HashMap<ObjId, Sprite>,
    agent_info: HashMap<ObjId, Vec<Sprite>>,
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
    pub shadow: Image,
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
            shadow: Image::new(context, "/shadow.png")?,
        })
    }
}

#[derive(Debug)]
pub struct BattleView {
    // TODO: https://docs.rs/ggez/0.4.2/ggez/struct.Context.html#structfield.default_font?
    font: Font,

    tile_size: f32,
    layers: Layers,
    scene: Scene,
    sprites: Sprites,
    images: Images,
}

impl BattleView {
    pub fn new(state: &State, context: &mut Context) -> ZResult<Self> {
        let font = Font::new(context, "/OpenSans-Regular.ttf", 24)?;
        let images = Images::new(context)?;
        let layers = Layers::default();
        let scene = Scene::new(layers.clone().sorted());
        let tile_size = tile_size(state.map().height());
        let mut selection_marker = Sprite::from_image(
            images.selection.clone(),
            tile_size * 2.0 * geom::FLATNESS_COEFFICIENT,
        );
        selection_marker.set_centered(true);
        selection_marker.set_color([0.0, 0.0, 1.0, 0.8].into());
        let sprites = Sprites {
            selection_marker,
            highlighted_tiles: Vec::new(),
            id_to_sprite_map: HashMap::new(),
            id_to_shadow_map: HashMap::new(),
            agent_info: HashMap::new(),
        };
        Ok(Self {
            font,
            sprites,
            scene,
            layers,
            tile_size,
            images,
        })
    }

    pub fn font(&self) -> &Font {
        &self.font
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
        self.scene.draw(context)
    }
    pub fn add_action(&mut self, action: Box<dyn Action>) {
        self.scene.add_action(action);
    }

    // TODO: return `(f32, f32)`? width and height separately?
    pub fn tile_size(&self) -> f32 {
        self.tile_size
    }

    pub fn layers(&self) -> &Layers {
        &self.layers
    }

    pub fn add_object(&mut self, id: ObjId, sprite: &Sprite, sprite_shadow: &Sprite) {
        let sprite_shadow = sprite_shadow.clone();
        let sprite = sprite.clone();
        self.sprites.id_to_sprite_map.insert(id, sprite);
        self.sprites.id_to_shadow_map.insert(id, sprite_shadow);
    }

    pub fn remove_object(&mut self, id: ObjId) {
        self.sprites.id_to_sprite_map.remove(&id).unwrap();
        self.sprites.id_to_shadow_map.remove(&id).unwrap();
    }

    pub fn id_to_sprite(&mut self, id: ObjId) -> &Sprite {
        &self.sprites.id_to_sprite_map[&id]
    }

    pub fn id_to_shadow_sprite(&mut self, id: ObjId) -> &Sprite {
        &self.sprites.id_to_shadow_map[&id]
    }

    pub fn agent_info_check(&self, id: ObjId) -> bool {
        self.sprites.agent_info.get(&id).is_some()
    }

    pub fn agent_info_get(&mut self, id: ObjId) -> Vec<Sprite> {
        self.sprites.agent_info.remove(&id).unwrap()
    }

    pub fn agent_info_set(&mut self, id: ObjId, sprites: Vec<Sprite>) {
        self.sprites.agent_info.insert(id, sprites);
    }

    pub fn set_mode(
        &mut self,
        state: &State,
        map: &HexMap<movement::Tile>,
        selected_id: ObjId,
        mode: &SelectionMode,
    ) -> ZResult {
        match *mode {
            SelectionMode::Normal => self.select_normal(state, map, selected_id),
            SelectionMode::Ability(ability) => self.select_ability(state, selected_id, ability),
        }
    }

    fn remove_highlights(&mut self) {
        for sprite in self.sprites.highlighted_tiles.split_off(0) {
            let color = Color {
                a: 0.0,
                ..sprite.color()
            };
            let action = {
                let layer = &self.layers().highlighted_tiles;
                let time = Duration::from_millis(300); // TODO: time_s
                let actions = vec![
                    action::ChangeColorTo::new(&sprite, color, time).boxed(),
                    action::Hide::new(layer, &sprite).boxed(),
                ];
                action::Sequence::new(actions).boxed()
            };
            self.add_action(action);
        }
    }

    pub fn deselect(&mut self) {
        self.hide_selection_marker();
        self.remove_highlights();
    }

    fn select_normal(&mut self, state: &State, map: &HexMap<movement::Tile>, id: ObjId) -> ZResult {
        self.show_selection_marker(state, id);
        self.show_walkable_tiles(state, map, id)?;
        self.show_attackable_tiles(state, id)
    }

    fn select_ability(&mut self, state: &State, selected_id: ObjId, ability: Ability) -> ZResult {
        self.remove_highlights();
        let positions = state.map().iter();
        for pos in positions {
            let command = command::Command::UseAbility(command::UseAbility {
                id: selected_id,
                ability,
                pos,
            });
            if core::check(state, &command).is_ok() {
                self.highlight(pos, TILE_COLOR_ABILITY.into())?;
            }
        }
        Ok(())
    }

    fn show_selection_marker(&mut self, state: &State, id: ObjId) {
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

    fn show_attackable_tiles(&mut self, state: &State, id: ObjId) -> ZResult {
        let parts = state.parts();
        let selected_agent_player_id = parts.belongs_to.get(id).0;
        for target_id in parts.agent.ids() {
            let target_pos = parts.pos.get(target_id).0;
            let target_player_id = parts.belongs_to.get(target_id).0;
            if target_player_id == selected_agent_player_id {
                continue;
            }
            let command_attack = command::Command::Attack(command::Attack {
                attacker_id: id,
                target_id,
            });
            if core::check(state, &command_attack).is_err() {
                continue;
            }
            self.highlight(target_pos, TILE_COLOR_ATTACKABLE.into())?;
        }
        Ok(())
    }

    fn show_walkable_tiles(
        &mut self,
        state: &State,
        map: &HexMap<movement::Tile>,
        id: ObjId,
    ) -> ZResult {
        let agent = state.parts().agent.get(id);
        if agent.moves == Moves(0) && agent.jokers == Jokers(0) {
            return Ok(());
        }
        for pos in map.iter() {
            if map.tile(pos).cost() > agent.move_points {
                continue;
            }
            self.highlight(pos, TILE_COLOR_WALKABLE.into())?
        }
        Ok(())
    }

    fn highlight(&mut self, pos: PosHex, color: Color) -> ZResult {
        let size = self.tile_size() * 2.0 * geom::FLATNESS_COEFFICIENT;
        let mut sprite = Sprite::from_image(self.images.white_hex.clone(), size);
        let color_from = Color { a: 0.0, ..color };
        sprite.set_centered(true);
        sprite.set_color(color_from);
        sprite.set_pos(hex_to_point(self.tile_size(), pos));
        let time = Duration::from_millis(300);
        let layer = &self.layers.highlighted_tiles;
        let actions = vec![
            action::Show::new(layer, &sprite).boxed(),
            action::ChangeColorTo::new(&sprite, color, time).boxed(),
        ];
        let action = action::Sequence::new(actions).boxed();
        self.scene.add_action(action);
        self.sprites.highlighted_tiles.push(sprite);
        Ok(())
    }
}

fn make_action_show_tile(state: &State, view: &BattleView, at: PosHex) -> ZResult<Box<dyn Action>> {
    let screen_pos = hex_to_point(view.tile_size(), at);
    let image = match state.map().tile(at) {
        TileType::Plain => view.images.tile.clone(),
        TileType::Rocks => view.images.tile_rocks.clone(),
    };
    let size = view.tile_size() * 2.0 * geom::FLATNESS_COEFFICIENT;
    let mut sprite = Sprite::from_image(image, size);
    sprite.set_centered(true);
    sprite.set_pos(screen_pos);
    Ok(action::Show::new(&view.layers().bg, &sprite).boxed())
}

fn make_action_grass(view: &BattleView, at: PosHex) -> ZResult<Box<dyn Action>> {
    let screen_pos = hex_to_point(view.tile_size(), at);
    let mut sprite = Sprite::from_image(view.images.grass.clone(), view.tile_size() * 2.0);
    let n = view.tile_size() * 0.5;
    let screen_pos_grass = Point2::new(
        screen_pos.x + thread_rng().gen_range(-n, n),
        screen_pos.y + thread_rng().gen_range(-n, n),
    );
    sprite.set_centered(true);
    sprite.set_pos(screen_pos_grass);
    Ok(action::Show::new(&view.layers().grass, &sprite).boxed())
}

pub fn make_action_create_map(state: &State, view: &BattleView) -> ZResult<Box<dyn Action>> {
    let mut actions = Vec::new();
    for hex_pos in state.map().iter() {
        actions.push(make_action_show_tile(state, view, hex_pos)?);
        if thread_rng().gen_range(0, 10) < 2 {
            actions.push(make_action_grass(view, hex_pos)?);
        }
    }
    Ok(action::Sequence::new(actions).boxed())
}
