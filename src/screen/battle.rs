use std::{
    sync::mpsc::{channel, Receiver, Sender},
    time::Duration,
};

use heck::TitleCase;
use log::{info, trace};
use mq::{color::Color, math::Vec2};

use ui::{self, Gui, Widget};
use zscene::{action, Action, Boxed};

use crate::{
    assets,
    core::{
        battle::{
            self,
            ability::{self, Ability, PassiveAbility},
            ai::Ai,
            check, command,
            component::Prototypes,
            effect,
            movement::Pathfinder,
            scenario,
            state::{self, BattleResult},
            Id, PlayerId, State,
        },
        map::PosHex,
    },
    geom,
    screen::{
        self,
        battle::{
            view::{make_action_create_map, BattleView, SelectionMode},
            visualize::{color, fork, visualize},
        },
        Screen, StackCommand,
    },
    utils::{self, line_heights, time_s},
    ZResult,
};

mod view;
mod visualize;

const FONT_SIZE: u16 = utils::font_size();

#[derive(Clone, Debug)]
enum Message {
    Exit,
    EndTurn,
    Ability(Ability),
    PassiveAbilityInfo(PassiveAbility),
    LastingEffectInfo(effect::Lasting),
}

fn textures() -> &'static assets::Textures {
    &assets::get().textures
}

fn line_with_info_button(
    gui: &mut Gui<Message>,
    text: &str,
    message: Message,
) -> ZResult<Box<dyn ui::Widget>> {
    let h = line_heights().normal;
    let font = assets::get().font;
    let text = ui::Drawable::text(text, font, FONT_SIZE);
    let icon = textures().icons.info;
    let button = ui::Button::new(ui::Drawable::Texture(icon), h, gui.sender(), message)?;
    let mut line = Box::new(ui::HLayout::new().stretchable(true));
    line.add(Box::new(ui::Label::new(text, h)?));
    line.add(Box::new(ui::Spacer::new_horizontal(0.0).stretchable(true)));
    line.add(Box::new(button));
    Ok(line)
}

