use std::{path::Path, sync::mpsc::Sender, time::Duration};

use ggez::{
    graphics::{self, Font, Text},
    nalgebra::Point2,
    Context,
};
use log::{debug, info};
use scene::{action, Action, Boxed};
use ui::{self, Gui};

use crate::{
    core::{
        battle::{
            self, ability,
            ability::Ability,
            ai::Ai,
            check, command,
            component::Prototypes,
            effect,
            movement::Pathfinder,
            scenario,
            state::{self, BattleResult},
            ObjId, PlayerId, State,
        },
        map::PosHex,
    },
    geom,
    screen::{
        battle::{
            view::{make_action_create_map, BattleView, SelectionMode},
            visualize::{fork, visualize},
        },
        Screen, Transition,
    },
    utils::{self, default_font, line_heights, time_s},
    ZResult,
};

mod view;
mod visualize;

const FONT_SIZE: f32 = utils::font_size();

#[derive(Clone, Debug)]
enum Message {
    Exit,
    Deselect,
    EndTurn,
    Ability(Ability),
}

fn build_panel_agent_info(
    context: &mut Context,
    font: Font,
    gui: &mut Gui<Message>,
    state: &State,
    id: ObjId,
) -> ZResult<ui::RcWidget> {
    let parts = state.parts();
    let st = parts.strength.get(id);
    let meta = parts.meta.get(id);
    let a = parts.agent.get(id);
    let mut layout = ui::VLayout::new();
    let h = line_heights().normal;
    {
        let mut line = |text: &str| -> ZResult {
            let text = Box::new(Text::new((text, font, FONT_SIZE)));
            let button = ui::Label::new(context, text, h)?;
            layout.add(Box::new(button));
            Ok(())
        };
        line(&format!("<{}>", meta.name.0))?;
        line(&format!(
            "strength: {}/{}",
            st.strength.0, st.base_strength.0
        ))?;
        if let Some(armor) = parts.armor.get_opt(id) {
            let armor = armor.armor.0;
            if armor != 0 {
                line(&format!("armor: {}", armor))?;
            }
        }
        if a.jokers.0 != 0 || a.base_jokers.0 != 0 {
            line(&format!("jokers: {}/{}", a.jokers.0, a.base_jokers.0))?;
        }
        line(&format!("attacks: {}/{}", a.attacks.0, a.base_attacks.0))?;
        line(&format!("moves: {}/{}", a.moves.0, a.base_moves.0))?;
        if a.reactive_attacks.0 != 0 {
            line(&format!("reactive attacks: {}", a.reactive_attacks.0))?;
        }
        if a.attack_distance.0 != 1 {
            line(&format!("attack distance: {}", a.attack_distance.0))?;
        }
        line(&format!("attack strength: {}", a.attack_strength.0))?;
        line(&format!("attack accuracy: {}", a.attack_accuracy.0))?;
        if a.dodge.0 > 0 {
            line(&format!("dodge: {}", a.dodge.0))?;
        }
        line(&format!("move points: {}", a.move_points.0))?;
        if let Some(abilities) = parts.passive_abilities.get_opt(id) {
            if !abilities.0.is_empty() {
                line("<passive abilities>:")?;
                for ability in &abilities.0 {
                    line(&format!("'{}'", ability.to_string()))?;
                }
            }
        }
        if let Some(abilities) = parts.abilities.get_opt(id) {
            if !abilities.0.is_empty() {
                line("<abilities>:")?;
                for ability in &abilities.0 {
                    let s = ability.ability.to_string();
                    line(&format!("'{}'", s))?;
                }
            }
        }
        if let Some(effects) = parts.effects.get_opt(id) {
            if !effects.0.is_empty() {
                line("<effects>:")?;
                for effect in &effects.0 {
                    let s = effect.effect.to_str();
                    match effect.duration {
                        effect::Duration::Forever => line(&format!("'{}'", s))?,
                        effect::Duration::Rounds(n) => line(&format!("'{}' ({})", s, n))?,
                    }
                }
            }
        }
    }
    let layout = ui::pack(layout);
    let anchor = ui::Anchor(ui::HAnchor::Left, ui::VAnchor::Bottom);
    gui.add(&layout, anchor);
    Ok(layout)
}

