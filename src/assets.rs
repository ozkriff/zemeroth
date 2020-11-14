use std::{collections::HashMap, sync::Mutex};

use macroquad::{
    text::{self, Font},
    texture::{load_texture, Texture2D},
};
use once_cell::sync::OnceCell;

use crate::{
    core::{
        battle::{
            component::{ObjType, Prototypes},
            scenario::Scenario,
        },
        campaign,
    },
    sprite_info::SpriteInfo,
    utils::{self, deserialize_from_file},
    ZResult,
};
use macroquad::texture::load_image;

static INSTANCE: OnceCell<Assets> = OnceCell::new();

pub async fn load_assets() {
    let assets = Assets::load().await.expect("TODO: err msg (important)");
    INSTANCE.set(assets).expect("TODO: err msg");
}

pub fn get() -> &'static Assets {
    INSTANCE.get().expect("TODO: err msg")
}

// type SpritesInfo = HashMap<String, SpriteInfo>;
type SpritesInfo = HashMap<ObjType, SpriteInfo>;

#[derive(Debug)]
pub struct Assets {
    pub images: Images,
    pub font: Font,

    pub sprites_info: SpritesInfo,
    pub sprite_frames: HashMap<ObjType, HashMap<String, Texture2D>>,

    // TODO: core configs
    // TODO: visual configs
    pub prototypes: Prototypes,
    pub demo_scenario: Scenario,
    pub campaign_plan: campaign::Plan,
    pub agent_campaign_info: HashMap<ObjType, campaign::AgentInfo>,
}

impl Assets {
    pub async fn load() -> ZResult<Self> {
        let images = Images::load().await;
        let font = text::load_ttf_font("assets/OpenSans-Regular.ttf").await;
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
        let prototypes = Prototypes::from_str(&utils::read_file("assets/objects.ron").await?);
        let demo_scenario = deserialize_from_file("assets/scenario_01.ron").await?;
        let campaign_plan = deserialize_from_file("assets/campaign_01.ron").await?;
        let agent_campaign_info = deserialize_from_file("assets/agent_campaign_info.ron").await?;
        Ok(Self {
            images,
            font,
            sprites_info,
            sprite_frames,
            prototypes,
            demo_scenario,
            campaign_plan,
            agent_campaign_info,
        })
    }
}

#[derive(Debug)]
pub struct Images {
    pub selection: Texture2D,
    pub white_hex: Texture2D,
    pub tile: Texture2D,
    pub tile_rocks: Texture2D,
    pub grass: Texture2D,
    pub dot: Texture2D,
    pub blood: Texture2D,
    pub explosion_ground_mark: Texture2D,
    pub shadow: Texture2D,
    pub attack_slash: Texture2D,
    pub attack_smash: Texture2D,
    pub attack_pierce: Texture2D,
    pub attack_claws: Texture2D,
    pub effect_stun: Texture2D,
    pub effect_poison: Texture2D,
    pub effect_bloodlust: Texture2D,
    pub icon_info: Texture2D,
    pub icon_end_turn: Texture2D,
    pub icon_main_menu: Texture2D,
}

impl Images {
    pub async fn load() -> Self {
        Self {
            selection: load_texture("assets/img/selection.png").await,
            white_hex: load_texture("assets/img/white_hex.png").await,
            tile: load_texture("assets/img/tile.png").await,
            tile_rocks: load_texture("assets/img/tile_rocks.png").await,
            grass: load_texture("assets/img/grass.png").await,
            dot: load_texture("assets/img/dot.png").await,
            blood: load_texture("assets/img/blood.png").await,
            explosion_ground_mark: load_texture("assets/img/explosion_ground_mark.png").await,
            shadow: load_texture("assets/img/shadow.png").await,
            attack_slash: load_texture("assets/img/slash.png").await,
            attack_smash: load_texture("assets/img/smash.png").await,
            attack_pierce: load_texture("assets/img/pierce.png").await,
            attack_claws: load_texture("assets/img/claw.png").await,
            effect_stun: load_texture("assets/img/effect_stun.png").await,
            effect_poison: load_texture("assets/img/effect_poison.png").await,
            effect_bloodlust: load_texture("assets/img/effect_bloodlust.png").await,
            icon_info: load_texture("assets/img/icon_info.png").await,
            icon_end_turn: load_texture("assets/img/icon_end_turn.png").await,
            icon_main_menu: load_texture("assets/img/icon_menu.png").await,
        }
    }
}