// TODO: consider moving ui `build_*` functions to a sub-module
fn build_panel_agent_info(gui: &mut Gui<Message>, state: &State, id: Id) -> ZResult<ui::RcWidget> {
    let font = assets::get().font;
    let parts = state.parts();
    let st = parts.strength.get(id);
    let meta = parts.meta.get(id);
    let a = parts.agent.get(id);
    let mut layout = Box::new(ui::VLayout::new().stretchable(true));
    let h = line_heights().normal;
    let space_between_buttons = h / 8.0;
    let mut add = |w| layout.add(w);
    let text_ = |s: &str| ui::Drawable::text(s, font, FONT_SIZE);
    let label_ = |text: &str| -> ZResult<_> { Ok(ui::Label::new(text_(text), h)?) };
    let label = |text: &str| -> ZResult<Box<dyn Widget>> { Ok(Box::new(label_(text)?)) };
    let label_s = |text: &str| -> ZResult<_> { Ok(Box::new(label_(text)?.stretchable(true))) };
    let line = |arg: &str, val: &str| -> ZResult<_> {
        let mut line = ui::HLayout::new().stretchable(true);
        line.add(label(arg)?);
        line.add(Box::new(ui::Spacer::new_horizontal(h).stretchable(true)));
        line.add(label(val)?);
        Ok(Box::new(line))
    };
    let line_i = |arg: &str, val: i32| -> ZResult<_> { line(arg, &val.to_string()) };
    let line_dot = |arg: &str, val: &str, color: Color| -> ZResult<_> {
        let mut line = ui::HLayout::new().stretchable(true);
        let dot_color = Color { a: 1.0, ..color };
        let param = ui::LabelParam {
            drawable_k: 0.3,
            ..Default::default()
        };
        let label_dot = ui::Label::from_params(ui::Drawable::Texture(textures().dot), h, param)?
            .with_color(dot_color);
        line.add(Box::new(label_dot));
        line.add(Box::new(ui::Spacer::new_horizontal(h * 0.1)));
        line.add(label(arg)?);
        line.add(Box::new(ui::Spacer::new_horizontal(h).stretchable(true)));
        line.add(label(val)?);
        Ok(Box::new(line))
    };
    {
        let title = meta.name.0.to_title_case();
        add(label_s(&format!("~~~ {} ~~~", title))?);
        add(line_dot(
            "strength:",
            &format!("{}/{}", st.strength.0, st.base_strength.0),
            color::STRENGTH,
        )?);
        if let Some(armor) = parts.armor.get_opt(id) {
            let armor = armor.armor.0;
            if armor != 0 {
                add(line_dot("armor:", &armor.to_string(), color::ARMOR)?);
            }
        }
        if a.jokers.0 != 0 || a.base_jokers.0 != 0 {
            add(line_dot(
                "jokers:",
                &format!("{}/{}", a.jokers.0, a.base_jokers.0),
                color::JOKERS,
            )?);
        }
        add(line_dot(
            "attacks:",
            &format!("{}/{}", a.attacks.0, a.base_attacks.0),
            color::ATTACKS,
        )?);
        if a.reactive_attacks.0 != 0 {
            add(line_dot(
                "reactive attacks:",
                &a.reactive_attacks.0.to_string(),
                color::ATTACKS,
            )?);
        }
        add(line_dot(
            "moves:",
            &format!("{}/{}", a.moves.0, a.base_moves.0),
            color::MOVES,
        )?);
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
        if let Some(blocker) = parts.blocker.get_opt(id) {
            add(line("weight:", &blocker.weight.to_string())?);
        }
        if let Some(abilities) = parts.passive_abilities.get_opt(id) {
            if !abilities.0.is_empty() {
                add(label_s("~ passive abilities ~")?);
                for &ability in &abilities.0 {
                    let text = ability.title();
                    let message = Message::PassiveAbilityInfo(ability);
                    add(line_with_info_button(gui, &text, message)?);
                    add(Box::new(ui::Spacer::new_vertical(space_between_buttons)));
                }
            }
        }
        if let Some(effects) = parts.effects.get_opt(id) {
            if !effects.0.is_empty() {
                add(label_s("~ effects ~")?);
                for effect in &effects.0 {
                    let s = effect.effect.title();
                    let text = match effect.duration {
                        effect::Duration::Forever => s.into(),
                        effect::Duration::Rounds(n) => format!("{} ({}t)", s, n),
                    };
                    let message = Message::LastingEffectInfo(effect.effect);
                    let text = ui::Drawable::text(text, font, FONT_SIZE);
                    let tex_info = ui::Drawable::Texture(textures().icons.info);
                    let button_info = ui::Button::new(tex_info, h, gui.sender(), message)?;
                    let icon_effect = visualize::get_effect_icon(&effect.effect);
                    let param = ui::LabelParam {
                        drawable_k: 0.6,
                        ..Default::default()
                    };
                    let label_effect =
                        ui::Label::from_params(ui::Drawable::Texture(icon_effect), h, param)?
                            .with_color(Color::new(1.0, 1.0, 1.0, 1.0));
                    let mut line = Box::new(ui::HLayout::new().stretchable(true));
                    line.add(Box::new(label_effect));
                    line.add(Box::new(ui::Spacer::new_horizontal(h * 0.1)));
                    line.add(Box::new(ui::Label::new(text, h)?));
                    line.add(Box::new(ui::Spacer::new_horizontal(0.0).stretchable(true)));
                    line.add(Box::new(button_info));
                    add(line);
                    add(Box::new(ui::Spacer::new_vertical(space_between_buttons)));
                }
            }
        }
    }
    layout.stretch_to_self();
    let layout = utils::add_offsets_and_bg(layout, utils::OFFSET_SMALL)?;
    let layout = ui::pack(layout);
    let anchor = ui::Anchor(ui::HAnchor::Left, ui::VAnchor::Bottom);
    gui.add(&layout, anchor);
    Ok(layout)
}

fn build_panel_agent_abilities(
    gui: &mut Gui<Message>,
    state: &State,
    id: Id,
    mode: &SelectionMode,
) -> ZResult<Option<ui::RcWidget>> {
    let font = assets::get().font;
    let parts = state.parts();
    let abilities = match parts.abilities.get_opt(id) {
        Some(abilities) => &abilities.0,
        None => return Ok(None),
    };
    let mut layout = ui::VLayout::new().stretchable(true);
    let h = line_heights().large;
    for ability in abilities {
        let icons = &assets::get().textures.icons.abilities;
        let texture = *icons.get(&ability.ability).expect("No such icon found");
        let drawable = ui::Drawable::Texture(texture);
        let msg = Message::Ability(ability.ability);
        let mut button = ui::Button::new(drawable, h, gui.sender(), msg)?;
        if !state::can_agent_use_ability(state, id, &ability.ability) {
            button.set_active(false);
        }
        if let SelectionMode::Ability(selected_ability) = mode {
            if selected_ability == &ability.ability {
                button.set_color(Color::new(0.0, 0.0, 0.9, 1.0));
            }
        }
        if let ability::Status::Cooldown(n) = ability.status {
            let mut layers = ui::LayersLayout::new();
            layers.add(Box::new(button));
            let text = ui::Drawable::text(format!(" ({})", n).as_str(), font, FONT_SIZE);
            let label = ui::Label::new(text, h / 2.0)?;
            layers.add(Box::new(label));
            layout.add(Box::new(layers));
        } else {
            layout.add(Box::new(button));
        }
        layout.add(Box::new(ui::Spacer::new_vertical(h / 8.0)));
    }
    let anchor = ui::Anchor(ui::HAnchor::Right, ui::VAnchor::Middle);
    let packed_layout = ui::pack(layout);
    gui.add(&packed_layout, anchor);
    Ok(Some(packed_layout))
}

