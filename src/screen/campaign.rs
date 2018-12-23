use std::{
    sync::mpsc::{channel, Receiver},
    time::Duration,
};

use ggez::{
    graphics::{self, Font, Point2, Text},
    Context,
};
use log::info;
use ui::{self, Gui};

use crate::{
    core::{
        campaign::{Mode, State},
        tactical_map::{scenario, state::BattleResult, PlayerId},
    },
    screen::{self, Screen, Transition},
    utils, ZResult,
};

#[derive(Clone, Debug)]
enum Message {
    Menu,
    StartBattle,
    Recruit(String),
}

// TODO: Fix code duplication with `Battle` screen! (Move to `utils` mod?)
fn line_height() -> f32 {
    0.08 * 1.5
}

fn basic_gui(context: &mut Context, font: &Font) -> ZResult<Gui<Message>> {
    let mut gui = Gui::new(context);
    let h = line_height();
    let button_menu = {
        let image = Text::new(context, "[exit]", font)?.into_inner();
        ui::Button::new(context, image, h, gui.sender(), Message::Menu)
    };
    let mut layout = ui::VLayout::new();
    layout.add(Box::new(button_menu));
    let anchor = ui::Anchor(ui::HAnchor::Left, ui::VAnchor::Top);
    gui.add(&ui::pack(layout), anchor);
    Ok(gui)
}

fn add_agents_panel(
    context: &mut Context,
    font: &Font,
    layout: &mut ui::VLayout,
    agents: &[String],
) -> ZResult {
    let h = line_height();
    {
        let text = "Your group consists of:";
        let image = Text::new(context, text, font)?.into_inner();
        layout.add(Box::new(ui::Label::new(context, image, h)));
    }
    for agent_type in agents {
        let text = format!("- {}", agent_type);
        let image = Text::new(context, &text, font)?.into_inner();
        let label = ui::Label::new(context, image, h);
        layout.add(Box::new(label));
    }
    Ok(())
}

// TODO: add spacer widget to ZGui. Remove this hack.
// https://github.com/ozkriff/zemeroth/issues/382
fn add_spacer(context: &mut Context, font: &Font, layout: &mut ui::VLayout) -> ZResult {
    let h = line_height();
    let image = Text::new(context, "", font)?.into_inner();
    layout.add(Box::new(ui::Label::new(context, image, h)));
    Ok(())
}

