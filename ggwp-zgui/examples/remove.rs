extern crate ggez;
extern crate ggwp_zgui as ui;

use ggez::conf;
use ggez::event;
use ggez::graphics::{self, Font, Image, Point2, Text};
use ggez::{Context, ContextBuilder, GameResult};

#[derive(Clone, Copy, Debug)]
enum Message {
    AddOrRemove,
}

fn make_label(context: &mut Context) -> ui::RcWidget {
    let image = Image::new(context, "/fire.png").expect("Can't load test image");
    ui::pack(ui::Label::new(context, image, 0.5))
}

fn make_gui(context: &mut Context, font: &Font) -> GameResult<ui::Gui<Message>> {
    let mut gui = ui::Gui::new(context);
    let anchor = ui::Anchor(ui::HAnchor::Right, ui::VAnchor::Bottom);
    let image = Text::new(context, "[Add/Remove]", &font)?.into_inner();
    let button = ui::Button::new(context, image, 0.2, gui.sender(), Message::AddOrRemove);
    gui.add(&ui::pack(button), anchor);
    Ok(gui)
}

struct State {
    gui: ui::Gui<Message>,
    label: Option<ui::RcWidget>,
}

impl State {
    fn new(context: &mut Context) -> GameResult<State> {
        let font = graphics::Font::new(context, "/Karla-Regular.ttf", 32)?;
        let gui = make_gui(context, &font)?;
        Ok(Self { gui, label: None })
    }

    fn resize(&mut self, _: &mut Context, w: u32, h: u32) {
        let aspect_ratio = w as f32 / h as f32;
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
        let label = make_label(context);
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
        graphics::clear(context);
        graphics::set_background_color(context, [1.0, 1.0, 1.0, 1.0].into());
        self.gui.draw(context)?;
        graphics::present(context);
        Ok(())
    }

    fn resize_event(&mut self, context: &mut Context, w: u32, h: u32) {
        self.resize(context, w, h);
    }

    fn mouse_button_up_event(
        &mut self,
        context: &mut Context,
        _: ggez::event::MouseButton,
        x: i32,
        y: i32,
    ) {
        let window_pos = Point2::new(x as _, y as _);
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

fn context() -> GameResult<ggez::Context> {
    let name = "ggwp_zgui example text_button";
    let window_conf = conf::WindowSetup::default().resizable(true).title(name);
    ContextBuilder::new(name, "ozkriff")
        .window_setup(window_conf)
        .add_resource_path("resources")
        .build()
}

fn main() -> GameResult<()> {
    let mut context = context()?;
    let mut state = State::new(&mut context)?;
    event::run(&mut context, &mut state)
}