fn build_panel_end_turn(gui: &mut Gui<Message>) -> ZResult<ui::RcWidget> {
    let h = line_heights().large;
    let tex = ui::Drawable::Texture(textures().icons.end_turn);
    let button = ui::Button::new(tex, h, gui.sender(), Message::EndTurn)?;
    let layout = ui::VLayout::from_widget(Box::new(button));
    let anchor = ui::Anchor(ui::HAnchor::Right, ui::VAnchor::Bottom);
    let packed_layout = ui::pack(layout);
    gui.add(&packed_layout, anchor);
    Ok(packed_layout)
}

fn build_panel_ability_description(
    gui: &mut Gui<Message>,
    state: &State,
    ability: &Ability,
    id: Id,
) -> ZResult<ui::RcWidget> {
    let font = assets::get().font;
    let text = |s: &str| ui::Drawable::text(s, font, FONT_SIZE);
    let h = line_heights().normal;
    let mut layout = Box::new(ui::VLayout::new().stretchable(true));
    let text_title = text(&format!("~~~ {} ~~~", ability.title()));
    let label_title = ui::Label::new(text_title, h)?.stretchable(true);
    layout.add(Box::new(label_title));
    layout.add(Box::new(ui::Spacer::new_vertical(h / 2.0)));
    for line in ability.description() {
        layout.add(Box::new(ui::Label::new(text(&line), h)?));
    }
    let agent_player_id = state.parts().belongs_to.get(id).0;
    let abilities = &state.parts().abilities.get(id).0;
    let r_ability = abilities.iter().find(|r| &r.ability == ability).unwrap();
    let is_enemy_agent = agent_player_id != state.player_id();
    let cooldown = r_ability.ability.base_cooldown();
    let text_cooldown = text(&format!("Cooldown: {}t", cooldown));
    layout.add(Box::new(ui::Label::new(text_cooldown, h)?));
    if !state::can_agent_use_ability(state, id, ability) {
        layout.add(Box::new(ui::Spacer::new_vertical(h / 2.0)));
        let s = if is_enemy_agent {
            "Can't be used: enemy agent.".into()
        } else if let ability::Status::Cooldown(n) = r_ability.status {
            format!("Can't be used: cooldown ({}t).", n)
        } else {
            "Can't be used: no attacks or jokers.".into()
        };
        let color = Color::new(0.5, 0.0, 0.0, 1.0);
        let label = ui::Label::new(text(&s), h)?.with_color(color);
        layout.add(Box::new(label));
    }
    layout.add(Box::new(ui::Spacer::new_vertical(h / 2.0)));
    let text_cancel = text("Click on an empty tile or the ability icon to cancel.");
    let color_cancel = Color::new(0.4, 0.4, 0.4, 1.0);
    let label_cancel_text = ui::Label::new(text_cancel, h)?.with_color(color_cancel);
    layout.add(Box::new(label_cancel_text));
    layout.stretch_to_self();
    let layout = utils::add_offsets_and_bg(layout, utils::OFFSET_SMALL)?;
    let layout = ui::pack(layout);
    let anchor = ui::Anchor(ui::HAnchor::Right, ui::VAnchor::Bottom);
    gui.add(&layout, anchor);
    Ok(layout)
}

fn make_gui() -> ZResult<ui::Gui<Message>> {
    let mut gui = ui::Gui::new();
    let h = line_heights().large;
    let icon = textures().icons.main_menu;
    let button = ui::Button::new(ui::Drawable::Texture(icon), h, gui.sender(), Message::Exit)?;
    let layout = ui::VLayout::from_widget(Box::new(button));
    let anchor = ui::Anchor(ui::HAnchor::Left, ui::VAnchor::Top);
    gui.add(&ui::pack(layout), anchor);
    Ok(gui)
}

