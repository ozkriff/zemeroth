use std::{
    sync::mpsc::{channel, Receiver},
    time::Duration,
};

use ggez::{
    graphics::{self, Font, Text},
    nalgebra::Point2,
    Context,
};
use log::info;
use ui::{self, Gui};

use crate::{
    core::{
        battle::{
            component::{ObjType, Prototypes},
            scenario,
            state::BattleResult,
            PlayerId,
        },
        campaign::{Action, Mode, State},
    },
    screen::{self, Screen, Transition},
    utils, ZResult,
};

#[derive(Clone, Debug)]
enum Message {
    Menu,
    StartBattle,
    AgentInfo(ObjType),
    Action(Action),
}

const FONT_SIZE: f32 = utils::font_size();

fn basic_gui(context: &mut Context, font: Font) -> ZResult<Gui<Message>> {
    let mut gui = Gui::new(context);
    let h = utils::line_heights().big;
    let button_menu = {
        let text = Box::new(Text::new(("[exit]", font, FONT_SIZE)));
        ui::Button::new(context, text, h, gui.sender(), Message::Menu)?
    };
    let mut layout = ui::VLayout::new();
    layout.add(Box::new(button_menu));
    let anchor = ui::Anchor(ui::HAnchor::Left, ui::VAnchor::Top);
    gui.add(&ui::pack(layout), anchor);
    Ok(gui)
}

fn add_agents_panel(
    context: &mut Context,
    font: Font,
    gui: &mut ui::Gui<Message>,
    agents: &[ObjType],
) -> ZResult<Box<dyn ui::Widget>> {
    let mut layout = ui::VLayout::new();
    let h = utils::line_heights().big;
    layout.add(label(context, font, "Your group consists of:")?);
    for agent_type in agents {
        let mut line = ui::HLayout::new();
        line.add(label(context, font, &format!("- {}", agent_type.0))?);
        line.add(spacer_h());
        {
            let text = Box::new(Text::new(("[i]", font, FONT_SIZE)));
            let message = Message::AgentInfo(agent_type.clone());
            let button = ui::Button::new(context, text, h, gui.sender(), message)?;
            line.add(Box::new(button));
        }
        layout.add(Box::new(line));
    }
    Ok(Box::new(layout))
}

fn spacer() -> Box<dyn ui::Widget> {
    let h = utils::line_heights().big;
    let rect = graphics::Rect {
        h,
        ..Default::default()
    };
    Box::new(ui::Spacer::new(rect))
}

/// Horizontal spacer.
fn spacer_h() -> Box<dyn ui::Widget> {
    let h = utils::line_heights().big;
    let rect = graphics::Rect {
        w: h / 2.0,
        h: 0.0,
        ..Default::default()
    };
    Box::new(ui::Spacer::new(rect))
}

fn label(context: &mut Context, font: Font, text: &str) -> ZResult<Box<dyn ui::Widget>> {
    let h = utils::line_heights().big;
    let text = Box::new(Text::new((text, font, FONT_SIZE)));
    Ok(Box::new(ui::Label::new(context, text, h)?))
}

#[derive(Debug)]
pub struct Campaign {
    state: State,
    font: graphics::Font,
    receiver: Option<Receiver<BattleResult>>,
    gui: Gui<Message>,
    layout: Option<ui::RcWidget>,
    button_start_battle: Option<ui::RcWidget>,
    label_central_message: Option<ui::RcWidget>,
}

impl Campaign {
    pub fn new(context: &mut Context) -> ZResult<Self> {
        let plan = utils::deserialize_from_file(context, "/campaign_01.ron")?;
        let upgrades = utils::deserialize_from_file(context, "/agent_campaign_info.ron")?;
        let state = State::new(plan, upgrades);
        let font = utils::default_font(context);
        let gui = basic_gui(context, font)?;
        let mut this = Self {
            gui,
            font,
            state,
            receiver: None,
            layout: None,
            button_start_battle: None,
            label_central_message: None,
        };
        this.set_mode(context, Mode::PreparingForBattle)?;
        Ok(this)
    }

    fn set_mode(&mut self, context: &mut Context, mode: Mode) -> ZResult {
        self.clean_ui()?;
        match mode {
            Mode::PreparingForBattle => self.set_mode_preparing(context)?,
            Mode::Won => self.set_mode_won(context)?,
            Mode::Failed => self.set_mode_failed(context)?,
        }
        Ok(())
    }

