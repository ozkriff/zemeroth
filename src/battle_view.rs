use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::time::Duration;

use ggez::graphics::{Color, Font, Point2};
use ggez::Context;
use scene::action;
use scene::{Action, Boxed, Layer, Scene, Sprite};

use core::ability::Ability;
use core::map::{HexMap, PosHex};
use core::{self, command, movement};
use core::{Jokers, Moves, ObjId, State, TileType};
use geom::hex_to_point;
use visualize;
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
    pub grass: Layer,
    pub highlighted_tiles: Layer,
    pub selection_marker: Layer,
    pub units: Layer,
    pub dots: Layer,
    pub flares: Layer,
    pub text: Layer,
}

impl Layers {
    fn sorted(self) -> Vec<Layer> {
        vec![
            self.bg,
            self.blood,
            self.grass,
            self.highlighted_tiles,
            self.selection_marker,
            self.units,
            self.dots,
            self.flares,
            self.text,
        ]
    }
}

pub fn tile_size(map_height: i32) -> f32 {
    1.0 / ((map_height + 1) as f32 * 0.75)
}

#[derive(Debug)]
struct Sprites {
    selection_marker: Sprite,
    highlighted_tiles: Vec<Sprite>,
    id_to_sprite_map: HashMap<ObjId, Sprite>,
    unit_info: HashMap<ObjId, Vec<Sprite>>,
}

#[derive(Debug)]
pub struct BattleView {
    // TODO: https://docs.rs/ggez/0.4.2/ggez/struct.Context.html#structfield.default_font?
    font: Font,

    tile_size: f32,
    layers: Layers,
    scene: Scene,
    sprites: Sprites,
}

impl BattleView {
    pub fn new(state: &State, context: &mut Context) -> ZResult<Self> {
        let font = Font::new(context, "/OpenSans-Regular.ttf", 24)?;
        let layers = Layers::default();
        let scene = Scene::new(layers.clone().sorted());
        let tile_size = tile_size(state.map().height());
        let mut selection_marker = Sprite::from_path(context, "/selection.png", tile_size * 2.0)?;
        selection_marker.set_centered(true);
        selection_marker.set_color([0.0, 0.0, 1.0, 0.8].into());
        let sprites = Sprites {
            selection_marker,
            highlighted_tiles: Vec::new(),
            id_to_sprite_map: HashMap::new(),
            unit_info: HashMap::new(),
        };
        Ok(Self {
            font,
            sprites,
            scene,
            layers,
            tile_size,
        })
    }