#[derive(PartialEq, Copy, Clone)]
enum CommandOrigin {
    Player,
    Internal,
}

#[derive(Debug)]
pub struct Battle {
    gui: Gui<Message>,
    state: State,
    battle_type: scenario::BattleType,
    mode: SelectionMode,
    view: BattleView,
    selected_agent_id: Option<Id>,
    pathfinder: Pathfinder,
    block_timer: Option<Duration>,
    ai: Ai,
    panel_info: Option<ui::RcWidget>,
    panel_abilities: Option<ui::RcWidget>,
    panel_ability_description: Option<ui::RcWidget>,
    panel_end_turn: Option<ui::RcWidget>,
    sender: Sender<Option<BattleResult>>,
    confirmation_receiver_exit: Option<Receiver<screen::confirm::Message>>,
}

impl Battle {
    pub fn new(
        scenario: scenario::Scenario,
        battle_type: scenario::BattleType,
        prototypes: Prototypes,
        sender: Sender<Option<BattleResult>>,
    ) -> ZResult<Self> {
        let radius = scenario.map_radius;
        let mut view = BattleView::new(radius)?;
        let mut gui = make_gui()?;
        let mut actions = Vec::new();
        let state = State::new(prototypes, scenario, &mut |state, event, phase| {
            let action =
                visualize(state, &mut view, event, phase).expect("Can't visualize the event");
            actions.push(fork(action));
        });
        actions.push(make_action_create_map(&state, &view)?);
        view.add_action(action::Sequence::new(actions).boxed());
        let panel_end_turn = Some(build_panel_end_turn(&mut gui)?);
        Ok(Self {
            gui,
            view,
            mode: SelectionMode::Normal,
            state,
            battle_type,
            selected_agent_id: None,
            pathfinder: Pathfinder::new(radius),
            block_timer: None,
            ai: Ai::new(PlayerId(1), radius),
            panel_info: None,
            panel_abilities: None,
            panel_end_turn,
            panel_ability_description: None,
            sender,
            confirmation_receiver_exit: None,
        })
    }

    fn end_turn(&mut self) -> ZResult {
        utils::remove_widget(&mut self.gui, &mut self.panel_end_turn)?;
        self.deselect()?;
        let command = command::EndTurn.into();
        let actions = vec![
            self.do_command_inner(&command, CommandOrigin::Internal),
            self.do_ai(),
        ];
        self.add_actions(actions);
        Ok(())
    }

    fn do_ai(&mut self) -> Box<dyn Action> {
        trace!("AI: <");
        let mut actions = Vec::new();
        while let Some(command) = self.ai.command(&self.state) {
            trace!("AI: command = {:?}", command);
            actions.push(self.do_command_inner(&command, CommandOrigin::Internal));
            actions.push(action::Sleep::new(time_s(0.2)).boxed());
            if let command::Command::EndTurn(_) = command {
                break;
            }
        }
        trace!("AI: >");
        action::Sequence::new(actions).boxed()
    }

    fn use_ability(&mut self, ability: Ability) -> ZResult {
        let id = self.selected_agent_id.unwrap();
        if let SelectionMode::Ability(current_ability) = &self.mode {
            if current_ability == &ability {
                // Exit the ability mode if its icon was pressed again.
                return self.set_mode(id, SelectionMode::Normal);
            }
        }
        self.set_mode(id, SelectionMode::Ability(ability))
    }

    fn popup_confirm_exit(&mut self) -> ZResult<Box<dyn Screen>> {
        let (sender, receiver) = channel();
        self.confirmation_receiver_exit = Some(receiver);
        let message = match self.battle_type {
            scenario::BattleType::Skirmish => "Abandon this battle?",
            scenario::BattleType::CampaignNode => "Abandon the whole campaign?",
        };
        let popup = screen::Confirm::from_line(message, sender)?;
        Ok(Box::new(popup))
    }

