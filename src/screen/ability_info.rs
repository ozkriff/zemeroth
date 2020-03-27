use std::time::Duration;

use gwg::{
    graphics::{self, Text},
    Context,
};
use nalgebra::Point2;
use ui::{self, Gui};

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
        let h = utils::line_heights().big;
        let font_size = utils::font_size();
        let mut layout = ui::VLayout::new();
        let mut label = |text: &str| -> ZResult<Box<dyn ui::Widget>> {
            let text = Box::new(Text::new((text, font, font_size)));
            Ok(Box::new(ui::Label::new(context, text, h)?))
        };
        let spacer = || Box::new(ui::Spacer::new_vertical(h * 0.5));
        let mut add = |w| layout.add(w);
        match ability {
            ActiveOrPassiveAbility::Active(ability) => {
                add(label(&ability.title())?);
                add(spacer());
                for line in ability.extended_description() {
                    add(label(&line)?);
                }
            }
            ActiveOrPassiveAbility::Passive(ability) => {
                add(label(&ability.title())?);
                add(spacer());
                for line in ability.extended_description() {
                    add(label(&line)?);
                }
            }
        }
        add(spacer());
        {
            let text = Box::new(Text::new(("back", font, font_size)));
            let button = ui::Button::new(context, text, h, gui.sender(), Message::Back)?;
            add(Box::new(button));
        }
        let layout = utils::wrap_widget_and_add_bg(context, Box::new(layout))?;
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

    fn click(&mut self, _: &mut Context, pos: Point2<f32>) -> ZResult<StackCommand> {
        let message = self.gui.click(pos);
        match message {
            Some(Message::Back) => Ok(StackCommand::Pop),
            None => Ok(StackCommand::None),
        }
    }

    fn resize(&mut self, aspect_ratio: f32) {
        self.gui.resize(aspect_ratio);
    }

    fn move_mouse(&mut self, _context: &mut Context, pos: Point2<f32>) -> ZResult {
        self.gui.move_mouse(pos);
        Ok(())
    }
}
