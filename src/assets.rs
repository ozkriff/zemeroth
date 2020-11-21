//! This module groups all the async loading stuff.

// TODO: https://github.com/rust-lang/rust-clippy/issues/4637
#![allow(clippy::eval_order_dependence)]

use std::{collections::HashMap, hash::Hash};

use macroquad::{
    file::load_file,
    text::{self, Font},
    texture::{load_texture, Texture2D},
};
use once_cell::sync::OnceCell;
use serde::{de::DeserializeOwned, Deserialize};

use crate::{
    core::{
        battle::{
            ability::Ability,
            component::{ObjType, Prototypes, WeaponType},
            effect,
            scenario::Scenario,
        },
        campaign,
    },
    error::ZError,
    ZResult,
};

static INSTANCE: OnceCell<Assets> = OnceCell::new();

pub async fn load_assets() {
    assert!(INSTANCE.get().is_none());
    let assets = Assets::load().await.expect("Can't load assets");
    INSTANCE.set(assets).expect("Can't set assets instance");
}

pub fn get() -> &'static Assets {
    INSTANCE.get().expect("Assets weren't loaded")
}

/// Read a file to a string.
async fn read_file(path: &str) -> ZResult<String> {
    let data = load_file(path).await.expect("Can't load the file"); // TODO: replace expect with ?
    Ok(String::from_utf8_lossy(&data[..]).to_string())
}

async fn deserialize_from_file<D: DeserializeOwned>(path: &str) -> ZResult<D> {
    let s = read_file(path).await?;
    ron::de::from_str(&s).map_err(|e| ZError::from_ron_de_error(e, path.into()))
}