    fn do_command_inner(
        &mut self,
        command: &command::Command,
        origin: CommandOrigin,
    ) -> Box<dyn Action> {
        trace!("do_command_inner: {:?}", command);
        self.view.messages_map_mut().clear();
        let mut actions = Vec::new();
        let state = &mut self.state;
        let view = &mut self.view;
        battle::execute(state, command, &mut |state, event, phase| {
            let action =
                visualize::visualize(state, view, event, phase).expect("Can't visualize the event");
            view.messages_map_mut().update(action.duration());
            actions.push(action);
            if origin != CommandOrigin::Player {
                let actual_sleep_duration = view.messages_map().total_duration().mul_f32(0.3);
                actions.push(action::Sleep::new(actual_sleep_duration).boxed());
                view.messages_map_mut().update(actual_sleep_duration);
            }
        })
        .expect("Can't execute command");
        action::Sequence::new(actions).boxed()
    }

    fn do_command(&mut self, command: &command::Command) {
        let action = self.do_command_inner(command, CommandOrigin::Player);
        self.add_action(action);
        self.view.messages_map_mut().clear();
    }

    fn add_actions(&mut self, actions: Vec<Box<dyn Action>>) {
        self.add_action(action::Sequence::new(actions).boxed());
    }

    fn add_action(&mut self, action: Box<dyn Action>) {
        self.block_timer = Some(action.duration());
        self.view.add_action(action);
    }

    fn deselect(&mut self) -> ZResult {
        self.remove_selected_highlighted_tiles_and_widgets()?;
        if self.selected_agent_id.is_some() {
            self.view.deselect();
        }
        self.selected_agent_id = None;
        self.mode = SelectionMode::Normal;
        Ok(())
    }

    fn remove_selected_highlighted_tiles_and_widgets(&mut self) -> ZResult {
        utils::remove_widget(&mut self.gui, &mut self.panel_info)?;
        utils::remove_widget(&mut self.gui, &mut self.panel_abilities)?;
        utils::remove_widget(&mut self.gui, &mut self.panel_ability_description)?;
        if self.selected_agent_id.is_some() {
            self.view.remove_highlights();
        }
        Ok(())
    }

    fn set_mode(&mut self, id: Id, mode: SelectionMode) -> ZResult {
        match mode {
            SelectionMode::Normal => self.deselect()?,
            SelectionMode::Ability(_) => self.remove_selected_highlighted_tiles_and_widgets()?,
        }
        if self.state.parts().agent.get_opt(id).is_none() {
            // This object is not an agent or dead.
            return Ok(());
        }
        self.selected_agent_id = Some(id);
        let state = &self.state;
        let gui = &mut self.gui;
        match mode {
            SelectionMode::Ability(ref ability) => {
                utils::remove_widget(gui, &mut self.panel_end_turn)?;
                self.panel_ability_description =
                    Some(build_panel_ability_description(gui, state, ability, id)?);
            }
            SelectionMode::Normal => {
                self.pathfinder.fill_map(state, id);
                if self.panel_end_turn.is_none() {
                    self.panel_end_turn = Some(build_panel_end_turn(gui)?);
                }
            }
        }
        self.panel_abilities = build_panel_agent_abilities(gui, state, id, &mode)?;
        self.panel_info = Some(build_panel_agent_info(gui, state, id)?);
        let map = self.pathfinder.map();
        self.view.set_mode(state, map, id, mode)?;
        self.mode = mode;
        Ok(())
    }

    fn handle_agent_click(&mut self, id: Id) -> ZResult {
        if self.state.parts().agent.get_opt(id).is_none() {
            // only agents can be selected
            return Ok(());
        }
        let other_agent_player_id = self.state.parts().belongs_to.get(id).0;
        if let Some(selected_agent_id) = self.selected_agent_id {
            let selected_agent_player_id = self.state.parts().belongs_to.get(selected_agent_id).0;
            if selected_agent_id == id {
                self.deselect()?;
                return Ok(());
            }
            if other_agent_player_id == selected_agent_player_id
                || other_agent_player_id == self.state.player_id()
            {
                self.set_mode(id, SelectionMode::Normal)?;
                return Ok(());
            }
            let command_attack = command::Attack {
                attacker_id: selected_agent_id,
                target_id: id,
            }
            .into();
            if check(&self.state, &command_attack).is_err() {
                return Ok(());
            }
            self.do_command(&command_attack);
            self.fill_map();
        } else {
            self.set_mode(id, SelectionMode::Normal)?;
        }
        Ok(())
    }

    fn fill_map(&mut self) {
        let selected_agent_id = self.selected_agent_id.unwrap();
        let parts = self.state.parts();
        if parts.agent.get_opt(selected_agent_id).is_some() {
            self.pathfinder.fill_map(&self.state, selected_agent_id);
        }
    }

