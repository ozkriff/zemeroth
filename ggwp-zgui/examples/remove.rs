use ggez::{
    conf, event,
    graphics::{self, Font, Image, Text},
    Context, GameResult,
};
use ggwp_zgui as ui;
use nalgebra::Point2;

#[derive(Clone, Copy, Debug)]
enum Message {
    AddOrRemove,
}

fn make_label(context: &mut Context) -> ui::Result<ui::RcWidget> {
    let image = Image::new(context, "/fire.png").expect("Can't load test image");
    let label = ui::Label::new_with_bg(context, Box::new(image), 0.5)?;
    Ok(ui::pack(label))
}

fn make_gui(context: &mut Context, font: Font) -> ui::Result<ui::Gui<Message>> {
    let font_size = 32.0;
    let mut gui = ui::Gui::new(context);
    let anchor = ui::Anchor(ui::HAnchor::Right, ui::VAnchor::Bottom);
    let text = Box::new(Text::new(("Add/Remove", font, font_size)));
    let button = ui::Button::new(context, text, 0.2, gui.sender(), Message::AddOrRemove)?;
    gui.add(&ui::pack(button), anchor);
    Ok(gui)
}

struct State {
    gui: ui::Gui<Message>,
    label: Option<ui::RcWidget>,
}

impl State {
    fn new(context: &mut Context) -> ui::Result<State> {
        let font = Font::new(context, "/Karla-Regular.ttf")?;
        let gui = make_gui(context, font)?;
        Ok(Self { gui, label: None })
    }

    fn resize(&mut self, _: &mut Context, w: f32, h: f32) {
        let aspect_ratio = w / h;
        self.gui.resize(aspect_ratio);
    }

    fn remove_label(&mut self) {
        println!("Removing...");
        if let Some(ref label) = self.label {
            self.gui.remove(label).expect("Can't remove the label");
        }
        self.label = None;
        println!("Removed.");
    }

    fn add_label(&mut self, context: &mut Context) {
        println!("Adding...");
        let label = make_label(context).expect("Can't make a label");
        let anchor = ui::Anchor(ui::HAnchor::Left, ui::VAnchor::Top);
        self.gui.add(&label, anchor);
        self.label = Some(label);
        println!("Added.");
    }
}

impl event::EventHandler for State {
    fn update(&mut self, _: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, context: &mut Context) -> GameResult<()> {
        let bg_color = [1.0, 1.0, 1.0, 1.0].into();
        graphics::clear(context, bg_color);
        self.gui.draw(context)?;
        graphics::present(context)
    }

    fn resize_event(&mut self, context: &mut Context, w: f32, h: f32) {
        self.resize(context, w, h);
    }

    fn mouse_button_up_event(
        &mut self,
        context: &mut Context,
        _: ggez::event::MouseButton,
        x: f32,
        y: f32,
    ) {
        let window_pos = Point2::new(x, y);
        let pos = ui::window_to_screen(context, window_pos);
        let message = self.gui.click(pos);
        println!("[{},{}] -> {}: {:?}", x, y, pos, message);
        if let Some(Message::AddOrRemove) = message {
            if self.label.is_some() {
                self.remove_label();
            } else {
                self.add_label(context);
            }
        }
    }
}

fn main() -> ggez::GameResult {
    ggez::start(
        conf::Conf {
            physical_root_dir: Some("resources".into()),
            ..Default::default()
        },
        |mut context| Box::new(State::new(&mut context).expect("Can't create the state")),
    )
}