    fn set_mode_preparing(&mut self, context: &mut Context) -> ZResult {
        let mut layout = ui::VLayout::new();
        let h = utils::line_heights().big;
        let casualties = self.state.last_battle_casualties();
        if !casualties.is_empty() {
            layout.add(label(
                context,
                self.font,
                "In the last battle you have lost:",
            )?);
            for agent_type in casualties {
                let text = &format!("- {} (killed)", agent_type.0);
                layout.add(label(context, self.font, text)?);
            }
        }
        layout.add(spacer());
        let panel = add_agents_panel(context, self.font, &mut self.gui, self.state.agents())?;
        layout.add(panel);
        layout.add(spacer());
        layout.add(label(
            context,
            self.font,
            &format!("Your renown is: {}", self.state.renown().0),
        )?);
        layout.add(spacer());
        layout.add(label(context, self.font, &"Actions:")?);
        for action in self.state.available_actions() {
            let mut line = ui::HLayout::new();
            {
                let cost = self.state.action_cost(action);
                let text = match action {
                    Action::Recruit { agent_type } => {
                        format!("- [Recruit {} for {}r]", agent_type.0, cost.0)
                    }
                    Action::Upgrade { from, to } => {
                        format!("- [Upgrade {} to {} for {}r]", from.0, to.0, cost.0)
                    }
                };
                if cost.0 <= self.state.renown().0 {
                    let text = Box::new(Text::new((text.as_str(), self.font, FONT_SIZE)));
                    let sender = self.gui.sender();
                    let message = Message::Action(action.clone());
                    let button = ui::Button::new(context, text, h, sender, message)?;
                    line.add(Box::new(button));
                } else {
                    line.add(label(context, self.font, &text)?);
                }
            }
            line.add(spacer_h());
            {
                let text = Box::new(Text::new(("[i]", self.font, FONT_SIZE)));
                let message = match action {
                    Action::Recruit { agent_type, .. } => Message::AgentInfo(agent_type.clone()),
                    Action::Upgrade { to, .. } => Message::AgentInfo(to.clone()),
                };
                let sender = self.gui.sender();
                let button = ui::Button::new(context, text, h, sender, message)?;
                line.add(Box::new(button));
            }
            layout.add(Box::new(line));
        }
        {
            let text = &format!(
                "- [Start battle - {}/{}]",
                self.state.current_scenario_index() + 1,
                self.state.scenarios_count()
            );
            let text = Box::new(Text::new((text.as_str(), self.font, FONT_SIZE)));
            let button =
                ui::Button::new(context, text, h, self.gui.sender(), Message::StartBattle)?;
            layout.add(Box::new(button));
        }
        let anchor = ui::Anchor(ui::HAnchor::Middle, ui::VAnchor::Middle);
        let layout = ui::pack(layout);
        self.gui.add(&layout, anchor);
        self.layout = Some(layout);
        Ok(())
    }

    fn set_mode_won(&mut self, context: &mut Context) -> ZResult {
        self.add_label_central_message(context, "You have won!")
    }

    fn set_mode_failed(&mut self, context: &mut Context) -> ZResult {
        self.add_label_central_message(context, "You have failed!")
    }

    fn clean_ui(&mut self) -> ZResult {
        utils::remove_widget(&mut self.gui, &mut self.layout)?;
        utils::remove_widget(&mut self.gui, &mut self.button_start_battle)?;
        utils::remove_widget(&mut self.gui, &mut self.label_central_message)?;
        Ok(())
    }

    fn add_label_central_message(&mut self, context: &mut Context, text: &str) -> ZResult {
        let h = utils::line_heights().large;
        let text = Box::new(Text::new((text, self.font, FONT_SIZE)));
        let label = ui::pack(ui::Label::new(context, text, h)?);
        let anchor = ui::Anchor(ui::HAnchor::Middle, ui::VAnchor::Middle);
        self.gui.add(&label, anchor);
        self.label_central_message = Some(label);
        Ok(())
    }

    fn try_get_battle_result(&self) -> Option<BattleResult> {
        if let Some(ref receiver) = self.receiver {
            receiver.try_recv().ok()
        } else {
            None
        }
    }

    fn start_battle(&mut self, context: &mut Context) -> ZResult<Box<dyn Screen>> {
        let mut scenario = self.state.scenario().clone();
        for typename in self.state.agents() {
            scenario.objects.push(scenario::ObjectsGroup {
                owner: Some(PlayerId(0)),
                typename: typename.clone(),
                line: Some(scenario::Line::Middle),
                count: 1,
            });
        }
        let (sender, receiver) = channel();
        self.receiver = Some(receiver);
        let prototypes = Prototypes::from_str(&utils::read_file(context, "/objects.ron")?);
        let screen = screen::Battle::new(context, scenario, prototypes, sender)?;
        Ok(Box::new(screen))
    }
}

impl Screen for Campaign {
    fn update(&mut self, context: &mut Context, _dtime: Duration) -> ZResult<Transition> {
        if let Some(result) = self.try_get_battle_result() {
            self.state
                .report_battle_results(&result)
                .expect("Campaign: Can't report battle results");
            let new_mode = self.state.mode();
            self.set_mode(context, new_mode)?;
        };
        Ok(Transition::None)
    }

    fn draw(&self, context: &mut Context) -> ZResult {
        self.gui.draw(context)?;
        Ok(())
    }

    fn resize(&mut self, aspect_ratio: f32) {
        self.gui.resize(aspect_ratio);
    }

    fn click(&mut self, context: &mut Context, pos: Point2<f32>) -> ZResult<Transition> {
        let message = self.gui.click(pos);
        info!(
            "screen::Campaign: click: pos={:?}, message={:?}",
            pos, message
        );
        match message {
            Some(Message::StartBattle) => {
                let screen = self.start_battle(context)?;
                Ok(Transition::Push(screen))
            }
            Some(Message::Action(action)) => {
                self.state.exectute_action(action);
                let new_mode = self.state.mode();
                self.set_mode(context, new_mode)?;
                Ok(Transition::None)
            }
            Some(Message::Menu) => Ok(Transition::Pop),
            Some(Message::AgentInfo(typename)) => {
                let prototypes = Prototypes::from_str(&utils::read_file(context, "/objects.ron")?);
                let screen = screen::AgentInfo::new(context, prototypes, &typename)?;
                Ok(Transition::Push(Box::new(screen)))
            }
            None => Ok(Transition::None),
        }
    }

    fn move_mouse(&mut self, _context: &mut Context, pos: Point2<f32>) -> ZResult {
        self.gui.move_mouse(pos);
        Ok(())
    }
}
