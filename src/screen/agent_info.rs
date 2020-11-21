use std::time::Duration;

use heck::TitleCase;
use macroquad::prelude::{Color, Vec2};
use ui::{self, Drawable, Gui, Widget};

use crate::{
    assets,
    core::battle::{
        ability::{Ability, PassiveAbility},
        component::{self, Component, ObjType, Prototypes},
    },
    screen::{self, Screen, StackCommand},
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

fn agent_image(typename: &ObjType) -> ZResult<Box<dyn ui::Widget>> {
    let h = 0.3;
    let assets = &assets::get();
    let default_frame = "";
    let texture = Drawable::Texture(assets.sprite_frames[typename][default_frame]);
    let label = ui::Label::new(texture, h)?
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
    gui: &mut ui::Gui<Message>,
    prototypes: &Prototypes,
    typename: &ObjType,
) -> ZResult<Box<dyn ui::Widget>> {
    let font = assets::get().font;
    let proto = &prototypes.0[&typename];
    let info = StaticObjectInfo::new(&typename, proto);
    let h = utils::line_heights().normal;
    let space_between_buttons = h / 8.0;
    let mut layout = Box::new(ui::VLayout::new().stretchable(true));
    layout.add(agent_image(typename)?);
    let mut add = |w| layout.add(w);
    let text_ = |s: &str| ui::Drawable::text(s, font, utils::font_size());
    let label_ = |text: &str| -> ZResult<_> { Ok(ui::Label::new(text_(text), h)?) };
    let label = |text: &str| -> ZResult<Box<_>> { Ok(Box::new(label_(text)?)) };
    let label_s = |text: &str| -> ZResult<_> { Ok(Box::new(label_(text)?.stretchable(true))) };
    let spacer_v = || Box::new(ui::Spacer::new_vertical(h * 0.5));
    let spacer_s = || Box::new(ui::Spacer::new_horizontal(h * 0.5).stretchable(true));
    let line = |arg: &str, val: &str| -> ZResult<_> {
        let mut line = ui::HLayout::new().stretchable(true);
        line.add(label(arg)?);
        line.add(spacer_s());
        line.add(label(val)?);
        Ok(Box::new(line))
    };
    let line_i = |arg: &str, val: i32| -> ZResult<_> { line(arg, &val.to_string()) };
    {
        if let Some(meta) = info.meta {
            let title = meta.name.0.to_title_case();
            add(label_s(&format!("~~~ {} ~~~", title))?);
            add(spacer_v());
        }
        if let Some(strength) = info.strength {
            add(line_i("strength:", strength.base_strength.0)?);
        }
        if let Some(a) = info.agent {
            add(line_i("attacks:", a.base_attacks.0)?);
            add(line_i("moves:", a.base_moves.0)?);
            if a.base_jokers.0 != 0 {
                add(line_i("jokers:", a.base_jokers.0)?);
            }
            if a.reactive_attacks.0 != 0 {
                add(line_i("reactive attacks:", a.reactive_attacks.0)?);
            }
            if a.attack_distance.0 != 1 {
                add(line_i("attack distance:", a.attack_distance.0)?);
            }
            add(line_i("attack strength:", a.attack_strength.0)?);
            add(line_i("attack accuracy:", a.attack_accuracy.0)?);
            if a.attack_break.0 > 0 {
                add(line_i("armor break:", a.attack_break.0)?);
            }
            if a.dodge.0 > 0 {
                add(line_i("dodge:", a.dodge.0)?);
            }
            add(line_i("move points:", a.move_points.0)?);
        }
        if let Some(armor) = info.armor {
            let armor = armor.armor.0;
            if armor != 0 {
                add(line_i("armor:", armor)?);
            }
        }
        if let Some(blocker) = info.blocker {
            add(line("weight:", &format!("{}", blocker.weight))?);
        }
        if let Some(abilities) = info.abilities {
            if !abilities.0.is_empty() {
                add(label_s("~ abilities ~")?);
                for r_ability in &abilities.0 {
                    let s = r_ability.ability.title();
                    let cooldown = r_ability.ability.base_cooldown();
                    let text = format!("{} (cooldown: {}t)", s, cooldown);
                    let mut line_layout = ui::HLayout::new().stretchable(true);
                    line_layout.add(label(&text)?);
                    line_layout.add(spacer_s());
                    let icon = Drawable::Texture(assets::get().textures.icons.info);
                    let message = Message::AbilityInfo(r_ability.ability);
                    let button = ui::Button::new(icon, h, gui.sender(), message)?;
                    line_layout.add(Box::new(button));
                    add(Box::new(line_layout));
                    add(Box::new(ui::Spacer::new_vertical(space_between_buttons)));
                }
            }
        }
        if let Some(abilities) = info.passive_abilities {
            if !abilities.0.is_empty() {
                add(label_s("~ passive abilities ~")?);
                for &ability in &abilities.0 {
                    let mut line_layout = ui::HLayout::new().stretchable(true);
                    line_layout.add(label(&ability.title())?);
                    line_layout.add(spacer_s());
                    let icon = Drawable::Texture(assets::get().textures.icons.info);
                    let message = Message::PassiveAbilityInfo(ability);
                    let button = ui::Button::new(icon, h, gui.sender(), message)?;
                    line_layout.add(Box::new(button));
                    add(Box::new(line_layout));
                    add(Box::new(ui::Spacer::new_vertical(space_between_buttons)));
                }
            }
        }
    }
    layout.stretch_to_self()?;
    Ok(layout)
}

fn button_back(gui: &mut ui::Gui<Message>, layout_width: f32) -> ZResult<Box<dyn ui::Widget>> {
    let font = assets::get().font;
    let h = utils::line_heights().normal;
    let text = ui::Drawable::text("back", font, utils::font_size());
    let msg = Message::Back;
    let mut button = ui::Button::new(text, h, gui.sender(), msg)?.stretchable(true);
    button.stretch(layout_width / 3.0)?;
    button.set_stretchable(false);
    Ok(Box::new(button))
}

#[derive(Debug)]
pub struct AgentInfo {
    gui: Gui<Message>,
}

impl AgentInfo {
    pub fn new_agent_info(prototypes: &Prototypes, typename: &ObjType) -> ZResult<Self> {
        let mut gui = ui::Gui::new();
        let mut layout = ui::VLayout::new();
        let h = utils::line_heights().big;
        layout.add(info_panel(&mut gui, prototypes, typename)?);
        layout.add(Box::new(ui::Spacer::new_vertical(h)));
        layout.add(button_back(&mut gui, layout.rect().w)?);
        let layout = utils::add_offsets_and_bg_big(Box::new(layout))?;
        let anchor = ui::Anchor(ui::HAnchor::Middle, ui::VAnchor::Middle);
        gui.add(&ui::pack(layout), anchor);
        Ok(Self { gui })
    }

    pub fn new_upgrade_info(
        prototypes: &Prototypes,
        from: &ObjType,
        to: &ObjType,
    ) -> ZResult<Self> {
        let font = assets::get().font;
        let mut gui = ui::Gui::new();
        let mut layout = ui::VLayout::new();
        let h = utils::line_heights().big;
        let line = {
            let mut line = Box::new(ui::HLayout::new());
            let panel_from = info_panel(&mut gui, prototypes, from)?;
            let panel_from_height = panel_from.rect().h;
            line.add(panel_from);
            line.add(Box::new(ui::Spacer::new_horizontal(h)));
            let col = {
                let mut col = Box::new(ui::VLayout::new());
                col.add(Box::new(ui::Spacer::new_vertical(panel_from_height * 0.5)));
                let text = ui::Drawable::text("=>", font, utils::font_size());
                col.add(Box::new(ui::Label::new(text, h)?));
                col
            };
            line.add(col);
            line.add(Box::new(ui::Spacer::new_horizontal(h)));
            line.add(info_panel(&mut gui, prototypes, to)?);
            line
        };
        layout.add(line);
        layout.add(Box::new(ui::Spacer::new_vertical(h)));
        layout.add(button_back(&mut gui, layout.rect().w)?);
        let layout = utils::add_offsets_and_bg_big(Box::new(layout))?;
        let anchor = ui::Anchor(ui::HAnchor::Middle, ui::VAnchor::Middle);
        gui.add(&ui::pack(layout), anchor);
        Ok(Self { gui })
    }
}

impl Screen for AgentInfo {
    fn update(&mut self, _dtime: Duration) -> ZResult<StackCommand> {
        Ok(StackCommand::None)
    }

    fn draw(&self) -> ZResult {
        self.gui.draw();
        Ok(())
    }

    fn click(&mut self, pos: Vec2) -> ZResult<StackCommand> {
        let message = self.gui.click(pos);
        match message {
            Some(Message::Back) => Ok(StackCommand::Pop),
            Some(Message::AbilityInfo(info)) => {
                let mut description = info.description();
                description.push(format!("Cooldown: {}t", info.base_cooldown()));
                let screen = screen::GeneralInfo::new(&info.title(), &description)?;
                Ok(StackCommand::PushPopup(Box::new(screen)))
            }
            Some(Message::PassiveAbilityInfo(info)) => {
                let screen = screen::GeneralInfo::new(&info.title(), &info.description())?;
                Ok(StackCommand::PushPopup(Box::new(screen)))
            }
            None => Ok(StackCommand::None),
        }
    }

    fn resize(&mut self, aspect_ratio: f32) {
        self.gui.resize_if_needed(aspect_ratio);
    }

    fn move_mouse(&mut self, pos: Vec2) -> ZResult {
        self.gui.move_mouse(pos);
        Ok(())
    }
}