async fn load_map<Key: Hash + Eq + Copy>(
    table: &[(Key, &str)],
    expand_path: fn(&str) -> String,
) -> HashMap<Key, Texture2D> {
    let mut map = HashMap::new();
    for &(key, path) in table {
        map.insert(key, load_texture(&expand_path(path)).await);
    }
    map
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpriteInfo {
    pub paths: HashMap<String, String>,
    pub offset_x: f32,
    pub offset_y: f32,
    pub shadow_size_coefficient: f32,

    #[serde(default = "default_sub_tile_z")]
    pub sub_tile_z: f32,
}

fn default_sub_tile_z() -> f32 {
    0.0
}

type SpritesInfo = HashMap<ObjType, SpriteInfo>;

#[derive(Debug)]
pub struct Assets {
    pub images: Images,
    pub font: Font,
    pub sprites_info: SpritesInfo,
    pub sprite_frames: HashMap<ObjType, HashMap<String, Texture2D>>,
    pub prototypes: Prototypes,
    pub demo_scenario: Scenario,
    pub campaign_plan: campaign::Plan,
    pub agent_campaign_info: HashMap<ObjType, campaign::AgentInfo>,
}

impl Assets {
    pub async fn load() -> ZResult<Self> {
        let sprites_info: SpritesInfo = deserialize_from_file("assets/sprites.ron").await?;
        let sprite_frames = {
            let mut sprite_frames = HashMap::new();
            for (obj_type, SpriteInfo { paths, .. }) in sprites_info.iter() {
                let mut frames = HashMap::new();
                for (frame_name, path) in paths {
                    frames.insert(frame_name.to_string(), load_texture(path).await);
                }
                sprite_frames.insert(obj_type.clone(), frames);
            }
            sprite_frames
        };
        Ok(Self {
            images: Images::load().await,
            font: text::load_ttf_font("assets/OpenSans-Regular.ttf").await,
            sprites_info,
            sprite_frames,
            prototypes: Prototypes::from_str(&read_file("assets/objects.ron").await?),
            demo_scenario: deserialize_from_file("assets/scenario_01.ron").await?,
            campaign_plan: deserialize_from_file("assets/campaign_01.ron").await?,
            agent_campaign_info: deserialize_from_file("assets/agent_campaign_info.ron").await?,
        })
    }
}

// TODO: rename to Textures (and do similar renames all over the project)
#[derive(Debug)]
pub struct Images {
    pub map: MapObjectImages,
    pub weapon_flashes: HashMap<WeaponType, Texture2D>,
    pub icons: IconImages,
    pub dot: Texture2D,
}

impl Images {
    async fn load() -> Self {
        Self {
            map: MapObjectImages::load().await,
            weapon_flashes: load_weapon_flashes().await,
            icons: IconImages::load().await,
            dot: load_texture("assets/img/dot.png").await,
        }
    }
}

#[derive(Debug)]
pub struct MapObjectImages {
    pub selection: Texture2D,
    pub white_hex: Texture2D,
    pub tile: Texture2D,
    pub tile_rocks: Texture2D,
    pub grass: Texture2D,
    pub blood: Texture2D,
    pub explosion_ground_mark: Texture2D,
    pub shadow: Texture2D,
}

impl MapObjectImages {
    async fn load() -> Self {
        Self {
            selection: load_texture("assets/img/selection.png").await,
            white_hex: load_texture("assets/img/white_hex.png").await,
            tile: load_texture("assets/img/tile.png").await,
            tile_rocks: load_texture("assets/img/tile_rocks.png").await,
            grass: load_texture("assets/img/grass.png").await,
            blood: load_texture("assets/img/blood.png").await,
            explosion_ground_mark: load_texture("assets/img/explosion_ground_mark.png").await,
            shadow: load_texture("assets/img/shadow.png").await,
        }
    }
}

#[derive(Debug)]
pub struct IconImages {
    pub info: Texture2D,
    pub end_turn: Texture2D,
    pub main_menu: Texture2D,
    pub abilities: HashMap<Ability, Texture2D>,
    pub lasting_effects: HashMap<effect::Lasting, Texture2D>,
}

impl IconImages {
    async fn load() -> Self {
        Self {
            info: load_texture("assets/img/icon_info.png").await,
            end_turn: load_texture("assets/img/icon_end_turn.png").await,
            main_menu: load_texture("assets/img/icon_menu.png").await,
            abilities: load_ability_icons().await,
            lasting_effects: load_lasting_effects().await,
        }
    }
}

async fn load_weapon_flashes() -> HashMap<WeaponType, Texture2D> {
    let map = &[
        (WeaponType::Slash, "slash"),
        (WeaponType::Smash, "smash"),
        (WeaponType::Pierce, "pierce"),
        (WeaponType::Claw, "claw"),
    ];
    load_map(map, |s| format!("assets/img/{}.png", s)).await
}

async fn load_ability_icons() -> HashMap<Ability, Texture2D> {
    let map = &[
        (Ability::Knockback, "knockback"),
        (Ability::Club, "club"),
        (Ability::Jump, "jump"),
        (Ability::LongJump, "long_jump"),
        (Ability::Bomb, "bomb"),
        (Ability::BombPush, "bomb_push"),
        (Ability::BombFire, "bomb_fire"),
        (Ability::BombPoison, "bomb_poison"),
        (Ability::BombDemonic, "bomb_demonic"),
        (Ability::Summon, "summon"),
        (Ability::Dash, "dash"),
        (Ability::Rage, "rage"),
        (Ability::Heal, "heal"),
        (Ability::GreatHeal, "great_heal"),
        (Ability::Bloodlust, "bloodlust"),
    ];
    load_map(map, |s| format!("assets/img/icon_ability_{}.png", s)).await
}

async fn load_lasting_effects() -> HashMap<effect::Lasting, Texture2D> {
    let map = &[
        (effect::Lasting::Stun, "stun"),
        (effect::Lasting::Poison, "poison"),
        (effect::Lasting::Bloodlust, "bloodlust"),
    ];
    load_map(map, |s| format!("assets/img/effect_{}.png", s)).await
}