    fn try_move_selected_agent(&mut self, pos: PosHex) {
        if let Some(id) = self.selected_agent_id {
            let path = match self.pathfinder.path(pos) {
                Some(path) => path,
                None => return,
            };
            assert_eq!(path.from(), self.state.parts().pos.get(id).0);
            let command_move = command::MoveTo { id, path }.into();
            if check(&self.state, &command_move).is_err() {
                return;
            }
            self.do_command(&command_move);
            self.fill_map();
        }
    }

    fn handle_click(&mut self, point: Vec2) -> ZResult {
        let pos = geom::point_to_hex(self.view.tile_size(), point);
        self.gui.click(point);
        if self.block_timer.is_some() {
            return Ok(());
        }
        if let SelectionMode::Ability(ability) = self.mode {
            let id = self.selected_agent_id.unwrap();
            let command = command::UseAbility { id, pos, ability }.into();
            if check(&self.state, &command).is_ok() {
                self.do_command(&command);
            } else {
                self.view.message(pos, "cancelled")?;
            }
            self.set_mode(id, SelectionMode::Normal)?;
        } else if self.state.map().is_inboard(pos) {
            if let Some(id) = state::agent_id_at_opt(&self.state, pos) {
                self.handle_agent_click(id)?;
            } else {
                self.try_move_selected_agent(pos);
            }
        }
        self.view.messages_map_mut().clear();
        Ok(())
    }

    fn update_block_timer(&mut self, dtime: Duration) -> ZResult {
        if let Some(time) = self.block_timer {
            if time < dtime {
                self.block_timer = None;
                if let Some(id) = self.selected_agent_id {
                    self.set_mode(id, SelectionMode::Normal)?;
                }
            }
        }
        if let Some(ref mut time) = self.block_timer {
            *time -= dtime;
        }
        Ok(())
    }

    fn send_battle_result(&self, result: Option<BattleResult>) {
        let err_msg = "Can't report back a battle's result";
        self.sender.send(result).expect(err_msg);
    }
}

impl Screen for Battle {
    fn update(&mut self, dtime: Duration) -> ZResult<StackCommand> {
        if screen::confirm::try_receive_yes(&self.confirmation_receiver_exit) {
            self.confirmation_receiver_exit = None;
            self.send_battle_result(None);
            return Ok(StackCommand::Pop);
        }
        self.view.tick(dtime);
        self.update_block_timer(dtime)?;
        if self.block_timer.is_none() {
            if let Some(result) = self.state.battle_result().clone() {
                self.send_battle_result(Some(result));
                return Ok(StackCommand::Pop);
            }
            if self.panel_end_turn.is_none() && self.mode == SelectionMode::Normal {
                self.panel_end_turn = Some(build_panel_end_turn(&mut self.gui)?);
            }
        }
        Ok(StackCommand::None)
    }

    fn draw(&self) -> ZResult {
        self.view.draw()?;
        self.gui.draw();
        Ok(())
    }

    fn click(&mut self, pos: Vec2) -> ZResult<StackCommand> {
        let message = self.gui.click(pos);
        info!("Battle: click: pos={:?}, message={:?}", pos, message);
        match message {
            Some(Message::Exit) => {
                return Ok(StackCommand::PushPopup(self.popup_confirm_exit()?));
            }
            Some(Message::EndTurn) => {
                assert!(self.block_timer.is_none());
                self.end_turn()?;
            }
            Some(Message::Ability(ability)) => self.use_ability(ability)?,
            Some(Message::PassiveAbilityInfo(ability)) => {
                let title = &ability.title();
                let description = &ability.description();
                let popup = screen::GeneralInfo::new(title, description)?;
                return Ok(StackCommand::PushPopup(Box::new(popup)));
            }
            Some(Message::LastingEffectInfo(effect)) => {
                let title = &effect.title();
                let description = &effect.description();
                let popup = screen::GeneralInfo::new(title, description)?;
                return Ok(StackCommand::PushPopup(Box::new(popup)));
            }
            None => self.handle_click(pos)?,
        }
        Ok(StackCommand::None)
    }

    fn resize(&mut self, aspect_ratio: f32) {
        self.gui.resize_if_needed(aspect_ratio);
    }

    fn move_mouse(&mut self, point: Vec2) -> ZResult {
        let pos = geom::point_to_hex(self.view.tile_size(), point);
        if self.state.map().is_inboard(pos) {
            self.view.show_current_tile_marker(pos);
        } else {
            self.view.hide_current_tile_marker();
        }
        self.gui.move_mouse(point);
        Ok(())
    }
}
