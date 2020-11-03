use std::{
    sync::mpsc::{channel, Receiver},
    time::Duration,
};

use heck::TitleCase;
use log::info;
use macroquad::prelude::{Color, Font, Vec2};
use ui::{self, Gui, Widget};

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
    UpgradeInfo { from: ObjType, to: ObjType },
    Action(Action),
}

const FONT_SIZE: u16 = utils::font_size();

// The main line height of this screen.
fn line_height() -> f32 {
    utils::line_heights().normal
}

fn line_height_small() -> f32 {
    line_height() / 8.0
}

fn basic_gui() -> ZResult<Gui<Message>> {
    let mut gui = Gui::new();
    let h = utils::line_heights().large;
    let button_menu = {
        let icon = crate::Image::new("/img/icon_menu.png")?;
        ui::Button::new(icon, h, gui.sender(), Message::Menu)?
    };
    let mut layout = ui::VLayout::new();
    layout.add(Box::new(button_menu));
    let anchor = ui::Anchor(ui::HAnchor::Left, ui::VAnchor::Top);
    gui.add(&ui::pack(layout), anchor);
    Ok(gui)
}

fn build_panel_agents(
    font: Font,
    gui: &mut ui::Gui<Message>,
    agents: &[ObjType],
) -> ZResult<Box<dyn ui::Widget>> {
    let mut layout = Box::new(ui::VLayout::new().stretchable(true));
    layout.add(label(font, "Your group consists of:")?);
    layout.add(Box::new(ui::Spacer::new_vertical(line_height_small())));
    for agent_type in agents {
        let mut line = ui::HLayout::new().stretchable(true);
        let title = agent_type.0.to_title_case();
        line.add(label(font, &format!("- {}", title))?);
        let spacer = ui::Spacer::new_horizontal(line_height_small()).stretchable(true);
        line.add(Box::new(spacer));
        {
            let icon = crate::Image::new("/img/icon_info.png")?;
            let message = Message::AgentInfo(agent_type.clone());
            let button = ui::Button::new(icon, line_height(), gui.sender(), message)?;
            line.add(Box::new(button));
        }
        layout.add(Box::new(line));
        layout.add(Box::new(ui::Spacer::new_vertical(line_height_small())));
    }
    layout.stretch_to_self()?;
    let layout = utils::add_offsets_and_bg_big(layout)?.stretchable(true);
    Ok(Box::new(layout))
}

fn build_panel_casualties(font: Font, state: &State) -> ZResult<Option<Box<dyn ui::Widget>>> {
    let casualties = state.last_battle_casualties();
    if casualties.is_empty() {
        return Ok(None);
    }
    let mut layout = Box::new(ui::VLayout::new());
    let section_title = "In the last battle you have lost:";
    layout.add(label(font, section_title)?);
    for agent_type in casualties {
        let text = &format!("- {}", agent_type.0.to_title_case());
        layout.add(label(font, text)?);
        layout.add(Box::new(ui::Spacer::new_vertical(line_height_small())));
    }
    let layout = utils::add_offsets_and_bg_big(layout)?.stretchable(true);
    Ok(Some(Box::new(layout)))
}

fn build_panel_renown(font: Font, state: &State) -> ZResult<Box<dyn ui::Widget>> {
    let mut layout = Box::new(ui::VLayout::new().stretchable(true));
    let renown_text = &format!("Your renown is: {}r", state.renown().0);
    layout.add(label(font, renown_text)?);
    let layout = utils::add_offsets_and_bg_big(layout)?.stretchable(true);
    Ok(Box::new(layout))
}

