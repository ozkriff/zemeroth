use std::time::Duration;

use gwg::{
    graphics::{self, Point2, Text},
    Context,
};
use ui::{self, Gui, Widget};

use crate::{
    core::battle::ability::{Ability, PassiveAbility},
    screen::{Screen, StackCommand},
    utils, ZResult,
};

#[derive(Clone, Debug)]
enum Message {
    Back,
}

pub enum ActiveOrPassiveAbility {
    Active(Ability),
    Passive(PassiveAbility),
}

#[derive(Debug)]
pub struct AbilityInfo {
    font: graphics::Font,
    gui: Gui<Message>,
}

impl AbilityInfo {
    pub fn new(context: &mut Context, ability: ActiveOrPassiveAbility) -> ZResult<Self> {
        let font = utils::default_font(context);
        let mut gui = ui::Gui::new(context);
        let h = utils::line_heights().normal;
        let font_size = utils::font_size();
        let mut layout = Box::new(ui::VLayout::new().stretchable(true));
        let text_ = |s: &str| Box::new(Text::new((s, font, font_size)));
        let label_ = |context: &mut Context, text: &str| -> ZResult<_> {
            Ok(ui::Label::new(context, text_(text), h)?)
        };
        let label = |context: &mut Context, text: &str| -> ZResult<_> {
            Ok(Box::new(label_(context, text)?))
        };
        let label_s = |context: &mut Context, text: &str| -> ZResult<_> {
            Ok(Box::new(label_(context, text)?.stretchable(true)))
        };
        let spacer = || Box::new(ui::Spacer::new_vertical(h * 0.5));
        let title = |text| format!("~~~ {} ~~~", text);
        match ability {
            ActiveOrPassiveAbility::Active(ability) => {
                layout.add(label_s(context, &title(ability.title()))?);
                layout.add(spacer());
                for line in ability.extended_description() {
                    layout.add(label(context, &line)?);
                }
            }
            ActiveOrPassiveAbility::Passive(ability) => {
                layout.add(label_s(context, &title(ability.title()))?);
                layout.add(spacer());
                for line in ability.extended_description() {
                    layout.add(label(context, &line)?);
                }
            }
        }
        layout.add(spacer());
        {
            let mut button =
                ui::Button::new(context, text_("back"), h, gui.sender(), Message::Back)?
                    .stretchable(true);
            button.stretch(context, layout.rect().w / 3.0)?;
            button.set_stretchable(false);
            layout.add(Box::new(button));
        }
        layout.stretch_to_self(context)?;
        let layout = utils::add_offsets_and_bg_big(context, layout)?;
        let anchor = ui::Anchor(ui::HAnchor::Middle, ui::VAnchor::Middle);
        gui.add(&ui::pack(layout), anchor);
        Ok(Self { font, gui })
    }
}

impl Screen for AbilityInfo {
    fn update(&mut self, _context: &mut Context, _dtime: Duration) -> ZResult<StackCommand> {
        Ok(StackCommand::None)
    }

    fn draw(&self, context: &mut Context) -> ZResult {
        self.gui.draw(context)?;
        Ok(())
    }

    fn click(&mut self, _: &mut Context, pos: Point2) -> ZResult<StackCommand> {
        let message = self.gui.click(pos);
        match message {
            Some(Message::Back) => Ok(StackCommand::Pop),
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
