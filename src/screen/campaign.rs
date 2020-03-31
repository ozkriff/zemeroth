use std::{
    sync::mpsc::{channel, Receiver},
    time::Duration,
};

use cgmath::Point2;
use gwg::{
    graphics::{self, Font, Text},
    Context,
};
use log::info;
use ui::{self, Gui};

use crate::{
    core::{
        battle::{
            component::{ObjType, Prototypes},
            scenario::{self, BattleType},
            state::BattleResult,
            PlayerId,
        },
        campaign::{Action, Mode, State},
    },
    screen::{self, Screen, StackCommand},
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

/// Big vertical spacer.
fn spacer_v() -> Box<dyn ui::Widget> {
    let h = utils::line_heights().big;
    Box::new(ui::Spacer::new_vertical(h))
}

/// Small vertical spacer.
fn spacer_v_small() -> Box<dyn ui::Widget> {
    let h = utils::line_heights().big / 8.0;
    Box::new(ui::Spacer::new_vertical(h))
}

/// Horizontal spacer.
fn spacer_h() -> Box<dyn ui::Widget> {
    let h = utils::line_heights().big;
    Box::new(ui::Spacer::new_horizontal(h / 4.0))
}

fn basic_gui(context: &mut Context) -> ZResult<Gui<Message>> {
    let mut gui = Gui::new(context);
    let h = utils::line_heights().large;
    let button_menu = {
        let icon = Box::new(graphics::Image::new(context, "/icon_menu.png")?);
        ui::Button::new(context, icon, h, gui.sender(), Message::Menu)?
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
    layout.add(spacer_v_small());
    for agent_type in agents {
        let mut line = ui::HLayout::new();
        line.add(label(context, font, &format!("- {}", agent_type.0))?);
        line.add(spacer_h());
        {
            let icon = Box::new(graphics::Image::new(context, "/icon_info.png")?);
            let message = Message::AgentInfo(agent_type.clone());
            let button = ui::Button::new(context, icon, h, gui.sender(), message)?;
            line.add(Box::new(button));
        }
        layout.add(Box::new(line));
        layout.add(spacer_v_small());
    }
    let layout = utils::wrap_widget_and_add_bg(context, Box::new(layout))?;
    Ok(Box::new(layout))
}

fn label(context: &mut Context, font: Font, text: &str) -> ZResult<Box<dyn ui::Widget>> {
    let h = utils::line_heights().big;
    let text = Box::new(Text::new((text, font, FONT_SIZE)));
    Ok(Box::new(ui::Label::new(context, text, h)?))
}

fn label_bg(context: &mut Context, font: Font, text: &str) -> ZResult<Box<dyn ui::Widget>> {
    let h = utils::line_heights().big;
    let text = Box::new(Text::new((text, font, FONT_SIZE)));
    Ok(Box::new(ui::Label::new_with_bg(context, text, h)?))
}

#[derive(Debug)]
pub struct Campaign {
    state: State,
    font: graphics::Font,
    receiver_battle_result: Option<Receiver<Option<BattleResult>>>,
    receiver_exit_confirmation: Option<Receiver<screen::confirm::Message>>,
    gui: Gui<Message>,
    layout: Option<ui::RcWidget>,
    label_central_message: Option<ui::RcWidget>,
}

impl Campaign {
    pub fn new(context: &mut Context) -> ZResult<Self> {
        let plan = utils::deserialize_from_file(context, "/campaign_01.ron")?;
        let upgrades = utils::deserialize_from_file(context, "/agent_campaign_info.ron")?;
        let state = State::new(plan, upgrades);
        let font = utils::default_font(context);
        let gui = basic_gui(context)?;
        let mut this = Self {
            gui,
            font,
            state,
            receiver_battle_result: None,
            receiver_exit_confirmation: None,
            layout: None,
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

    // TODO: Wrap the list into `ScrollArea`
    fn set_mode_preparing(&mut self, context: &mut Context) -> ZResult {
        let mut layout = ui::VLayout::new();
        let h = utils::line_heights().big;
        let casualties = self.state.last_battle_casualties();
        if !casualties.is_empty() {
            let layout_casualties = {
                let mut layout = ui::VLayout::new();
                let section_title = "In the last battle you have lost:";
                layout.add(label(context, self.font, section_title)?);
                for agent_type in casualties {
                    let text = &format!("- {} (killed)", agent_type.0);
                    layout.add(label(context, self.font, text)?);
                    layout.add(spacer_v_small());
                }
                utils::wrap_widget_and_add_bg(context, Box::new(layout))?
            };
            layout.add(Box::new(layout_casualties));
            layout.add(spacer_v());
        }
        let agents_panel =
            add_agents_panel(context, self.font, &mut self.gui, self.state.agents())?;
        layout.add(agents_panel);
        layout.add(spacer_v());
        let renown_text = &format!("Your renown is: {}", self.state.renown().0);
        layout.add(label_bg(context, self.font, renown_text)?);
        layout.add(spacer_v());
        let layout_actions = {
            let mut layout = ui::VLayout::new();
            layout.add(label(context, self.font, "Actions:")?);
            layout.add(spacer_v_small());
            for action in self.state.available_actions() {
                let mut line = ui::HLayout::new();
                let action_cost = self.state.action_cost(action);
                let text = match action {
                    Action::Recruit { agent_type } => {
                        format!("Recruit {} for {}r", agent_type.0, action_cost.0)
                    }
                    Action::Upgrade { from, to } => {
                        format!("Upgrade {} to {} for {}r", from.0, to.0, action_cost.0)
                    }
                };
                {
                    let text = Box::new(Text::new((text.as_str(), self.font, FONT_SIZE)));
                    let sender = self.gui.sender();
                    let message = Message::Action(action.clone());
                    let mut button = ui::Button::new(context, text, h, sender, message)?;
                    if action_cost.0 > self.state.renown().0 {
                        button.set_active(false);
                    }
                    line.add(Box::new(button));
                }
                line.add(spacer_h());
                {
                    let icon = Box::new(graphics::Image::new(context, "/icon_info.png")?);
                    let message = match action {
                        Action::Recruit { agent_type, .. } => {
                            Message::AgentInfo(agent_type.clone())
                        }
                        Action::Upgrade { to, .. } => Message::AgentInfo(to.clone()),
                    };
                    let sender = self.gui.sender();
                    let button = ui::Button::new(context, icon, h, sender, message)?;
                    line.add(Box::new(button));
                }
                layout.add(Box::new(line));
                layout.add(spacer_v_small());
            }
            {
                let text = &format!(
                    "Start battle - {}/{}",
                    self.state.current_scenario_index() + 1,
                    self.state.scenarios_count()
                );
                let text = Box::new(Text::new((text.as_str(), self.font, FONT_SIZE)));
                let command = Message::StartBattle;
                let button = ui::Button::new(context, text, h, self.gui.sender(), command)?;
                layout.add(Box::new(button));
            }
            utils::wrap_widget_and_add_bg(context, Box::new(layout))?
        };
        layout.add(Box::new(layout_actions));
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
        utils::remove_widget(&mut self.gui, &mut self.label_central_message)?;
        Ok(())
    }

    fn add_label_central_message(&mut self, context: &mut Context, text: &str) -> ZResult {
        let h = utils::line_heights().large;
        let text = Box::new(Text::new((text, self.font, FONT_SIZE)));
        let label = ui::pack(ui::Label::new_with_bg(context, text, h)?);
        let anchor = ui::Anchor(ui::HAnchor::Middle, ui::VAnchor::Middle);
        self.gui.add(&label, anchor);
        self.label_central_message = Some(label);
        Ok(())
    }

    fn start_battle(&mut self, context: &mut Context) -> ZResult<Box<dyn Screen>> {
        let mut scenario = self.state.scenario().clone();
        // TODO: extract a function for this? add_player_agents_to_scenario?
        for typename in self.state.agents() {
            scenario.objects.push(scenario::ObjectsGroup {
                owner: Some(PlayerId(0)),
                typename: typename.clone(),
                line: Some(scenario::Line::Middle),
                count: 1,
            });
        }
        let (sender, receiver) = channel();
        self.receiver_battle_result = Some(receiver);
        let prototypes = Prototypes::from_str(&utils::read_file(context, "/objects.ron")?);
        let battle_type = BattleType::CampaignNode;
        let screen = screen::Battle::new(context, scenario, battle_type, prototypes, sender)?;
        Ok(Box::new(screen))
    }
}

impl Screen for Campaign {
    fn update(&mut self, context: &mut Context, _dtime: Duration) -> ZResult<StackCommand> {
        if let Some(result) = utils::try_receive(&self.receiver_battle_result) {
            if let Some(result) = result {
                self.state
                    .report_battle_results(&result)
                    .expect("Campaign: Can't report battle results");
                let new_mode = self.state.mode();
                self.set_mode(context, new_mode)?;
            } else {
                // None result means that the player has abandoned the campaign battle.
                // This means abandoning the campaign too.
                return Ok(StackCommand::Pop);
            }
        };
        if screen::confirm::try_receive_yes(&self.receiver_exit_confirmation) {
            Ok(StackCommand::Pop)
        } else {
            Ok(StackCommand::None)
        }
    }

    fn draw(&self, context: &mut Context) -> ZResult {
        self.gui.draw(context)?;
        Ok(())
    }

    fn resize(&mut self, aspect_ratio: f32) {
        self.gui.resize(aspect_ratio);
    }

    fn click(&mut self, context: &mut Context, pos: Point2<f32>) -> ZResult<StackCommand> {
        let message = self.gui.click(pos);
        info!(
            "screen::Campaign: click: pos={:?}, message={:?}",
            pos, message
        );
        match message {
            Some(Message::StartBattle) => {
                let screen = self.start_battle(context)?;
                Ok(StackCommand::PushScreen(screen))
            }
            Some(Message::Action(action)) => {
                let cost = self.state.action_cost(&action);
                if cost.0 <= self.state.renown().0 {
                    self.state.exectute_action(action);
                    let new_mode = self.state.mode();
                    self.set_mode(context, new_mode)?;
                }
                Ok(StackCommand::None)
            }
            Some(Message::Menu) => {
                // Ask only if the player hasn't won or failed, otherwise just pop the screen.
                if self.state.mode() == Mode::PreparingForBattle {
                    let (sender, receiver) = channel();
                    self.receiver_exit_confirmation = Some(receiver);
                    let screen =
                        screen::Confirm::from_line(context, "Abandon the campaign?", sender)?;
                    Ok(StackCommand::PushPopup(Box::new(screen)))
                } else {
                    Ok(StackCommand::Pop)
                }
            }
            Some(Message::AgentInfo(typename)) => {
                let prototypes = Prototypes::from_str(&utils::read_file(context, "/objects.ron")?);
                let popup_screen = screen::AgentInfo::new(context, prototypes, &typename)?;
                Ok(StackCommand::PushPopup(Box::new(popup_screen)))
            }
            None => Ok(StackCommand::None),
        }
    }

    fn move_mouse(&mut self, _context: &mut Context, pos: Point2<f32>) -> ZResult {
        self.gui.move_mouse(pos);
        Ok(())
    }
}