fn label(context: &mut Context, font: &Font, text: &str) -> ZResult<Box<dyn ui::Widget>> {
    let h = line_height();
    let image = Text::new(context, text, font)?.into_inner();
    Ok(Box::new(ui::Label::new(context, image, h)))
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
        let state = State::from_plan(plan);
        let font = utils::default_font(context);
        let gui = basic_gui(context, &font)?;
        let mut this = Self {
            gui,
            font,
            state,
            receiver: None,
            layout: None,
            button_start_battle: None,
            label_central_message: None,
        };
        this.set_mode(context, Mode::ReadyForBattle)?;
        Ok(this)
    }

    fn set_mode(&mut self, context: &mut Context, mode: Mode) -> ZResult {
        self.clean_ui()?;
        match mode {
            Mode::PreparingForBattle => self.set_mode_preparing(context)?,
            Mode::ReadyForBattle => self.set_mode_ready(context)?,
            Mode::Won => self.set_mode_won(context)?,
            Mode::Failed => self.set_mode_failed(context)?,
        }
        Ok(())
    }

    fn set_mode_preparing(&mut self, context: &mut Context) -> ZResult {
        let mut layout = ui::VLayout::new();
        let h = line_height();

        let casualties = self.state.last_battle_casualties();
        if !casualties.is_empty() {
            layout.add(label(
                context,
                &self.font,
                "In the last battle you have lost:",
            )?);
            for agent_type in casualties {
                let text = format!("- {} (killed)", agent_type);
                let image = Text::new(context, &text, &self.font)?.into_inner();
                let label = ui::Label::new(context, image, h);
                layout.add(Box::new(label));
            }
        }

        add_spacer(context, &self.font, &mut layout)?;
        add_agents_panel(context, &self.font, &mut layout, self.state.agents())?;
        add_spacer(context, &self.font, &mut layout)?;

        if let Mode::PreparingForBattle = self.state.mode() {
            {
                let image = Text::new(context, "Choose:", &self.font)?.into_inner();
                layout.add(Box::new(ui::Label::new(context, image, h)));
            }
            for agent_type in self.state.aviable_recruits() {
                let text = format!("- [Recruit {}]", agent_type);
                let image = Text::new(context, &text, &self.font)?.into_inner();
                let sender = self.gui.sender();
                let message = Message::Recruit(agent_type.to_string());
                let button = ui::Button::new(context, image, h, sender, message);
                layout.add(Box::new(button));
            }
        }

        let anchor = ui::Anchor(ui::HAnchor::Middle, ui::VAnchor::Middle);
        let layout = ui::pack(layout);
        self.gui.add(&layout, anchor);
        self.layout = Some(layout);

        Ok(())
    }

    fn set_mode_ready(&mut self, context: &mut Context) -> ZResult {
        let mut layout = ui::VLayout::new();
        add_agents_panel(context, &self.font, &mut layout, self.state.agents())?;
        let anchor = ui::Anchor(ui::HAnchor::Middle, ui::VAnchor::Middle);
        let layout = ui::pack(layout);
        self.gui.add(&layout, anchor);
        self.layout = Some(layout);
        {
            let h = line_height() * 1.5;
            let text = &format!(
                "[start battle - {}/{}]",
                self.state.current_scenario_index() + 1,
                self.state.scenarios_count()
            );
            let image = Text::new(context, text, &self.font)?.into_inner();
            let button =
                ui::Button::new(context, image, h, self.gui.sender(), Message::StartBattle);
            let rc_button = ui::pack(button);
            let anchor = ui::Anchor(ui::HAnchor::Middle, ui::VAnchor::Bottom);
            self.gui.add(&rc_button, anchor);
            self.button_start_battle = Some(rc_button);
        }
        Ok(())
    }

    fn set_mode_won(&mut self, context: &mut Context) -> ZResult {
        self.add_label_central_message(context, "You have won!")?;
        Ok(())
    }

    fn set_mode_failed(&mut self, context: &mut Context) -> ZResult {
        self.add_label_central_message(context, "You have failed!")?;
        Ok(())
    }

    fn clean_ui(&mut self) -> ZResult {
        if let Some(button) = self.button_start_battle.take() {
            self.gui.remove(&button)?;
        }
        if let Some(panel) = self.layout.take() {
            self.gui.remove(&panel)?;
        }
        if let Some(label) = self.label_central_message.take() {
            self.gui.remove(&label)?;
        }
        Ok(())
    }

    fn add_label_central_message(&mut self, context: &mut Context, text: &str) -> ZResult {
        let h = line_height() * 1.5;
        let image = Text::new(context, text, &self.font)?.into_inner();
        let label = ui::pack(ui::Label::new(context, image, h));
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
                line: scenario::Line::Middle,
                count: 1,
            });
        }
        let (sender, receiver) = channel();
        self.receiver = Some(receiver);
        let screen = screen::Battle::new(context, scenario, sender)?;
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
        self.gui.draw(context)
    }

    fn resize(&mut self, aspect_ratio: f32) {
        self.gui.resize(aspect_ratio);
    }

    fn click(&mut self, context: &mut Context, pos: Point2) -> ZResult<Transition> {
        let message = self.gui.click(pos);
        info!(
            "StrategyScreen: click: pos={:?}, message={:?}",
            pos, message
        );
        match message {
            Some(Message::StartBattle) => {
                let screen = self.start_battle(context)?;
                Ok(Transition::Push(screen))
            }
            Some(Message::Recruit(typename)) => {
                self.state.recruit(typename);
                let new_mode = self.state.mode();
                self.set_mode(context, new_mode)?;
                Ok(Transition::None)
            }
            Some(Message::Menu) => Ok(Transition::Pop),
            None => Ok(Transition::None),
        }
    }
}
