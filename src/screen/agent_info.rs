use std::{collections::HashMap, time::Duration};

use gwg::{
    graphics::{self, Color, Image, Point2, Text},
    Context,
};
use ui::{self, Gui, Widget};

use crate::{
    core::battle::{
        ability::{Ability, PassiveAbility},
        component::{self, Component, ObjType, Prototypes},
    },
    screen::{self, Screen, StackCommand},
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

type SpritesInfo = HashMap<String, SpriteInfo>;

fn load_sprites_info(context: &mut Context) -> ZResult<SpritesInfo> {
    let info = utils::deserialize_from_file(context, "/sprites.ron")?;
    Ok(info)
}

fn agent_image(context: &mut Context, typename: &ObjType) -> ZResult<Box<dyn ui::Widget>> {
    let h = 0.3;
    let sprites_info = load_sprites_info(context)?;
    let sprite_info = sprites_info[&typename.0].clone();
    let default_frame = "";
    let default_frame_path = &sprite_info.paths[default_frame];
    let image = Image::new(context, default_frame_path).expect("Can't load agent's image");
    let label = ui::Label::new(context, Box::new(image), h)?
        .with_color(Color::new(1.0, 1.0, 1.0, 1.0))
        .stretchable(true);
    Ok(Box::new(label))
}

#[derive(Clone, Debug)]
enum Message {
    Back,
    AbilityInfo(Ability),
    PassiveAbilityInfo(PassiveAbility),
}

fn info_panel(
    context: &mut Context,
    font: graphics::Font,
    gui: &mut ui::Gui<Message>,
    prototypes: &Prototypes,
    typename: &ObjType,
) -> ZResult<Box<dyn ui::Widget>> {
    let proto = &prototypes.0[&typename];
    let info = StaticObjectInfo::new(&typename, proto);
    let h = utils::line_heights().normal;
    let space_between_buttons = h / 8.0;
    let mut layout = Box::new(ui::VLayout::new().stretchable(true));
    layout.add(agent_image(context, typename)?);
    let mut add = |w| layout.add(w);
    let text_ = |s: &str| Box::new(Text::new((s, font, utils::font_size())));
    let label_ = |context: &mut Context, text: &str| -> ZResult<_> {
        Ok(ui::Label::new(context, text_(text), h)?)
    };
    let label = |context: &mut Context, text: &str| -> ZResult<Box<_>> {
        Ok(Box::new(label_(context, text)?))
    };
    let label_s = |context: &mut Context, text: &str| -> ZResult<_> {
        Ok(Box::new(label_(context, text)?.stretchable(true)))
    };
    let spacer_v = || Box::new(ui::Spacer::new_vertical(h * 0.5));
    let spacer_s = || Box::new(ui::Spacer::new_horizontal(h * 0.5).stretchable(true));
    let line = |context: &mut Context, arg: &str, val: &str| -> ZResult<_> {
        let mut line = ui::HLayout::new().stretchable(true);
        line.add(label(context, arg)?);
        line.add(spacer_s());
        line.add(label(context, val)?);
        Ok(Box::new(line))
    };
    let line_i = |context: &mut Context, arg: &str, val: i32| -> ZResult<_> {
        line(context, arg, &val.to_string())
    };
    {
        if let Some(meta) = info.meta {
            add(label_s(context, &format!("~~~ {} ~~~", meta.name.0))?);
            add(spacer_v());
        }
        if let Some(strength) = info.strength {
            add(line_i(context, "strength:", strength.base_strength.0)?);
        }
        if let Some(a) = info.agent {
            add(line_i(context, "attacks:", a.base_attacks.0)?);
            add(line_i(context, "moves:", a.base_moves.0)?);
            if a.base_jokers.0 != 0 {
                add(line_i(context, "jokers:", a.base_jokers.0)?);
            }
            if a.reactive_attacks.0 != 0 {
                add(line_i(context, "reactive attacks:", a.reactive_attacks.0)?);
            }
            if a.attack_distance.0 != 1 {
                add(line_i(context, "attack distance:", a.attack_distance.0)?);
            }
            add(line_i(context, "attack strength:", a.attack_strength.0)?);
            add(line_i(context, "attack accuracy:", a.attack_accuracy.0)?);
            if a.attack_break.0 > 0 {
                add(line_i(context, "armor break:", a.attack_break.0)?);
            }
            if a.dodge.0 > 0 {
                add(line_i(context, "dodge:", a.dodge.0)?);
            }
            add(line_i(context, "move points:", a.move_points.0)?);
        }
        if let Some(armor) = info.armor {
            let armor = armor.armor.0;
            if armor != 0 {
                add(line_i(context, "armor:", armor)?);
            }
        }
        if let Some(blocker) = info.blocker {
            add(line(context, "weight:", &format!("{}", blocker.weight))?);
        }
        if let Some(abilities) = info.abilities {
            if !abilities.0.is_empty() {
                add(label_s(context, "~ abilities ~")?);
                for ability in &abilities.0 {
                    let s = ability.ability.title();
                    let cooldown = ability.base_cooldown;
                    let text = format!("'{}' (cooldown: {})", s, cooldown);
                    let mut line_layout = ui::HLayout::new().stretchable(true);
                    line_layout.add(label(context, &text)?);
                    line_layout.add(spacer_s());
                    // TODO: Don't reload images every time, preload them (like object frames)
                    let icon = Box::new(graphics::Image::new(context, "/img/icon_info.png")?);
                    let message = Message::AbilityInfo(ability.ability.clone());
                    let button = ui::Button::new(context, icon, h, gui.sender(), message)?;
                    line_layout.add(Box::new(button));
                    add(Box::new(line_layout));
                    add(Box::new(ui::Spacer::new_vertical(space_between_buttons)));
                }
            }
        }
        if let Some(abilities) = info.passive_abilities {
            if !abilities.0.is_empty() {
                add(label_s(context, "~ passive abilities ~")?);
                for &ability in &abilities.0 {
                    let text = format!("'{}'", ability.title());
                    let mut line_layout = ui::HLayout::new().stretchable(true);
                    line_layout.add(label(context, &text)?);
                    line_layout.add(spacer_s());
                    let icon = Box::new(graphics::Image::new(context, "/img/icon_info.png")?);
                    let message = Message::PassiveAbilityInfo(ability);
                    let button = ui::Button::new(context, icon, h, gui.sender(), message)?;
                    line_layout.add(Box::new(button));
                    add(Box::new(line_layout));
                    add(Box::new(ui::Spacer::new_vertical(space_between_buttons)));
                }
            }
        }
    }
    layout.stretch_to_self(context)?;
    Ok(layout)
}

fn button_back(
    context: &mut Context,
    font: graphics::Font,
    gui: &mut ui::Gui<Message>,
    layout_width: f32,
) -> ZResult<Box<dyn ui::Widget>> {
    let h = utils::line_heights().normal;
    let text = Box::new(Text::new(("back", font, utils::font_size())));
    let msg = Message::Back;
    let mut button = ui::Button::new(context, text, h, gui.sender(), msg)?.stretchable(true);
    button.stretch(context, layout_width / 3.0)?;
    button.set_stretchable(false);
    Ok(Box::new(button))
}

#[derive(Debug)]
pub struct AgentInfo {
    font: graphics::Font,
    gui: Gui<Message>,
}

impl AgentInfo {
    pub fn new_agent_info(
        context: &mut Context,
        prototypes: &Prototypes,
        typename: &ObjType,
    ) -> ZResult<Self> {
        let font = utils::default_font(context);
        let mut gui = ui::Gui::new(context);
        let mut layout = ui::VLayout::new();
        let h = utils::line_heights().big;
        layout.add(info_panel(context, font, &mut gui, prototypes, typename)?);
        layout.add(Box::new(ui::Spacer::new_vertical(h)));
        layout.add(button_back(context, font, &mut gui, layout.rect().w)?);
        let layout = utils::add_offsets_and_bg_big(context, Box::new(layout))?;
        let anchor = ui::Anchor(ui::HAnchor::Middle, ui::VAnchor::Middle);
        gui.add(&ui::pack(layout), anchor);
        Ok(Self { font, gui })
    }

    pub fn new_upgrade_info(
        context: &mut Context,
        prototypes: &Prototypes,
        from: &ObjType,
        to: &ObjType,
    ) -> ZResult<Self> {
        let font = utils::default_font(context);
        let mut gui = ui::Gui::new(context);
        let mut layout = ui::VLayout::new();
        let h = utils::line_heights().big;
        let line = {
            let mut line = Box::new(ui::HLayout::new());
            let panel_from = info_panel(context, font, &mut gui, prototypes, from)?;
            let panel_from_height = panel_from.rect().h;
            line.add(panel_from);
            line.add(Box::new(ui::Spacer::new_horizontal(h)));
            let col = {
                let mut col = Box::new(ui::VLayout::new());
                col.add(Box::new(ui::Spacer::new_vertical(panel_from_height * 0.5)));
                let text = Box::new(Text::new(("=>", font, utils::font_size())));
                col.add(Box::new(ui::Label::new(context, text, h)?));
                col
            };
            line.add(col);
            line.add(Box::new(ui::Spacer::new_horizontal(h)));
            line.add(info_panel(context, font, &mut gui, prototypes, to)?);
            line
        };
        layout.add(line);
        layout.add(Box::new(ui::Spacer::new_vertical(h)));
        layout.add(button_back(context, font, &mut gui, layout.rect().w)?);
        let layout = utils::add_offsets_and_bg_big(context, Box::new(layout))?;
        let anchor = ui::Anchor(ui::HAnchor::Middle, ui::VAnchor::Middle);
        gui.add(&ui::pack(layout), anchor);
        Ok(Self { font, gui })
    }
}

impl Screen for AgentInfo {
    fn update(&mut self, _context: &mut Context, _dtime: Duration) -> ZResult<StackCommand> {
        Ok(StackCommand::None)
    }

    fn draw(&self, context: &mut Context) -> ZResult {
        self.gui.draw(context)?;
        Ok(())
    }

    fn click(&mut self, context: &mut Context, pos: Point2) -> ZResult<StackCommand> {
        let message = self.gui.click(pos);
        match message {
            Some(Message::Back) => Ok(StackCommand::Pop),
            Some(Message::AbilityInfo(info)) => {
                let screen = screen::GeneralInfo::new(context, &info.title(), &info.description())?;
                Ok(StackCommand::PushPopup(Box::new(screen)))
            }
            Some(Message::PassiveAbilityInfo(info)) => {
                let screen = screen::GeneralInfo::new(context, &info.title(), &info.description())?;
                Ok(StackCommand::PushPopup(Box::new(screen)))
            }
            None => Ok(StackCommand::None),
        }
    }

    fn resize(&mut self, aspect_ratio: f32) {
        self.gui.resize(aspect_ratio);
    }

    fn move_mouse(&mut self, _context: &mut Context, pos: Point2) -> ZResult {
        self.gui.move_mouse(pos);
        Ok(())
    }
}
