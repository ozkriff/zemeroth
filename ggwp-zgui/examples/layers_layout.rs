use ggez::{
    conf, event,
    graphics::{self, Font, Image, Text},
    Context, GameResult,
};
use ggwp_zgui as ui;
use nalgebra::Point2;

#[derive(Clone, Copy, Debug)]
enum Message {
    Command1,
}

fn make_gui(context: &mut Context, font: Font) -> ui::Result<ui::Gui<Message>> {
    let font_size = 64.0;
    let mut gui = ui::Gui::new(context);
    let text = Box::new(Text::new(("text", font, font_size)));
    let image = Box::new(Image::new(context, "/fire.png")?);
    let button_1 = ui::Button::new(context, image, 0.2, gui.sender(), Message::Command1)?;
    let button_2 = ui::Label::new(context, text, 0.1)?;
    let mut layout = ui::LayersLayout::new();
    layout.add(Box::new(button_1));
    layout.add(Box::new(button_2));
    let anchor = ui::Anchor(ui::HAnchor::Right, ui::VAnchor::Bottom);
    gui.add(&ui::pack(layout), anchor);
    Ok(gui)
}

struct State {
    gui: ui::Gui<Message>,
}

impl State {
    fn new(context: &mut Context) -> ui::Result<State> {
        let font = Font::new(context, "/Karla-Regular.ttf")?;
        let gui = make_gui(context, font)?;
        Ok(Self { gui })
    }

    fn resize(&mut self, _: &mut Context, w: f32, h: f32) {
        let aspect_ratio = w / h;
        self.gui.resize(aspect_ratio);
    }
}

impl event::EventHandler for State {
    fn update(&mut self, _: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, context: &mut Context) -> GameResult {
        let bg_color = [1.0, 1.0, 1.0, 1.0].into();
        graphics::clear(context, bg_color);
        self.gui.draw(context)?;
        graphics::present(context)
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
    }

    fn resize_event(&mut self, context: &mut Context, w: f32, h: f32) {
        self.resize(context, w, h);
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