fn build_panel_agent_abilities(
    context: &mut Context,
    font: Font,
    gui: &mut Gui<Message>,
    state: &State,
    id: ObjId,
) -> ZResult<Option<ui::RcWidget>> {
    let parts = state.parts();
    let abilities = match parts.abilities.get_opt(id) {
        Some(abilities) => &abilities.0,
        None => return Ok(None),
    };
    let agent = parts.agent.get(id);
    if agent.attacks <= battle::Attacks(0) && agent.jokers <= battle::Jokers(0) {
        return Ok(None);
    }
    let mut layout = ui::VLayout::new();
    let h = line_heights().normal;
    for ability in abilities {
        let text = match ability.status {
            ability::Status::Ready => format!("[{}]", ability.ability.to_string()),
            ability::Status::Cooldown(n) => format!("[{} ({})]", ability.ability.to_string(), n),
        };
        let text = Box::new(Text::new((text.as_str(), font, FONT_SIZE)));
        let msg = Message::Ability(ability.ability.clone());
        let button = ui::Button::new(context, text, h, gui.sender(), msg)?;
        layout.add(Box::new(button));
    }
    let anchor = ui::Anchor(ui::HAnchor::Right, ui::VAnchor::Middle);
    let layout = ui::pack(layout);
    gui.add(&layout, anchor);
    Ok(Some(layout))
}

fn make_gui(context: &mut Context, font: Font) -> ZResult<ui::Gui<Message>> {
    let mut gui = ui::Gui::new(context);
    {
        let text = Box::new(Text::new(("[deselect]", font, FONT_SIZE)));
        let button = ui::Button::new(context, text, 0.1, gui.sender(), Message::Deselect)?;
        let mut layout = ui::VLayout::new();
        layout.add(Box::new(button));
        let anchor = ui::Anchor(ui::HAnchor::Right, ui::VAnchor::Top);
        gui.add(&ui::pack(layout), anchor);
    }
    {
        let text = Box::new(Text::new(("[end turn]", font, FONT_SIZE)));
        let button = ui::Button::new(context, text, 0.1, gui.sender(), Message::EndTurn)?;
        let mut layout = ui::VLayout::new();
        layout.add(Box::new(button));
        let anchor = ui::Anchor(ui::HAnchor::Right, ui::VAnchor::Bottom);
        gui.add(&ui::pack(layout), anchor);
    }
    {
        let text = Box::new(Text::new(("[exit]", font, FONT_SIZE)));
        let button = ui::Button::new(context, text, 0.1, gui.sender(), Message::Exit)?;
        let mut layout = ui::VLayout::new();
        layout.add(Box::new(button));
        let anchor = ui::Anchor(ui::HAnchor::Left, ui::VAnchor::Top);
        gui.add(&ui::pack(layout), anchor);
    }
    Ok(gui)
}

pub fn load_prototypes(context: &mut Context, path: &Path) -> ZResult<Prototypes> {
    let buf = utils::read_file_to_string(context, path)?;
    let prototypes = Prototypes::from_string(&buf);
    debug!("{:?}", prototypes);
    Ok(prototypes)
}

#[derive(Debug)]
pub struct Battle {
    font: graphics::Font,
    gui: Gui<Message>,
    state: State,
    mode: SelectionMode,
    view: BattleView,
    selected_agent_id: Option<ObjId>,
    pathfinder: Pathfinder,
    block_timer: Option<Duration>,
    ai: Ai,
    panel_info: Option<ui::RcWidget>,
    panel_abilities: Option<ui::RcWidget>,
    sender: Sender<BattleResult>,
}

impl Battle {
    pub fn new(
        context: &mut Context,
        scenario: scenario::Scenario,
        sender: Sender<BattleResult>,
    ) -> ZResult<Self> {
        let font = default_font(context);
        let gui = make_gui(context, font)?;
        let prototypes = load_prototypes(context, Path::new("/objects.ron"))?;
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
        Ok(Self {
            gui,
            font,
            view,
            mode: SelectionMode::Normal,
            state,
            selected_agent_id: None,
            pathfinder: Pathfinder::new(radius),
            block_timer: None,
            ai: Ai::new(PlayerId(1), radius),
            panel_info: None,
            panel_abilities: None,
            sender,
        })
    }

    fn end_turn(&mut self, context: &mut Context) -> ZResult {
        if self.block_timer.is_some() {
            return Ok(());
        }
        self.deselect()?;
        let command = command::EndTurn.into();
        let mut actions = Vec::new();
        actions.push(self.do_command_inner(context, &command));
        actions.push(self.do_ai(context));
        self.add_actions(actions);
        Ok(())
    }

    fn do_ai(&mut self, context: &mut Context) -> Box<dyn Action> {
        debug!("AI: <");
        let mut actions = Vec::new();
        while let Some(command) = self.ai.command(&self.state) {
            debug!("AI: command = {:?}", command);
            actions.push(self.do_command_inner(context, &command));
            actions.push(action::Sleep::new(time_s(0.3)).boxed());
            if let command::Command::EndTurn(_) = command {
                break;
            }
        }
        debug!("AI: >");
        action::Sequence::new(actions).boxed()
    }

