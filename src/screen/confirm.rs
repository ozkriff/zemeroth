use std::{
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

use cgmath::Point2;
use gwg::{
    graphics::{self, Text},
    Context,
};
use ui::{self, Gui, Widget};

use crate::{
    screen::{Screen, StackCommand},
    utils, ZResult,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    Yes,
    No,
}

//// A helper function for a receiving side.
pub fn try_receive_yes(opt_rx: &Option<Receiver<Message>>) -> bool {
    utils::try_receive(opt_rx) == Some(Message::Yes)
}

#[derive(Debug)]
pub struct Confirm {
    font: graphics::Font,
    gui: Gui<Message>,
    sender: Sender<Message>,
}

impl Confirm {
    pub fn from_lines(
        context: &mut Context,
        lines: &[impl AsRef<str>],
        sender: Sender<Message>,
    ) -> ZResult<Self> {
        let font = utils::default_font(context);
        let h = utils::line_heights().big;
        let font_size = utils::font_size();
        let mut layout = ui::VLayout::new();
        for line in lines {
            let text = Box::new(Text::new((line.as_ref(), font, font_size)));
            let label = Box::new(ui::Label::new(context, text, h)?);
            layout.add(label);
        }
        Self::from_widget(context, Box::new(layout), sender)
    }

    pub fn from_line(context: &mut Context, line: &str, sender: Sender<Message>) -> ZResult<Self> {
        Self::from_lines(context, &[line], sender)
    }

    pub fn from_widget(
        context: &mut Context,
        widget: Box<dyn ui::Widget>,
        sender: Sender<Message>,
    ) -> ZResult<Self> {
        let font = utils::default_font(context);
        let mut gui = ui::Gui::new(context);
        let h = utils::line_heights().big;
        let font_size = utils::font_size();
        let mut layout = ui::VLayout::new();
        let spacer = || Box::new(ui::Spacer::new_vertical(h * 0.5));
        let mut add = |w| layout.add(w);
        let button = |context: &mut Context, line, message| {
            let text = Box::new(Text::new((line, font, font_size)));
            ui::Button::new(context, text, h, gui.sender(), message)
        };
        let yes = button(context, "yes", Message::Yes)?;
        let no = button(context, "no", Message::No)?;
        let spacer_width = widget.rect().w - yes.rect().w - no.rect().w;
        let mut line_layout = ui::HLayout::new();
        line_layout.add(Box::new(yes));
        line_layout.add(Box::new(ui::Spacer::new_horizontal(spacer_width)));
        line_layout.add(Box::new(no));
        add(widget);
        add(spacer());
        add(Box::new(line_layout));
        let pack_offset = 0.04;
        let layout = utils::pack_widget_into_offset_table(Box::new(layout), pack_offset);
        let layout = utils::wrap_widget_and_add_bg(context, layout)?;
        let anchor = ui::Anchor(ui::HAnchor::Middle, ui::VAnchor::Middle);
        gui.add(&ui::pack(layout), anchor);
        Ok(Self { font, gui, sender })
    }
}

// TODO: handle Enter/ESC keys
impl Screen for Confirm {
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
            Some(message) => {
                self.sender
                    .send(message)
                    .expect("Can't report back the result");
                Ok(StackCommand::Pop)
            }
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
