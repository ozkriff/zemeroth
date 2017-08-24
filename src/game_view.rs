use std::collections::HashMap;
use rand::{thread_rng, Rng};
use cgmath::Vector2;
use hate::{Context, Scene, Sprite, Time};
use hate::scene::action;
use hate::scene::{Action, Layer};
use hate::geom::Point;
use map;
use core::{ObjId, State, TileType};

#[derive(Debug, Clone, Default)]
pub struct Layers {
    pub bg: Layer,
    pub grass: Layer,
    pub walkable_tiles: Layer,
    pub attackable_tiles: Layer,
    pub selection_marker: Layer,
    pub fg: Layer,
    pub text: Layer,
}

impl Layers {
    fn sorted(self) -> Vec<Layer> {
        vec![
            self.bg,
            self.grass,
            self.walkable_tiles,
            self.attackable_tiles,
            self.selection_marker,
            self.fg,
            self.text,
        ]
    }
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
        let layers = Layers::default();
        let scene = Scene::new(layers.clone().sorted());
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

// TODO: move to game.rs?
fn make_map_action(state: &State, view: &GameView, context: &mut Context) -> Box<Action> {
    let mut rng = thread_rng();
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
        if rng.gen_range(0, 10) < 2 {
            let mut sprite = Sprite::from_path(context, "grass.png", view.tile_size() * 2.0);
            let n = view.tile_size() / 2.0;
            let screen_pos_grass = Point(Vector2 {
                x: screen_pos.0.x + rng.gen_range(-n, n),
                y: screen_pos.0.y + rng.gen_range(-n, n),
            });
            sprite.set_pos(screen_pos_grass);
            actions.push(Box::new(action::Show::new(&view.layers.grass, &sprite)));
        }
    }
    Box::new(action::Sequence::new(actions))
}