    fn use_ability(&mut self, context: &mut Context, ability: Ability) -> ZResult {
        // TODO: code duplication (see check.rs and event.rs)
        let id = self.selected_agent_id.unwrap(); // TODO: Extract to some specific method
        let agent_player_id = self.state.parts().belongs_to.get(id).0;
        if agent_player_id != self.state.player_id() {
            debug!("Can't command enemy agent");
            return Ok(()); // TODO: fix error handling
        }
        for rechargeable in &self.state.parts().abilities.get(id).0 {
            if rechargeable.ability == ability && rechargeable.status != ability::Status::Ready {
                debug!("ability isn't ready yet");
                return Ok(());
            }
        }
        self.set_mode(context, id, SelectionMode::Ability(ability))
    }

    fn do_command_inner(
        &mut self,
        context: &mut Context,
        command: &command::Command,
    ) -> Box<dyn Action> {
        debug!("do_command_inner: {:?}", command);
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
        if let Some(panel) = self.panel_info.take() {
            self.gui.remove(&panel)?;
        }
        if let Some(panel) = self.panel_abilities.take() {
            self.gui.remove(&panel)?;
        }
        if self.selected_agent_id.is_some() {
            self.view.deselect();
        }
        self.selected_agent_id = None;
        self.mode = SelectionMode::Normal;
        Ok(())
    }

    fn set_mode(&mut self, context: &mut Context, id: ObjId, mode: SelectionMode) -> ZResult {
        self.deselect()?;
        if self.state.parts().agent.get_opt(id).is_none() {
            // This object is not an agent or dead.
            return Ok(());
        }
        self.selected_agent_id = Some(id);
        let state = &self.state;
        let gui = &mut self.gui;
        match mode {
            SelectionMode::Ability(_) => {
                // TODO: Update the GUI here: explain how to use or cancel the ability.
                // 'Select target tile'
            }
            SelectionMode::Normal => {
                self.pathfinder.fill_map(state, id);
                self.panel_info = Some(build_panel_agent_info(context, self.font, gui, state, id)?);
                self.panel_abilities =
                    build_panel_agent_abilities(context, self.font, gui, state, id)?;
            }
        }
        let map = self.pathfinder.map();
        self.view.set_mode(state, context, map, id, &mode)?;
        self.mode = mode;
        Ok(())
    }

    fn handle_agent_click(&mut self, context: &mut Context, id: ObjId) -> ZResult {
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

    fn handle_event_click(&mut self, context: &mut Context, point: Point2<f32>) -> ZResult {
        let pos = geom::point_to_hex(self.view.tile_size(), point);
        self.gui.click(point);
        if self.block_timer.is_some() {
            return Ok(());
        }
        if self.state.map().is_inboard(pos) {
            if let SelectionMode::Ability(ability) = self.mode.clone() {
                let id = self.selected_agent_id.unwrap();
                let command = command::UseAbility { id, pos, ability }.into();
                if check(&self.state, &command).is_ok() {
                    self.do_command(context, &command);
                } else {
                    self.view.message(context, pos, "cancelled")?;
                }
                self.set_mode(context, id, SelectionMode::Normal)?;
            } else if let Some(id) = state::agent_id_at_opt(&self.state, pos) {
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
}

impl Screen for Battle {
    fn update(&mut self, context: &mut Context, dtime: Duration) -> ZResult<Transition> {
        self.view.tick(dtime);
        self.update_block_timer(context, dtime)?;
        if self.block_timer.is_none() {
            if let Some(result) = self.state.battle_result().clone() {
                self.sender
                    .send(result)
                    .expect("Can't report back a battle's result");
                return Ok(Transition::Pop);
            }
        }
        Ok(Transition::None)
    }

    fn draw(&self, context: &mut Context) -> ZResult {
        self.view.draw(context)?;
        self.gui.draw(context)?;
        Ok(())
    }

    fn resize(&mut self, aspect_ratio: f32) {
        self.gui.resize(aspect_ratio);
    }

    fn click(&mut self, context: &mut Context, pos: Point2<f32>) -> ZResult<Transition> {
        let message = self.gui.click(pos);
        info!("Battle: click: pos={:?}, message={:?}", pos, message);
        match message {
            Some(Message::Exit) => return Ok(Transition::Pop),
            Some(Message::EndTurn) => self.end_turn(context)?,
            Some(Message::Deselect) => self.deselect()?,
            Some(Message::Ability(ability)) => self.use_ability(context, ability)?,
            None => self.handle_event_click(context, pos)?,
        }
        Ok(Transition::None)
    }

    fn move_mouse(&mut self, _context: &mut Context, point: Point2<f32>) -> ZResult {
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