    pub fn font(&self) -> &Font {
        &self.font
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
    pub fn add_action(&mut self, action: Box<Action>) {
        self.scene.add_action(action);
    }

    pub fn tile_size(&self) -> f32 {
        self.tile_size
    }

    pub fn layers(&self) -> &Layers {
        &self.layers
    }

    pub fn add_object(&mut self, id: ObjId, sprite: &Sprite) {
        self.sprites.id_to_sprite_map.insert(id, sprite.clone());
    }

    pub fn remove_object(&mut self, id: ObjId) {
        self.sprites.id_to_sprite_map.remove(&id).unwrap();
    }

    pub fn id_to_sprite(&mut self, id: ObjId) -> &Sprite {
        &self.sprites.id_to_sprite_map[&id]
    }

    pub fn unit_info_check(&self, id: ObjId) -> bool {
        self.sprites.unit_info.get(&id).is_some()
    }

    pub fn unit_info_get(&mut self, id: ObjId) -> Vec<Sprite> {
        self.sprites.unit_info.remove(&id).unwrap()
    }

    pub fn unit_info_set(&mut self, id: ObjId, sprites: Vec<Sprite>) {
        self.sprites.unit_info.insert(id, sprites);
    }

    pub fn set_mode(
        &mut self,
        state: &State,
        map: &HexMap<movement::Tile>,
        context: &mut Context,
        selected_id: ObjId,
        mode: &SelectionMode,
    ) -> ZResult {
        match *mode {
            SelectionMode::Normal => self.select_normal(state, map, context, selected_id),
            SelectionMode::Ability(ability) => {
                self.select_ability(state, context, selected_id, ability)
            }
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

    fn select_normal(
        &mut self,
        state: &State,
        map: &HexMap<movement::Tile>,
        context: &mut Context,
        id: ObjId,
    ) -> ZResult {
        self.show_selection_marker(state, id);
        self.show_walkable_tiles(state, map, context, id)?;
        self.show_attackable_tiles(state, context, id)
    }

    fn select_ability(
        &mut self,
        state: &State,
        context: &mut Context,
        selected_id: ObjId,
        ability: Ability,
    ) -> ZResult {
        self.remove_highlights();
        let positions = state.map().iter();
        for pos in positions {
            let command = command::Command::UseAbility(command::UseAbility {
                id: selected_id,
                ability,
                pos,
            });
            if core::check(state, &command).is_ok() {
                self.highlight(context, pos, TILE_COLOR_ABILITY.into())?;
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

    fn show_attackable_tiles(
        &mut self,
        state: &State,
        context: &mut Context,
        id: ObjId,
    ) -> ZResult {
        let parts = state.parts();
        let selected_unit_player_id = parts.belongs_to.get(id).0;
        for target_id in parts.agent.ids() {
            let target_pos = parts.pos.get(target_id).0;
            let target_player_id = parts.belongs_to.get(target_id).0;
            if target_player_id == selected_unit_player_id {
                continue;
            }
            let command_attack = command::Command::Attack(command::Attack {
                attacker_id: id,
                target_id,
            });
            if core::check(state, &command_attack).is_err() {
                continue;
            }
            self.highlight(context, target_pos, TILE_COLOR_ATTACKABLE.into())?;
        }
        Ok(())
    }

    fn show_walkable_tiles(
        &mut self,
        state: &State,
        map: &HexMap<movement::Tile>,
        context: &mut Context,
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
            self.highlight(context, pos, TILE_COLOR_WALKABLE.into())?
        }
        Ok(())
    }

    fn highlight(&mut self, context: &mut Context, pos: PosHex, color: Color) -> ZResult {
        let size = self.tile_size() * 2.0;
        let mut sprite = Sprite::from_path(context, "/white_hex.png", size)?;
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

fn make_action_show_tile(
    context: &mut Context,
    state: &State,
    view: &BattleView,
    at: PosHex,
) -> ZResult<Box<Action>> {
    let screen_pos = hex_to_point(view.tile_size(), at);
    let texture_name = match state.map().tile(at) {
        TileType::Plain => "/tile.png",
        TileType::Rocks => "/tile_rocks.png",
    };
    let size = view.tile_size() * 2.0;
    let mut sprite = Sprite::from_path(context, texture_name, size)?;
    sprite.set_centered(true);
    sprite.set_pos(screen_pos);
    Ok(action::Show::new(&view.layers().bg, &sprite).boxed())
}

fn make_action_grass(context: &mut Context, view: &BattleView, at: PosHex) -> ZResult<Box<Action>> {
    let screen_pos = hex_to_point(view.tile_size(), at);
    let mut sprite = Sprite::from_path(context, "/grass.png", view.tile_size() * 2.0)?;
    let n = view.tile_size() * 0.5;
    let screen_pos_grass = Point2::new(
        screen_pos.x + thread_rng().gen_range(-n, n),
        screen_pos.y + thread_rng().gen_range(-n, n),
    );
    sprite.set_centered(true);
    sprite.set_pos(screen_pos_grass);
    Ok(action::Show::new(&view.layers().grass, &sprite).boxed())
}

pub fn make_action_create_map(
    state: &State,
    view: &BattleView,
    context: &mut Context,
) -> ZResult<Box<Action>> {
    let mut actions = Vec::new();
    for hex_pos in state.map().iter() {
        actions.push(make_action_show_tile(context, state, view, hex_pos)?);
        if thread_rng().gen_range(0, 10) < 2 {
            actions.push(make_action_grass(context, view, hex_pos)?);
        }
    }
    Ok(action::Sequence::new(actions).boxed())
}
