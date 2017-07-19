use std::collections::HashMap;
use hate::{Time, Sprite, Context, Scene};
use hate::scene::action;
use hate::scene::{Action, Layer};
use map;
use core::{ObjId, TileType, State};

#[derive(Debug)]
pub struct Layers {
    pub bg: Layer,
    pub walkable_tiles: Layer,
    pub attackable_tiles: Layer,
    pub selection_marker: Layer,
    pub fg: Layer,
    pub text: Layer,
}

#[derive(Debug)]
pub struct GameView {
    tile_size: f32,
    layers: Layers,
    obj_to_sprite_map: HashMap<ObjId, Sprite>,
    scene: Scene,
}

impl GameView {
    pub fn new(state: &State, context: &mut Context) -> Self {
        let obj_to_sprite_map = HashMap::new();
        let layers = Layers {
            bg: Layer::new(),
            walkable_tiles: Layer::new(),
            attackable_tiles: Layer::new(),
            selection_marker: Layer::new(),
            fg: Layer::new(),
            text: Layer::new(),
        };
        let scene = Scene::new(vec![
            layers.bg.clone(),
            layers.walkable_tiles.clone(),
            layers.attackable_tiles.clone(),
            layers.selection_marker.clone(),
            layers.fg.clone(),
            layers.text.clone(),
        ]);
        let tile_size = 0.1;
        let mut this = Self {
            scene,
            tile_size,
            layers,
            obj_to_sprite_map,
        };
        let map_action = make_map_action(state, &this, context);
        this.scene.add_action(map_action);
        this
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
        self.obj_to_sprite_map.insert(id, sprite.clone());
    }

    pub fn remove_object(&mut self, id: ObjId) {
        self.obj_to_sprite_map.remove(&id).unwrap();
    }

    pub fn id_to_sprite(&mut self, id: ObjId) -> &Sprite {
        &self.obj_to_sprite_map[&id]
    }
}

fn make_map_action(state: &State, view: &GameView, context: &mut Context) -> Box<Action> {
    let mut actions: Vec<Box<Action>> = Vec::new();
    for hex_pos in state.map().iter() {
        let screen_pos = map::hex_to_point(view.tile_size(), hex_pos);
        let mut sprite = Sprite::from_path(context, "tile.png", view.tile_size() * 2.0);
        match state.map().tile(hex_pos) {
            TileType::Floor => sprite.set_color([1.0, 1.0, 1.0, 1.0]),
            TileType::Lava => sprite.set_color([1.0, 0.7, 0.7, 1.0]),
        }
        sprite.set_pos(screen_pos);
        actions.push(Box::new(action::Show::new(&view.layers.bg, &sprite)));
    }
    Box::new(action::Sequence::new(actions))
}
