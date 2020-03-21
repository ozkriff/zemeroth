use std::{collections::HashMap, time::Duration};

use ggez::{
    graphics::{self, Text},
    Context,
};
use nalgebra::Point2;
use scene::Sprite;
use ui::{self, Gui};

use crate::{
    core::battle::component::{self, Component, ObjType, Prototypes},
    screen::{Screen, Transition},
    sprite_info::SpriteInfo,
    utils, ZResult,
};

/// This struct is similar to [component::Parts],
/// but stores only the static information about one object type.
#[derive(Clone, Debug, Default)]
struct StaticObjectInfo {
    meta: Option<component::Meta>,
    strength: Option<component::Strength>,
    armor: Option<component::Armor>,
    agent: Option<component::Agent>,
    blocker: Option<component::Blocker>,
    abilities: Option<component::Abilities>,
    passive_abilities: Option<component::PassiveAbilities>,
    summoner: Option<component::Summoner>,
}

impl StaticObjectInfo {
    fn new(typename: &ObjType, components: &[Component]) -> Self {
        let mut this = StaticObjectInfo::default();
        let name = typename.clone();
        this.meta = Some(component::Meta { name });
        for component in components {
            match component.clone() {
                Component::Strength(c) => this.strength = Some(c),
                Component::Armor(c) => this.armor = Some(c),
                Component::Meta(c) => this.meta = Some(c),
                Component::Agent(c) => this.agent = Some(c),
                Component::Abilities(c) => this.abilities = Some(c),
                Component::PassiveAbilities(c) => this.passive_abilities = Some(c),
                Component::Summoner(c) => this.summoner = Some(c),
                Component::Blocker(c) => this.blocker = Some(c),
                Component::BelongsTo(_)
                | Component::Pos(_)
                | Component::Effects(_)
                | Component::Schedule(_) => (),
            }
        }
        this
    }
}

#[derive(Copy, Clone, Debug)]
enum Message {
    Back,
}

#[derive(Debug)]
pub struct AgentInfo {
    font: graphics::Font,
    gui: Gui<Message>,
    agent_sprite: Sprite,
}

impl AgentInfo {
    pub fn new(context: &mut Context, prototypes: Prototypes, typename: &ObjType) -> ZResult<Self> {
        let sprite_info = {
            type SpritesInfo = HashMap<String, SpriteInfo>;
            let sprites_info: SpritesInfo = utils::deserialize_from_file(context, "/sprites.ron")?;
            sprites_info[&typename.0].clone()
        };
        let font = utils::default_font(context);
        let mut gui = ui::Gui::new(context);
        let h = utils::line_heights().big;
        let font_size = utils::font_size();
        let proto = &prototypes.0[&typename];
        let info = StaticObjectInfo::new(&typename, proto);
        {
            let text = Box::new(Text::new(("[back]", font, font_size)));
            let button = ui::Button::new(context, text, h, gui.sender(), Message::Back)?;
            let anchor = ui::Anchor(ui::HAnchor::Left, ui::VAnchor::Top);
            gui.add(&ui::pack(button), anchor);
        }
        let mut layout = ui::VLayout::new();
        {
            let mut line = |text: &str| -> ZResult {
                let text = Box::new(Text::new((text, font, font_size)));
                let label = ui::Label::new(context, text, h)?;
                layout.add(Box::new(label));
                Ok(())
            };
            if let Some(meta) = info.meta {
                line(&format!("name: '{}'", meta.name.0))?;
            }
            if let Some(strength) = info.strength {
                line(&format!("strength: {}", strength.base_strength.0))?;
            }
            if let Some(blocker) = info.blocker {
                line(&format!("weight: {}", blocker.weight))?;
            }
            if let Some(agent) = info.agent {
                line(&format!("attacks: {}", agent.base_attacks.0))?;
                line(&format!("moves: {}", agent.base_moves.0))?;
                if agent.base_jokers.0 != 0 {
                    line(&format!("jokers: {}", agent.base_jokers.0))?;
                }
                if agent.reactive_attacks.0 != 0 {
                    line(&format!("reactive attacks: {}", agent.reactive_attacks.0))?;
                }
                if agent.attack_distance.0 != 1 {
                    line(&format!("attack distance: {}", agent.attack_distance.0))?;
                }
                line(&format!("attack strength: {}", agent.attack_strength.0))?;
                line(&format!("attack accuracy: {}", agent.attack_accuracy.0))?;
                if agent.attack_break.0 > 0 {
                    line(&format!("armor break: {}", agent.attack_break.0))?;
                }
                if agent.dodge.0 > 0 {
                    line(&format!("dodge: {}", agent.dodge.0))?;
                }
                line(&format!("move points: {}", agent.move_points.0))?;
            }
            if let Some(armor) = info.armor {
                let armor = armor.armor.0;
                if armor != 0 {
                    line(&format!("armor: {}", armor))?;
                }
            }
            if let Some(abilities) = info.abilities {
                if !abilities.0.is_empty() {
                    line("abilities:")?;
                    for ability in &abilities.0 {
                        let s = ability.ability.to_string();
                        let cooldown = ability.base_cooldown;
                        line(&format!(" - {} (cooldown: {})", s, cooldown))?;
                    }
                }
            }
            if let Some(abilities) = info.passive_abilities {
                if !abilities.0.is_empty() {
                    line("passive abilities:")?;
                    for ability in &abilities.0 {
                        line(&format!(" - {}", ability.to_string()))?;
                    }
                }
            }
        }
        let anchor = ui::Anchor(ui::HAnchor::Middle, ui::VAnchor::Middle);
        gui.add(&ui::pack(layout), anchor);
        let agent_sprite = {
            let mut sprite = Sprite::from_path(context, &sprite_info.paths[""], 0.4)?;
            sprite.set_centered(true);
            sprite.set_pos(Point2::new(0.6, 0.0));
            sprite
        };
        Ok(Self {
            font,
            gui,
            agent_sprite,
        })
    }
}

impl Screen for AgentInfo {
    fn update(&mut self, _context: &mut Context, _dtime: Duration) -> ZResult<Transition> {
        Ok(Transition::None)
    }

    fn draw(&self, context: &mut Context) -> ZResult {
        self.gui.draw(context)?;
        self.agent_sprite.draw(context)?;
        Ok(())
    }

    fn click(&mut self, _: &mut Context, pos: Point2<f32>) -> ZResult<Transition> {
        let message = self.gui.click(pos);
        match message {
            Some(Message::Back) => Ok(Transition::Pop),
            None => Ok(Transition::None),
        }
    }

    fn resize(&mut self, aspect_ratio: f32) {
        self.gui.resize(aspect_ratio);
    }

    fn move_mouse(&mut self, _context: &mut Context, pos: Point2<f32>) -> ZResult {
        self.gui.move_mouse(pos);
        Ok(())
    }
}
