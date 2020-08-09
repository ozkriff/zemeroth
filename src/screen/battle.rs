use std::{
    sync::mpsc::{channel, Receiver, Sender},
    time::Duration,
};

use gwg::{
    graphics::{self, Font, Point2, Text},
    Context,
};
use log::{info, trace};
use ui::{self, Gui};
use zscene::{action, Action, Boxed};

use crate::{
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
            visualize::{fork, visualize},
        },
        Screen, StackCommand,
    },
    utils::{self, default_font, line_heights, time_s},
    ZResult,
};

// TODO: Don't use graphics::Image::new in this file! Pre-load all images into View.

mod view;
mod visualize;

const FONT_SIZE: f32 = utils::font_size();

#[derive(Clone, Debug)]
enum Message {
    Exit,
    EndTurn,
    Ability(Ability),
    PassiveAbilityInfo(PassiveAbility),
}

fn build_panel_agent_info(
    context: &mut Context,
    font: Font,
    gui: &mut Gui<Message>,
    state: &State,
    id: Id,
) -> ZResult<ui::RcWidget> {
    let parts = state.parts();
    let st = parts.strength.get(id);
    let meta = parts.meta.get(id);
    let a = parts.agent.get(id);
    let mut layout = ui::VLayout::new();
    let h = line_heights().normal;
    {
        let label = |context: &mut Context, text: &str| -> ZResult<Box<dyn ui::Widget>> {
            let text = Box::new(Text::new((text, font, FONT_SIZE)));
            Ok(Box::new(ui::Label::new(context, text, h)?))
        };
        let mut add = |w| layout.add(w);
        let mut line = |context: &mut Context, text: &str| -> ZResult {
            add(label(context, text)?);
            Ok(())
        };
        line(context, &format!("<{}>", meta.name.0))?;
        line(
            context,
            &format!("strength: {}/{}", st.strength.0, st.base_strength.0),
        )?;
        if let Some(armor) = parts.armor.get_opt(id) {
            let armor = armor.armor.0;
            if armor != 0 {
                line(context, &format!("armor: {}", armor))?;
            }
        }
        if let Some(blocker) = parts.blocker.get_opt(id) {
            line(context, &format!("weight: {}", blocker.weight))?;
        }
        if a.jokers.0 != 0 || a.base_jokers.0 != 0 {
            line(
                context,
                &format!("jokers: {}/{}", a.jokers.0, a.base_jokers.0),
            )?;
        }
        line(
            context,
            &format!("attacks: {}/{}", a.attacks.0, a.base_attacks.0),
        )?;
        if a.reactive_attacks.0 != 0 {
            line(
                context,
                &format!("reactive attacks: {}", a.reactive_attacks.0),
            )?;
        }
        line(context, &format!("moves: {}/{}", a.moves.0, a.base_moves.0))?;
        if a.attack_distance.0 != 1 {
            line(
                context,
                &format!("attack distance: {}", a.attack_distance.0),
            )?;
        }
        line(
            context,
            &format!("attack strength: {}", a.attack_strength.0),
        )?;
        line(
            context,
            &format!("attack accuracy: {}", a.attack_accuracy.0),
        )?;
        if a.attack_break.0 > 0 {
            line(context, &format!("armor break: {}", a.attack_break.0))?;
        }
        if a.dodge.0 > 0 {
            line(context, &format!("dodge: {}", a.dodge.0))?;
        }
        line(context, &format!("move points: {}", a.move_points.0))?;
        if let Some(abilities) = parts.passive_abilities.get_opt(id) {
            if !abilities.0.is_empty() {
                line(context, "<passive abilities>:")?;
                for &ability in &abilities.0 {
                    let text = format!("'{}'", ability.title());
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
        // TODO: add (i) buttons for effects
        if let Some(effects) = parts.effects.get_opt(id) {
            if !effects.0.is_empty() {
                add(label(context, "<effects>:")?);
                for effect in &effects.0 {
                    let s = effect.effect.to_str();
                    let text = match effect.duration {
                        effect::Duration::Forever => format!("'{}'", s),
                        effect::Duration::Rounds(n) => format!("'{}' ({})", s, n),
                    };
                    add(label(context, &text)?);
                }
            }
        }
    }
    let packed_layout = ui::pack(utils::wrap_widget_and_add_bg(context, Box::new(layout))?);
    let anchor = ui::Anchor(ui::HAnchor::Left, ui::VAnchor::Bottom);
    gui.add(&packed_layout, anchor);
    Ok(packed_layout)
}

fn build_panel_agent_abilities(
    context: &mut Context,
    _view: &BattleView, // TODO: use this for cloning stored icon images
    font: Font,
    gui: &mut Gui<Message>,
    state: &State,
    id: Id,
    mode: &SelectionMode,
) -> ZResult<Option<ui::RcWidget>> {
    let parts = state.parts();
    let abilities = match parts.abilities.get_opt(id) {
        Some(abilities) => &abilities.0,
        None => return Ok(None),
    };
    let mut layout = ui::VLayout::new();
    let h = line_heights().large;
    for ability in abilities {
        let image_path = match ability.ability {
            // TODO: load all the images only once. Store them in some struct and only clone them here.
            // TODO: Move into view::Images!
            Ability::Club => "/icon_ability_club.png",
            Ability::Knockback(_) => "/icon_ability_knockback.png",
            Ability::Jump(_) => "/icon_ability_jump.png",
            Ability::Dash => "/icon_ability_dash.png",
            Ability::Rage(_) => "/icon_ability_rage.png",
            Ability::Heal(_) => "/icon_ability_heal.png",
            Ability::BombPush(_) => "/icon_ability_bomb_push.png",
            Ability::Bomb(_) => "/icon_ability_bomb.png",
            Ability::BombFire(_) => "/icon_ability_bomb_fire.png",
            Ability::BombPoison(_) => "/icon_ability_bomb_poison.png",
            Ability::BombDemonic(_) => "/icon_ability_bomb_demonic.png",
            Ability::Summon => "/icon_ability_summon.png",
            Ability::Bloodlust => "/icon_ability_bloodlust.png",
            ref ability => panic!("No icon for {:?}", ability),
        };
        let image = graphics::Image::new(context, image_path)?;
        let msg = Message::Ability(ability.ability.clone());
        let mut button = ui::Button::new(context, Box::new(image), h, gui.sender(), msg)?;
        if !state::can_agent_use_ability(state, id, &ability.ability) {
            button.set_active(false);
        }
        if let SelectionMode::Ability(selected_ability) = mode {
            if selected_ability == &ability.ability {
                button.set_color([0.0, 0.0, 0.9, 1.0].into());
            }
        }
        if let ability::Status::Cooldown(n) = ability.status {
            let mut layers = ui::LayersLayout::new();
            layers.add(Box::new(button));
            let text = Box::new(Text::new((format!(" ({})", n).as_str(), font, FONT_SIZE)));
            let label = ui::Label::new(context, text, h / 2.0)?;
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

fn build_panel_end_turn(context: &mut Context, gui: &mut Gui<Message>) -> ZResult<ui::RcWidget> {
    let h = line_heights().large;
    let icon = Box::new(graphics::Image::new(context, "/icon_end_turn.png")?);
    let button = ui::Button::new(context, icon, h, gui.sender(), Message::EndTurn)?;
    let layout = ui::VLayout::from_widget(Box::new(button));
    let anchor = ui::Anchor(ui::HAnchor::Right, ui::VAnchor::Bottom);
    let packed_layout = ui::pack(layout);
    gui.add(&packed_layout, anchor);
    Ok(packed_layout)
}

fn build_panel_ability_description(
    context: &mut Context,
    font: Font,
    gui: &mut Gui<Message>,
    state: &State,
    ability: &Ability,
    id: Id,
) -> ZResult<ui::RcWidget> {
    let text = |s: &str| Box::new(Text::new((s, font, FONT_SIZE)));
    let h = line_heights().normal;
    let mut layout = ui::VLayout::new();
    let text_title = text(&format!("<Ability '{}'>", ability.title()));
    layout.add(Box::new(ui::Label::new(context, text_title, h)?));
    for line in ability.extended_description() {
        layout.add(Box::new(ui::Label::new(context, text(&line), h)?));
    }
    let agent_player_id = state.parts().belongs_to.get(id).0;
    let abilities = &state.parts().abilities.get(id).0;
    let r_ability = abilities.iter().find(|r| &r.ability == ability).unwrap();
    let is_enemy_agent = agent_player_id != state.player_id();
    let text_cooldown = text(&format!("Cooldown: {}", r_ability.base_cooldown));
    layout.add(Box::new(ui::Label::new(context, text_cooldown, h)?));
    if !state::can_agent_use_ability(state, id, ability) {
        layout.add(Box::new(ui::Spacer::new_vertical(h / 2.0)));
        let s = if is_enemy_agent {
            "Can't be used: enemy agent.".into()
        } else if let ability::Status::Cooldown(n) = r_ability.status {
            format!("Can't be used: cooldown ({} turns).", n)
        } else {
            "Can't be used: no attacks or jokers.".into()
        };
        layout.add(Box::new(ui::Label::new(context, text(&s), h)?));
    }
    layout.add(Box::new(ui::Spacer::new_vertical(h / 2.0)));
    let text_cancel = text("Click on an empty tile to cancel.");
    layout.add(Box::new(ui::Label::new(context, text_cancel, h)?));
    let anchor = ui::Anchor(ui::HAnchor::Right, ui::VAnchor::Bottom);
    let layout = utils::wrap_widget_and_add_bg(context, Box::new(layout))?;
    let packed_layout = ui::pack(layout);
    gui.add(&packed_layout, anchor);
    Ok(packed_layout)
}

fn make_gui(context: &mut Context) -> ZResult<ui::Gui<Message>> {
    let mut gui = ui::Gui::new(context);
    let h = line_heights().large;
    {
        let icon = Box::new(graphics::Image::new(context, "/icon_menu.png")?);
        let button = ui::Button::new(context, icon, h, gui.sender(), Message::Exit)?;
        let layout = ui::VLayout::from_widget(Box::new(button));
        let anchor = ui::Anchor(ui::HAnchor::Left, ui::VAnchor::Top);
        gui.add(&ui::pack(layout), anchor);
    }
    Ok(gui)
}

#[derive(Debug)]
pub struct Battle {
    font: graphics::Font,
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
        context: &mut Context,
        scenario: scenario::Scenario,
        battle_type: scenario::BattleType,
        prototypes: Prototypes,
        sender: Sender<Option<BattleResult>>,
    ) -> ZResult<Self> {
        let font = default_font(context);
        let mut gui = make_gui(context)?;
        let radius = scenario.map_radius;
        let mut view = BattleView::new(radius, context)?;
        let mut actions = Vec::new();
        let state = State::new(prototypes, scenario, &mut |state, event, phase| {
            let action = visualize(state, &mut view, context, event, phase)
                .expect("Can't visualize the event");
            actions.push(fork(action));
        });
        actions.push(make_action_create_map(&state, context, &view)?);
        view.add_action(action::Sequence::new(actions).boxed());
        let panel_end_turn = Some(build_panel_end_turn(context, &mut gui)?);
        Ok(Self {
            gui,
            font,
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

    fn end_turn(&mut self, context: &mut Context) -> ZResult {
        utils::remove_widget(&mut self.gui, &mut self.panel_end_turn)?;
        self.deselect()?;
        let command = command::EndTurn.into();
        let mut actions = Vec::new();
        actions.push(self.do_command_inner(context, &command));
        actions.push(self.do_ai(context));
        self.add_actions(actions);
        Ok(())
    }

    fn do_ai(&mut self, context: &mut Context) -> Box<dyn Action> {
        trace!("AI: <");
        let mut actions = Vec::new();
        while let Some(command) = self.ai.command(&self.state) {
            trace!("AI: command = {:?}", command);
            actions.push(self.do_command_inner(context, &command));
            actions.push(action::Sleep::new(time_s(0.3)).boxed());
            if let command::Command::EndTurn(_) = command {
                break;
            }
        }
        trace!("AI: >");
        action::Sequence::new(actions).boxed()
    }

    fn use_ability(&mut self, context: &mut Context, ability: Ability) -> ZResult {
        let id = self.selected_agent_id.unwrap();
        if let SelectionMode::Ability(current_ability) = &self.mode {
            if current_ability == &ability {
                // Exit the ability mode if its icon was pressed again.
                return self.set_mode(context, id, SelectionMode::Normal);
            }
        }
        self.set_mode(context, id, SelectionMode::Ability(ability))
    }

    fn popup_confirm_exit(&mut self, context: &mut Context) -> ZResult<Box<dyn Screen>> {
        let (sender, receiver) = channel();
        self.confirmation_receiver_exit = Some(receiver);
        let message = match self.battle_type {
            scenario::BattleType::Skirmish => "Abandon this battle?",
            scenario::BattleType::CampaignNode => "Abandon the whole campaign?",
        };
        let popup = screen::Confirm::from_line(context, message, sender)?;
        Ok(Box::new(popup))
    }

    fn popup_passive_ability_info(
        &mut self,
        context: &mut Context,
        ability: PassiveAbility,
    ) -> ZResult<Box<dyn Screen>> {
        let ability = screen::ability_info::ActiveOrPassiveAbility::Passive(ability);
        let popup = screen::AbilityInfo::new(context, ability)?;
        Ok(Box::new(popup))
    }

    fn do_command_inner(
        &mut self,
        context: &mut Context,
        command: &command::Command,
    ) -> Box<dyn Action> {
        trace!("do_command_inner: {:?}", command);
        let mut actions = Vec::new();
        let state = &mut self.state;
        let view = &mut self.view;
        battle::execute(state, command, &mut |state, event, phase| {
            let action = visualize::visualize(state, view, context, event, phase)
                .expect("Can't visualize the event");
            actions.push(action);
        })
        .expect("Can't execute command");
        action::Sequence::new(actions).boxed()
    }

    fn do_command(&mut self, context: &mut Context, command: &command::Command) {
        let action = self.do_command_inner(context, command);
        self.add_action(action);
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

    fn set_mode(&mut self, context: &mut Context, id: Id, mode: SelectionMode) -> ZResult {
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
                self.panel_ability_description = Some(build_panel_ability_description(
                    context, self.font, gui, state, ability, id,
                )?);
            }
            SelectionMode::Normal => {
                self.pathfinder.fill_map(state, id);
                if self.panel_end_turn.is_none() {
                    self.panel_end_turn = Some(build_panel_end_turn(context, gui)?);
                }
            }
        }
        self.panel_abilities =
            build_panel_agent_abilities(context, &self.view, self.font, gui, state, id, &mode)?;
        self.panel_info = Some(build_panel_agent_info(context, self.font, gui, state, id)?);
        let map = self.pathfinder.map();
        self.view.set_mode(state, context, map, id, &mode)?;
        self.mode = mode;
        Ok(())
    }

    fn handle_agent_click(&mut self, context: &mut Context, id: Id) -> ZResult {
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
                self.set_mode(context, id, SelectionMode::Normal)?;
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
            self.do_command(context, &command_attack);
            self.fill_map();
        } else {
            self.set_mode(context, id, SelectionMode::Normal)?;
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

    fn try_move_selected_agent(&mut self, context: &mut Context, pos: PosHex) {
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
            self.do_command(context, &command_move);
            self.fill_map();
        }
    }

    fn handle_click(&mut self, context: &mut Context, point: Point2) -> ZResult {
        let pos = geom::point_to_hex(self.view.tile_size(), point);
        self.gui.click(point);
        if self.block_timer.is_some() {
            return Ok(());
        }
        if let SelectionMode::Ability(ability) = self.mode.clone() {
            let id = self.selected_agent_id.unwrap();
            let command = command::UseAbility { id, pos, ability }.into();
            if check(&self.state, &command).is_ok() {
                self.do_command(context, &command);
            } else {
                self.view.message(context, pos, "cancelled")?;
            }
            self.set_mode(context, id, SelectionMode::Normal)?;
        } else if self.state.map().is_inboard(pos) {
            if let Some(id) = state::agent_id_at_opt(&self.state, pos) {
                self.handle_agent_click(context, id)?;
            } else {
                self.try_move_selected_agent(context, pos);
            }
        }
        Ok(())
    }

    fn update_block_timer(&mut self, context: &mut Context, dtime: Duration) -> ZResult {
        if let Some(time) = self.block_timer {
            if time < dtime {
                self.block_timer = None;
                if let Some(id) = self.selected_agent_id {
                    self.set_mode(context, id, SelectionMode::Normal)?;
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
    fn update(&mut self, context: &mut Context, dtime: Duration) -> ZResult<StackCommand> {
        if screen::confirm::try_receive_yes(&self.confirmation_receiver_exit) {
            self.confirmation_receiver_exit = None;
            self.send_battle_result(None);
            return Ok(StackCommand::Pop);
        }
        self.view.tick(dtime);
        self.update_block_timer(context, dtime)?;
        if self.block_timer.is_none() {
            if let Some(result) = self.state.battle_result().clone() {
                self.send_battle_result(Some(result));
                return Ok(StackCommand::Pop);
            }
            if self.panel_end_turn.is_none() && self.mode == SelectionMode::Normal {
                self.panel_end_turn = Some(build_panel_end_turn(context, &mut self.gui)?);
            }
        }
        Ok(StackCommand::None)
    }

    fn draw(&self, context: &mut Context) -> ZResult {
        self.view.draw(context)?;
        self.gui.draw(context)?;
        Ok(())
    }

    fn resize(&mut self, aspect_ratio: f32) {
        self.gui.resize(aspect_ratio);
    }

    fn click(&mut self, context: &mut Context, pos: Point2) -> ZResult<StackCommand> {
        let message = self.gui.click(pos);
        info!("Battle: click: pos={:?}, message={:?}", pos, message);
        match message {
            Some(Message::Exit) => {
                return Ok(StackCommand::PushPopup(self.popup_confirm_exit(context)?));
            }
            Some(Message::EndTurn) => {
                assert!(self.block_timer.is_none());
                self.end_turn(context)?;
            }
            Some(Message::Ability(ability)) => self.use_ability(context, ability)?,
            Some(Message::PassiveAbilityInfo(ability)) => {
                let popup = self.popup_passive_ability_info(context, ability)?;
                return Ok(StackCommand::PushPopup(popup));
            }
            None => self.handle_click(context, pos)?,
        }
        Ok(StackCommand::None)
    }

    fn move_mouse(&mut self, _context: &mut Context, point: Point2) -> ZResult {
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