fn build_panel_actions(
    font: Font,
    gui: &mut ui::Gui<Message>,
    state: &State,
) -> ZResult<Box<dyn ui::Widget>> {
    let h = line_height();
    let mut layout = Box::new(ui::VLayout::new().stretchable(true));
    layout.add(label(font, "Actions:")?);
    layout.add(Box::new(ui::Spacer::new_vertical(line_height_small())));
    for action in state.available_actions() {
        let mut line = ui::HLayout::new().stretchable(true);
        let action_cost = state.action_cost(action);
        let text = match action {
            Action::Recruit { agent_type } => {
                let title = agent_type.0.to_title_case();
                format!("Recruit {} for {}r", title, action_cost.0)
            }
            Action::Upgrade { from, to } => {
                let from = from.0.to_title_case();
                let to = to.0.to_title_case();
                format!("Upgrade {} to {} for {}r", from, to, action_cost.0)
            }
        };
        {
            let text = ui::Drawable::text(text.as_str(), font, FONT_SIZE);
            let sender = gui.sender();
            let message = Message::Action(action.clone());
            let mut button = ui::Button::new(text, h, sender, message)?.stretchable(true);
            if action_cost.0 > state.renown().0 {
                button.set_active(false);
            }
            line.add(Box::new(button));
        }
        line.add(Box::new(ui::Spacer::new_horizontal(line_height_small())));
        {
            let icon = crate::Image::new("/img/icon_info.png")?;
            let message = match action.clone() {
                Action::Recruit { agent_type, .. } => Message::AgentInfo(agent_type),
                Action::Upgrade { from, to } => Message::UpgradeInfo { from, to },
            };
            let sender = gui.sender();
            let button = ui::Button::new(icon, h, sender, message)?;
            line.add(Box::new(button));
        }
        layout.add(Box::new(line));
        layout.add(Box::new(ui::Spacer::new_vertical(line_height_small())));
    }
    {
        let text = &format!(
            "Start battle - {}/{}",
            state.current_scenario_index() + 1,
            state.scenarios_count()
        );
        let text = ui::Drawable::text(text.as_str(), font, FONT_SIZE);
        let command = Message::StartBattle;
        let button = ui::Button::new(text, h, gui.sender(), command)?.stretchable(true);
        layout.add(Box::new(button));
    }
    layout.stretch_to_self()?;
    let layout = utils::add_offsets_and_bg_big(layout)?.stretchable(true);
    Ok(Box::new(layout))
}

fn label(font: Font, text: &str) -> ZResult<Box<dyn ui::Widget>> {
    let text = ui::Drawable::text(text, font, FONT_SIZE);
    Ok(Box::new(ui::Label::new(text, line_height())?))
}

#[derive(Debug)]
pub struct Campaign {
    state: State,
    font: Font,
    receiver_battle_result: Option<Receiver<Option<BattleResult>>>,
    receiver_exit_confirmation: Option<Receiver<screen::confirm::Message>>,
    gui: Gui<Message>,
    layout: Option<ui::RcWidget>,
    label_central_message: Option<ui::RcWidget>,
}

impl Campaign {
    pub async fn new() -> ZResult<Self> {
        let plan = utils::deserialize_from_file("/campaign_01.ron").await?;
        let upgrades = utils::deserialize_from_file("/agent_campaign_info.ron").await?;
        let state = State::new(plan, upgrades);
        let font = utils::default_font();
        let gui = basic_gui()?;
        let mut this = Self {
            gui,
            font,
            state,
            receiver_battle_result: None,
            receiver_exit_confirmation: None,
            layout: None,
            label_central_message: None,
        };
        this.set_mode(Mode::PreparingForBattle)?;
        Ok(this)
    }

    fn set_mode(&mut self, mode: Mode) -> ZResult {
        self.clean_ui()?;
        match mode {
            Mode::PreparingForBattle => self.set_mode_preparing()?,
            Mode::Won => self.set_mode_won()?,
            Mode::Failed => self.set_mode_failed()?,
        }
        Ok(())
    }

    // TODO: Wrap the list into `ScrollArea`
    fn set_mode_preparing(&mut self) -> ZResult {
        let state = &self.state;
        let gui = &mut self.gui;
        let mut layout = ui::VLayout::new().stretchable(true);
        if let Some(panel) = build_panel_casualties(self.font, state)? {
            layout.add(panel);
            layout.add(Box::new(ui::Spacer::new_vertical(line_height())));
        }
        let mut line = ui::HLayout::new().stretchable(true);
        line.add(build_panel_agents(self.font, gui, state.agents())?);
        line.add(Box::new(ui::Spacer::new_horizontal(line_height())));
        line.add(build_panel_renown(self.font, state)?);
        layout.add(Box::new(line));
        layout.add(Box::new(ui::Spacer::new_vertical(line_height())));
        layout.add(build_panel_actions(self.font, gui, state)?);
        layout.stretch_to_self()?;
        let anchor = ui::Anchor(ui::HAnchor::Middle, ui::VAnchor::Middle);
        let layout = ui::pack(layout);
        self.gui.add(&layout, anchor);
        self.layout = Some(layout);
        Ok(())
    }

