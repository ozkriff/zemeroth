use std::io::Read;
use std::time::Duration;

use ggez::graphics::{self, Font, Point2, Text};
use ggez::Context;
use ron;
use scene::{action, Action, Boxed};
use ui::{self, Gui};

use ai::Ai;
use battle_view::{make_action_create_map, BattleView, SelectionMode};
use core::ability::Ability;
use core::effect;
use core::map::PosHex;
use core::movement::Pathfinder;
use core::{self, ability, check, command, execute, state};
use core::{ObjId, PlayerId, State};
use geom;
use screen::{Screen, Transition};
use visualize;
use ZResult;

#[derive(Clone, Copy, Debug)]
enum Message {
    Exit,
    Deselect,
    EndTurn,
    Ability(Ability),
}

fn line_height() -> f32 {
    0.08
}

// TODO: reverse?
fn build_panel_unit_info(
    context: &mut Context,
    font: &Font,
    gui: &mut Gui<Message>,
    state: &State,
    id: ObjId,
) -> ZResult<ui::RcWidget> {
    let parts = state.parts();
    let st = parts.strength.get(id);
    let meta = parts.meta.get(id);
    let a = parts.agent.get(id);
    let mut layout = ui::VLayout::new();
    let h = line_height();
    {
        let mut line = |text: &str| -> ZResult {
            let image = Text::new(context, text, font)?.into_inner();
            let button = ui::Label::new(image, h);
            layout.add(Box::new(button));
            Ok(())
        };
        line(&format!("[{}]", meta.name))?;
        line(&format!(
            "strength: {}/{}",
            st.strength.0, st.base_strength.0
        ))?;
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
        line(&format!("move points: {}", a.move_points.0))?;
        if let Some(abilities) = parts.passive_abilities.get_opt(id) {
            if !abilities.0.is_empty() {
                line("[passive abilities]:")?;
                for ability in &abilities.0 {
                    line(&format!("'{}'", ability.to_string()))?;
                }
            }
        }
        if let Some(abilities) = parts.abilities.get_opt(id) {
            if !abilities.0.is_empty() {
                line("[abilities]:")?;
                for ability in &abilities.0 {
                    let s = ability.ability.to_string();
                    line(&format!("'{}'", s))?;
                }
            }
        }
        if let Some(effects) = parts.effects.get_opt(id) {
            if !effects.0.is_empty() {
                line("[effects]:")?;
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

fn build_panel_unit_abilities(
    context: &mut Context,
    font: &Font,
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
    if agent.attacks <= core::Attacks(0) && agent.jokers <= core::Jokers(0) {
        return Ok(None);
    }
    let mut layout = ui::VLayout::new();
    let h = line_height();
    for &ability in abilities {
        let text = match ability.status {
            ability::Status::Ready => format!("[{}]", ability.ability.to_string()),
            ability::Status::Cooldown(n) => format!("[{} ({})]", ability.ability.to_string(), n),
        };
        let image = Text::new(context, &text, font)?.into_inner();
        let button = ui::Button::new(image, h, gui.sender(), Message::Ability(ability.ability));
        layout.add(Box::new(button));
    }
    let anchor = ui::Anchor(ui::HAnchor::Right, ui::VAnchor::Middle);
    let layout = ui::pack(layout);
    gui.add(&layout, anchor);
    Ok(Some(layout))
}

fn make_gui(context: &mut Context, font: &Font) -> ZResult<ui::Gui<Message>> {
    let mut gui = ui::Gui::new(context);
    {
        let image = Text::new(context, "[deselect]", font)?.into_inner();
        let button = ui::Button::new(image, 0.1, gui.sender(), Message::Deselect);
        let mut layout = ui::VLayout::new();
        layout.add(Box::new(button));
        let anchor = ui::Anchor(ui::HAnchor::Right, ui::VAnchor::Top);
        gui.add(&ui::pack(layout), anchor);
    }
    {
        let image = Text::new(context, "[end turn]", font)?.into_inner();
        let button = ui::Button::new(image, 0.1, gui.sender(), Message::EndTurn);
        let mut layout = ui::VLayout::new();
        layout.add(Box::new(button));
        let anchor = ui::Anchor(ui::HAnchor::Right, ui::VAnchor::Bottom);
        gui.add(&ui::pack(layout), anchor);
    }
    {
        let image = Text::new(context, "[exit]", font)?.into_inner();
        let button = ui::Button::new(image, 0.1, gui.sender(), Message::Exit);
        let mut layout = ui::VLayout::new();
        layout.add(Box::new(button));
        let anchor = ui::Anchor(ui::HAnchor::Left, ui::VAnchor::Top);
        gui.add(&ui::pack(layout), anchor);
    }
    Ok(gui)
}

fn prepare_map_and_state(
    context: &mut Context,
    state: &mut State,
    view: &mut BattleView,
) -> ZResult {
    let mut actions = Vec::new();
    execute::create_terrain(state);
    actions.push(make_action_create_map(state, view, context)?);
    execute::create_objects(state, &mut |state, event, phase| {
        let action = visualize::visualize(state, view, context, event, phase)
            .expect("Can't visualize the event");
        let action = action::Fork::new(action).boxed();
        actions.push(action);
    });
    view.add_action(action::Sequence::new(actions).boxed());
    Ok(())
}

#[derive(Debug)]
pub struct Battle {
    font: graphics::Font, // TODO: use Context::default_font?
    gui: Gui<Message>,

    state: State,
    mode: SelectionMode,
    view: BattleView,
    selected_unit_id: Option<ObjId>,
    pathfinder: Pathfinder,
    block_timer: Option<Duration>,
    ai: Ai,
    panel_info: Option<ui::RcWidget>,
    panel_abilities: Option<ui::RcWidget>,
}

impl Battle {
    pub fn new(context: &mut Context) -> ZResult<Self> {
        let font = Font::new(context, "/OpenSans-Regular.ttf", 24)?;
        let gui = make_gui(context, &font)?;
        let mut prototypes_str = String::new();
        let mut file = context.filesystem.open("/objects.ron")?;
        file.read_to_string(&mut prototypes_str)?;
        let prototypes = ron::de::from_str(&prototypes_str).unwrap();
        debug!("{:?}", prototypes);
        let mut state = State::new(prototypes);
        let radius = state.map().radius();
        let mut view = BattleView::new(&state, context)?;
        prepare_map_and_state(context, &mut state, &mut view)?;
        Ok(Self {
            gui,
            font,
            view,
            mode: SelectionMode::Normal,
            state,
            selected_unit_id: None,
            pathfinder: Pathfinder::new(radius),
            block_timer: None,
            ai: Ai::new(PlayerId(1), radius),
            panel_info: None,
            panel_abilities: None,
        })
    }

    fn end_turn(&mut self, context: &mut Context) -> ZResult {
        if self.block_timer.is_some() {
            return Ok(());
        }
        self.deselect()?;
        let command = command::Command::EndTurn(command::EndTurn);
        let mut actions = Vec::new();
        actions.push(self.do_command_inner(context, &command));
        actions.push(self.do_ai(context));
        self.add_actions(actions);
        Ok(())
    }

    fn do_ai(&mut self, context: &mut Context) -> Box<Action> {
        debug!("AI: <");
        let mut actions = Vec::new();
        loop {
            let command = self.ai.command(&self.state).unwrap();
            debug!("AI: command = {:?}", command);
            actions.push(self.do_command_inner(context, &command));
            let time = Duration::from_millis(300); // TODO: use `time_s(0.3)`
            actions.push(action::Sleep::new(time).boxed());
            if let command::Command::EndTurn(_) = command {
                break;
            }
        }
        debug!("AI: >");
        action::Sequence::new(actions).boxed()
    }

    fn use_ability(&mut self, context: &mut Context, ability: Ability) -> ZResult {
        // TODO: code duplication (see check.rs and event.rs)
        let id = self.selected_unit_id.unwrap(); // TODO: Extract to some specific method
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
    ) -> Box<Action> {
        debug!("do_command_inner: {:?}", command);
        let mut actions = Vec::new();
        let state = &mut self.state;
        let view = &mut self.view;
        core::execute(state, command, &mut |state, event, phase| {
            let action = visualize::visualize(state, view, context, event, phase)
                .expect("Can't visualize the event");
            actions.push(action);
        }).expect("Can't execute command");
        action::Sequence::new(actions).boxed()
    }

    fn do_command(&mut self, context: &mut Context, command: &command::Command) {
        let action = self.do_command_inner(context, command);
        self.add_action(action);
    }

    fn add_actions(&mut self, actions: Vec<Box<Action>>) {
        self.add_action(action::Sequence::new(actions).boxed());
    }

    fn add_action(&mut self, action: Box<Action>) {
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
        if self.selected_unit_id.is_some() {
            self.view.deselect();
        }
        self.selected_unit_id = None;
        self.mode = SelectionMode::Normal;
        Ok(())
    }

    fn set_mode(&mut self, context: &mut Context, id: ObjId, mode: SelectionMode) -> ZResult {
        self.deselect()?;
        if self.state.parts().agent.get_opt(id).is_none() {
            // This object is not an agent or dead.
            return Ok(());
        }
        self.selected_unit_id = Some(id);
        let state = &self.state;
        let gui = &mut self.gui;
        match mode {
            SelectionMode::Ability(_) => {
                // TODO: Update the GUI here: explain how to use or cancel the ability.
                // 'Select target tile'
            }
            SelectionMode::Normal => {
                self.pathfinder.fill_map(state, id);
                self.panel_info = Some(build_panel_unit_info(context, &self.font, gui, state, id)?);
                self.panel_abilities =
                    build_panel_unit_abilities(context, &self.font, gui, state, id)?;
            }
        }
        let map = self.pathfinder.map();
        self.view.set_mode(state, map, context, id, &mode)?;
        self.mode = mode;
        Ok(())
    }

    fn handle_unit_click(&mut self, context: &mut Context, id: ObjId) -> ZResult {
        if self.state.parts().agent.get_opt(id).is_none() {
            // only agents can be selected
            return Ok(());
        }
        let other_unit_player_id = self.state.parts().belongs_to.get(id).0;
        if let Some(selected_unit_id) = self.selected_unit_id {
            let selected_unit_player_id = self.state.parts().belongs_to.get(selected_unit_id).0;
            if selected_unit_id == id {
                self.deselect()?;
                return Ok(());
            }
            if other_unit_player_id == selected_unit_player_id
                || other_unit_player_id == self.state.player_id()
            {
                self.set_mode(context, id, SelectionMode::Normal)?;
                return Ok(());
            }
            let command_attack = command::Command::Attack(command::Attack {
                attacker_id: selected_unit_id,
                target_id: id,
            });
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
        let selected_unit_id = self.selected_unit_id.unwrap();
        let parts = self.state.parts();
        if parts.agent.get_opt(selected_unit_id).is_some() {
            self.pathfinder.fill_map(&self.state, selected_unit_id);
        }
    }

    fn try_move_selected_unit(&mut self, context: &mut Context, pos: PosHex) {
        if let Some(id) = self.selected_unit_id {
            let path = match self.pathfinder.path(pos) {
                Some(path) => path,
                None => return,
            };
            let command_move = command::Command::MoveTo(command::MoveTo { id, path });
            if check(&self.state, &command_move).is_err() {
                return;
            }
            self.do_command(context, &command_move);
            self.fill_map();
        }
    }

    fn handle_event_click(&mut self, context: &mut Context, point: Point2) -> ZResult {
        let pos = geom::point_to_hex(self.view.tile_size(), point);
        self.gui.click(point);
        if self.block_timer.is_some() {
            return Ok(());
        }
        if self.state.map().is_inboard(pos) {
            if let SelectionMode::Ability(ability) = self.mode {
                let selected_id = self.selected_unit_id.unwrap();
                let command = command::Command::UseAbility(command::UseAbility {
                    id: selected_id,
                    pos,
                    ability,
                });
                if check(&self.state, &command).is_ok() {
                    self.do_command(context, &command);
                } else {
                    self.view.message(context, pos, "cancelled")?;
                }
                self.set_mode(context, selected_id, SelectionMode::Normal)?;
            } else if let Some(id) = state::agent_id_at_opt(&self.state, pos) {
                self.handle_unit_click(context, id)?;
            } else {
                self.try_move_selected_unit(context, pos);
            }
        }
        Ok(())
    }

    fn update_block_timer(&mut self, context: &mut Context, dtime: Duration) -> ZResult {
        if let Some(time) = self.block_timer {
            if time < dtime {
                self.block_timer = None;
                if let Some(id) = self.selected_unit_id {
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
        Ok(Transition::None)
    }

    fn draw(&self, context: &mut Context) -> ZResult {
        self.view.draw(context)?;
        self.gui.draw(context)
    }

    fn resize(&mut self, aspect_ratio: f32) {
        self.gui.resize(aspect_ratio);
    }

    fn click(&mut self, context: &mut Context, pos: Point2) -> ZResult<Transition> {
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
}
