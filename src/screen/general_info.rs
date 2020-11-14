use std::time::Duration;

use macroquad::prelude::{Color, Font, Vec2};

use ui::{self, Gui, Widget};

use crate::{
    assets,
    screen::{Screen, StackCommand},
    utils, ZResult,
};

#[derive(Clone, Debug)]
enum Message {
    Back,
}

#[derive(Debug)]
pub struct GeneralInfo {
    font: Font,
    gui: Gui<Message>,
}

impl GeneralInfo {
    pub fn new(title: &str, lines: &[String]) -> ZResult<Self> {
        let font = assets::get().font;
        let mut gui = ui::Gui::new();
        let h = utils::line_heights().normal;
        let font_size = utils::font_size();
        let mut layout = Box::new(ui::VLayout::new().stretchable(true));
        let text_ = |s: &str| ui::Drawable::text(s, font, font_size);
        let label_ = |text: &str| -> ZResult<_> { Ok(ui::Label::new(text_(text), h)?) };
        let label = |text: &str| -> ZResult<_> { Ok(Box::new(label_(text)?)) };
        let label_s = |text: &str| -> ZResult<_> { Ok(Box::new(label_(text)?.stretchable(true))) };
        let spacer = || Box::new(ui::Spacer::new_vertical(h * 0.5));
        layout.add(label_s(&format!("~~~ {} ~~~", title))?);
        layout.add(spacer());
        for line in lines {
            layout.add(label(&line)?);
        }
        layout.add(spacer());
        {
            let mut button =
                ui::Button::new(text_("back"), h, gui.sender(), Message::Back)?.stretchable(true);
            button.stretch(layout.rect().w / 3.0)?;
            button.set_stretchable(false);
            layout.add(Box::new(button));
        }
        layout.stretch_to_self()?;
        let layout = utils::add_offsets_and_bg_big(layout)?;
        let anchor = ui::Anchor(ui::HAnchor::Middle, ui::VAnchor::Middle);
        gui.add(&ui::pack(layout), anchor);
        Ok(Self { font, gui })
    }
}

impl Screen for GeneralInfo {
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
            Some(Message::Back) => Ok(StackCommand::Pop),
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
