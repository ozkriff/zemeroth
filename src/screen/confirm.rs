use std::{
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

use macroquad::prelude::{Color, Font, Vec2};
use ui::{self, Gui, Widget};

use crate::{
    assets,
    screen::{Screen, StackCommand},
    utils, ZResult,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    Yes,
    No,
}

/// A helper function for a receiving side.
pub fn try_receive_yes(opt_rx: &Option<Receiver<Message>>) -> bool {
    utils::try_receive(opt_rx) == Some(Message::Yes)
}

#[derive(Debug)]
pub struct Confirm {
    gui: Gui<Message>,
    sender: Sender<Message>,
}

impl Confirm {
    pub fn from_lines(lines: &[impl AsRef<str>], sender: Sender<Message>) -> ZResult<Self> {
        let font = assets::get().font;
        let h = utils::line_heights().big;
        let font_size = utils::font_size();
        let mut layout = ui::VLayout::new();
        for line in lines {
            let text = ui::Drawable::text(line.as_ref(), font, font_size);
            let label = Box::new(ui::Label::new(text, h)?);
            layout.add(label);
        }
        Self::from_widget(Box::new(layout), sender)
    }

    pub fn from_line(line: &str, sender: Sender<Message>) -> ZResult<Self> {
        Self::from_lines(&[line], sender)
    }

    pub fn from_widget(widget: Box<dyn ui::Widget>, sender: Sender<Message>) -> ZResult<Self> {
        let font = assets::get().font;
        let mut gui = ui::Gui::new();
        let h = utils::line_heights().big;
        let font_size = utils::font_size();
        let mut layout = Box::new(ui::VLayout::new());
        let spacer = || Box::new(ui::Spacer::new_vertical(h * 0.5));
        let button = |line, message| -> ZResult<_> {
            let text = ui::Drawable::text(line, font, font_size);
            let b = ui::Button::new(text, h, gui.sender(), message)?.stretchable(true);
            Ok(b)
        };
        let button_width = widget.rect().w / 3.0;
        let mut yes = button("yes", Message::Yes)?;
        yes.stretch(button_width)?;
        let mut no = button("no", Message::No)?;
        no.stretch(button_width)?;
        let spacer_width = widget.rect().w - yes.rect().w - no.rect().w;
        let mut line_layout = ui::HLayout::new();
        line_layout.add(Box::new(yes));
        line_layout.add(Box::new(ui::Spacer::new_horizontal(spacer_width)));
        line_layout.add(Box::new(no));
        layout.add(widget);
        layout.add(spacer());
        layout.add(Box::new(line_layout));
        let layout = utils::add_offsets_and_bg_big(layout)?;
        let anchor = ui::Anchor(ui::HAnchor::Middle, ui::VAnchor::Middle);
        gui.add(&ui::pack(layout), anchor);
        Ok(Self { gui, sender })
    }
}

// TODO: handle Enter/ESC keys
impl Screen for Confirm {
    fn update(&mut self, _dtime: Duration) -> ZResult<StackCommand> {
        Ok(StackCommand::None)
    }

    fn draw(&self) -> ZResult {
        self.gui.draw();
        Ok(())
    }

    fn click(&mut self, pos: Vec2) -> ZResult<StackCommand> {
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

    fn move_mouse(&mut self, pos: Vec2) -> ZResult {
        self.gui.move_mouse(pos);
        Ok(())
    }
}
