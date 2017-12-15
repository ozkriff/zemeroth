use std::collections::HashMap;
use hate::{Context, Scene, Sprite, Time};
use hate::scene::Layer;
use hate::scene::action::{self, Action};
use core::{check, Jokers, Moves, State};
use core::ObjId;
use core::map::HexMap;
use core::movement::Tile;
use core::command;
use map::hex_to_point;

const WALKBALE_TILE_COLOR: [f32; 4] = [0.2, 1.0, 0.2, 0.5];

#[derive(Debug, Clone, Default)]
pub struct Layers {
    pub bg: Layer,
    pub blood: Layer,
    pub grass: Layer,
    pub walkable_tiles: Layer,
    pub attackable_tiles: Layer,
    pub selection_marker: Layer,
    pub units: Layer,
    pub text: Layer,
}

impl Layers {
    fn sorted(self) -> Vec<Layer> {
        vec![
            self.bg,
            self.blood,
            self.grass,
            self.walkable_tiles,
            self.attackable_tiles,
            self.selection_marker,
            self.units,
            self.text,
        ]
    }
}

#[derive(Debug)]
struct Sprites {
    selection_marker: Sprite,
    walkable_tiles: Vec<Sprite>,
    attackable_tiles: Vec<Sprite>,
    id_to_sprite_map: HashMap<ObjId, Sprite>,
    unit_info: HashMap<ObjId, Vec<Sprite>>,
}

#[derive(Debug)]
pub struct GameView {
    tile_size: f32,
    layers: Layers,
    scene: Scene,
    sprites: Sprites,
}

impl GameView {
    pub fn new(state: &State, context: &mut Context) -> Self {
        let layers = Layers::default();
        let scene = Scene::new(layers.clone().sorted());
        let map_height = state.map().radius().0 * 2 + 1;
        let tile_size = 1.0 / ((map_height + 1) as f32 * 0.75);
        let mut selection_marker = Sprite::from_path(context, "selection.png", tile_size * 2.0);
        selection_marker.set_color([0.0, 0.0, 1.0, 0.8]);
        let sprites = Sprites {
            selection_marker,
            walkable_tiles: Vec::new(),
            attackable_tiles: Vec::new(),
            id_to_sprite_map: HashMap::new(),
            unit_info: HashMap::new(),
        };
        Self {
            scene,
            tile_size,
            layers,
            sprites,
        }
    }

    pub fn tick(&mut self, context: &mut Context, dtime: Time) {
        self.scene.tick(dtime);
        self.scene.draw(context);
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

    pub fn deselect(&mut self) {
        let action_hide = Box::new(action::Hide::new(
            &self.layers.selection_marker,
            &self.sprites.selection_marker,
        ));
        self.add_action(action_hide);
        for sprite in self.sprites.walkable_tiles.split_off(0) {
            let mut color = WALKBALE_TILE_COLOR;
            color[3] = 0.0;
            let action = {
                let layer = &self.layers().walkable_tiles;
                Box::new(action::Sequence::new(vec![
                    Box::new(action::ChangeColorTo::new(&sprite, color, Time(0.2))),
                    Box::new(action::Hide::new(layer, &sprite)),
                ]))
            };
            self.add_action(action);
        }
        for sprite in self.sprites.attackable_tiles.split_off(0) {
            let action = {
                let layer = &self.layers().attackable_tiles;
                Box::new(action::Hide::new(layer, &sprite))
            };
            self.add_action(action);
        }
    }

    pub fn select_unit(
        &mut self,
        state: &State,
        map: &HexMap<Tile>,
        context: &mut Context,
        id: ObjId,
    ) {
        self.show_selection_marker(state, id);
        self.show_walkable_tiles(state, map, context, id);
        self.show_attackable_tiles(state, context, id);
    }

    fn show_selection_marker(&mut self, state: &State, id: ObjId) {
        let pos = state.parts().pos.get(id).0;
        let point = hex_to_point(self.tile_size(), pos);
        self.sprites.selection_marker.set_pos(point);
        let action = Box::new(action::Show::new(
            &self.layers().selection_marker,
            &self.sprites.selection_marker,
        ));
        self.add_action(action);
    }

    fn show_attackable_tiles(&mut self, state: &State, context: &mut Context, id: ObjId) {
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
                target_id: target_id,
            });
            if check(state, &command_attack).is_err() {
                continue;
            }
            let size = self.tile_size() * 2.0;
            let mut sprite = Sprite::from_path(context, "tile.png", size);
            self.sprites.attackable_tiles.push(sprite.clone());
            sprite.set_color([1.0, 0.3, 0.3, 0.8]);
            sprite.set_pos(hex_to_point(self.tile_size(), target_pos));
            let action = Box::new(action::Show::new(&self.layers().attackable_tiles, &sprite));
            self.add_action(action);
        }
    }

    fn show_walkable_tiles(
        &mut self,
        state: &State,
        map: &HexMap<Tile>,
        context: &mut Context,
        id: ObjId,
    ) {
        let agent = state.parts().agent.get(id);
        if agent.moves == Moves(0) && agent.jokers == Jokers(0) {
            return;
        }
        for pos in map.iter() {
            if map.tile(pos).cost() > agent.move_points {
                continue;
            }
            let size = self.tile_size() * 2.0;
            let mut sprite = Sprite::from_path(context, "tile.png", size);
            self.sprites.walkable_tiles.push(sprite.clone());
            let mut color_from = WALKBALE_TILE_COLOR;
            color_from[3] = 0.0;
            sprite.set_color(color_from);
            sprite.set_pos(hex_to_point(self.tile_size(), pos));
            let color_to = WALKBALE_TILE_COLOR;
            let action = Box::new(action::Sequence::new(vec![
                Box::new(action::Show::new(&self.layers().walkable_tiles, &sprite)),
                Box::new(action::ChangeColorTo::new(&sprite, color_to, Time(0.2))),
            ]));
            self.add_action(action);
        }
    }
}