    fn set_mode_won(&mut self) -> ZResult {
        self.add_label_central_message("You have won!")
    }

    fn set_mode_failed(&mut self) -> ZResult {
        self.add_label_central_message("You have failed!")
    }

    fn clean_ui(&mut self) -> ZResult {
        utils::remove_widget(&mut self.gui, &mut self.layout)?;
        utils::remove_widget(&mut self.gui, &mut self.label_central_message)?;
        Ok(())
    }

    fn add_label_central_message(&mut self, text: &str) -> ZResult {
        let h = utils::line_heights().large;
        let text = ui::Drawable::text(text, self.font, FONT_SIZE);
        let label = ui::pack(ui::Label::new_with_bg(text, h)?);
        let anchor = ui::Anchor(ui::HAnchor::Middle, ui::VAnchor::Middle);
        self.gui.add(&label, anchor);
        self.label_central_message = Some(label);
        Ok(())
    }

    async fn start_battle(&mut self) -> ZResult<Box<dyn Screen>> {
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
        let prototypes = Prototypes::from_str(&utils::read_file("/objects.ron").await?);
        let battle_type = BattleType::CampaignNode;
        let screen = screen::Battle::new(scenario, battle_type, prototypes, sender).await?;
        Ok(Box::new(screen))
    }
}

impl Screen for Campaign {
    fn update(&mut self, _dtime: Duration) -> ZResult<StackCommand> {
        if let Some(result) = utils::try_receive(&self.receiver_battle_result) {
            if let Some(result) = result {
                self.state
                    .report_battle_results(&result)
                    .expect("Campaign: Can't report battle results");
                let new_mode = self.state.mode();
                self.set_mode(new_mode)?;
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

    fn draw(&self) -> ZResult {
        self.gui.draw();
        Ok(())
    }

    fn click(&mut self, pos: Vec2) -> ZResult<StackCommand> {
        let message = self.gui.click(pos);
        info!(
            "screen::Campaign: click: pos={:?}, message={:?}",
            pos, message
        );
        match message {
            Some(Message::StartBattle) => {
                let screen = unimplemented!();
                Ok(StackCommand::PushScreen(screen))
            }
            Some(Message::Action(action)) => {
                let cost = self.state.action_cost(&action);
                if cost.0 <= self.state.renown().0 {
                    self.state.execute_action(action);
                    let new_mode = self.state.mode();
                    self.set_mode(new_mode)?;
                }
                Ok(StackCommand::None)
            }
            Some(Message::Menu) => {
                // Ask only if the player hasn't won or failed, otherwise just pop the screen.
                if self.state.mode() == Mode::PreparingForBattle {
                    // let (sender, receiver) = channel();
                    // self.receiver_exit_confirmation = Some(receiver);
                    // let screen = screen::Confirm::from_line("Abandon the campaign?", sender)?;
                    // Ok(StackCommand::PushPopup(Box::new(screen)))
                    unimplemented!()
                } else {
                    Ok(StackCommand::Pop)
                }
            }
            Some(Message::AgentInfo(typename)) => {
                // let prototypes = Prototypes::from_str(&utils::read_file("/objects.ron")?);
                // let popup = screen::AgentInfo::new_agent_info(&prototypes, &typename)?;
                // Ok(StackCommand::PushPopup(Box::new(popup)))
                unimplemented!()
            }
            Some(Message::UpgradeInfo { from, to }) => {
                // let prototypes = Prototypes::from_str(&utils::read_file("/objects.ron")?);
                // let popup = screen::AgentInfo::new_upgrade_info(&prototypes, &from, &to)?;
                // Ok(StackCommand::PushPopup(Box::new(popup)))
                unimplemented!()
            }
            None => Ok(StackCommand::None),
        }
    }

    fn resize(&mut self, aspect_ratio: f32) {
        self.gui.resize(aspect_ratio);
    }

    fn move_mouse(&mut self, pos: Vec2) -> ZResult {
        self.gui.move_mouse(pos);
        Ok(())
    }
}
