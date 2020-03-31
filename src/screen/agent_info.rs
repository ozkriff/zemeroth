use std::{collections::HashMap, time::Duration};

use cgmath::Point2;
use gwg::{
    graphics::{self, Text},
    Context,
};
use ui::{self, Gui};
use zscene::Sprite;

use crate::{
    core::battle::{
        ability::{Ability, PassiveAbility},
        component::{self, Component, ObjType, Prototypes},
    },
    screen::{self, ability_info::ActiveOrPassiveAbility, Screen, StackCommand},
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

#[derive(Clone, Debug)]
enum Message {
    Back,
    AbilityInfo(Ability),
    PassiveAbilityInfo(PassiveAbility),
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

        let mut layout = ui::VLayout::new();
        {
            let label = |context: &mut Context, text: &str| -> ZResult<Box<dyn ui::Widget>> {
                let text = Box::new(Text::new((text, font, font_size)));
                Ok(Box::new(ui::Label::new(context, text, h)?))
            };
            let mut add = |w| layout.add(w);
            let spacer = || Box::new(ui::Spacer::new_vertical(h * 0.5));

            if let Some(meta) = info.meta {
                add(label(context, &format!("name: '{}'", meta.name.0))?);
                add(spacer());
            }
            if let Some(strength) = info.strength {
                add(label(
                    context,
                    &format!("strength: {}", strength.base_strength.0),
                )?);
            }
            if let Some(blocker) = info.blocker {
                add(label(context, &format!("weight: {}", blocker.weight))?);
            }
            if let Some(agent) = info.agent {
                add(label(
                    context,
                    &format!("attacks: {}", agent.base_attacks.0),
                )?);
                add(label(context, &format!("moves: {}", agent.base_moves.0))?);
                if agent.base_jokers.0 != 0 {
                    add(label(context, &format!("jokers: {}", agent.base_jokers.0))?);
                }
                if agent.reactive_attacks.0 != 0 {
                    add(label(
                        context,
                        &format!("reactive attacks: {}", agent.reactive_attacks.0),
                    )?);
                }
                if agent.attack_distance.0 != 1 {
                    add(label(
                        context,
                        &format!("attack distance: {}", agent.attack_distance.0),
                    )?);
                }
                add(label(
                    context,
                    &format!("attack strength: {}", agent.attack_strength.0),
                )?);
                add(label(
                    context,
                    &format!("attack accuracy: {}", agent.attack_accuracy.0),
                )?);
                if agent.attack_break.0 > 0 {
                    add(label(
                        context,
                        &format!("armor break: {}", agent.attack_break.0),
                    )?);
                }
                if agent.dodge.0 > 0 {
                    add(label(context, &format!("dodge: {}", agent.dodge.0))?);
                }
                add(label(
                    context,
                    &format!("move points: {}", agent.move_points.0),
                )?);
            }
            if let Some(armor) = info.armor {
                let armor = armor.armor.0;
                if armor != 0 {
                    add(label(context, &format!("armor: {}", armor))?);
                }
            }
            if let Some(abilities) = info.abilities {
                if !abilities.0.is_empty() {
                    add(label(context, "abilities:")?);
                    for ability in &abilities.0 {
                        let s = ability.ability.title();
                        let cooldown = ability.base_cooldown;
                        let text = format!(" - {} (cooldown: {})", s, cooldown);
                        let mut line_layout = ui::HLayout::new();
                        line_layout.add(label(context, &text)?);
                        // TODO: Don't reload images every time, preload them (like object frames)
                        let icon = Box::new(graphics::Image::new(context, "/icon_info.png")?);
                        let message = Message::AbilityInfo(ability.ability.clone());
                        let button = ui::Button::new(context, icon, h, gui.sender(), message)?;
                        line_layout.add(Box::new(button));
                        add(Box::new(line_layout));
                    }
                }
            }
            if let Some(abilities) = info.passive_abilities {
                if !abilities.0.is_empty() {
                    add(label(context, "passive abilities:")?);
                    for &ability in &abilities.0 {
                        let text = format!(" - {}", ability.title());
                        let mut line_layout = ui::HLayout::new();
                        line_layout.add(label(context, &text)?);
                        let icon = Box::new(graphics::Image::new(context, "/icon_info.png")?);
                        let message = Message::PassiveAbilityInfo(ability);
                        let button = ui::Button::new(context, icon, h, gui.sender(), message)?;
                        line_layout.add(Box::new(button));
                        add(Box::new(line_layout));
                    }
                }
            }
            add(spacer());
            {
                let text = Box::new(Text::new(("back", font, font_size)));
                let button = ui::Button::new(context, text, h, gui.sender(), Message::Back)?;
                add(Box::new(button));
            }
        }

        let layout = utils::wrap_widget_and_add_bg(context, Box::new(layout))?;

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
    fn update(&mut self, _context: &mut Context, _dtime: Duration) -> ZResult<StackCommand> {
        Ok(StackCommand::None)
    }

    fn draw(&self, context: &mut Context) -> ZResult {
        self.gui.draw(context)?;
        self.agent_sprite.draw(context)?;
        Ok(())
    }

    fn click(&mut self, context: &mut Context, pos: Point2<f32>) -> ZResult<StackCommand> {
        let message = self.gui.click(pos);
        match message {
            Some(Message::Back) => Ok(StackCommand::Pop),
            Some(Message::AbilityInfo(info)) => {
                let ability = ActiveOrPassiveAbility::Active(info);
                let screen = screen::AbilityInfo::new(context, ability)?;
                Ok(StackCommand::PushPopup(Box::new(screen)))
            }
            Some(Message::PassiveAbilityInfo(info)) => {
                let ability = ActiveOrPassiveAbility::Passive(info);
                let screen = screen::AbilityInfo::new(context, ability)?;
                Ok(StackCommand::PushPopup(Box::new(screen)))
            }
            None => Ok(StackCommand::None),
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
