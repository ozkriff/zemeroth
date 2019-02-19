use ggez::{
    conf, event,
    graphics::{self, Font, Text},
    nalgebra::Point2,
    Context, ContextBuilder, GameResult,
};
use ggwp_zgui as ui;

#[derive(Clone, Copy, Debug)]
enum Message {
    Command1,
    Command2,
}

fn make_gui(context: &mut Context, font: Font) -> GameResult<ui::Gui<Message>> {
    let font_size = 64.0;
    let mut gui = ui::Gui::new(context);
    let text_1 = Box::new(Text::new(("[Button1]", font, font_size)));
    let text_2 = Box::new(Text::new(("[Button2]", font, font_size)));
    let button_1 = ui::Button::new(context, text_1, 0.2, gui.sender(), Message::Command1);
    let button_2 = ui::Button::new(context, text_2, 0.2, gui.sender(), Message::Command2);
    let mut layout = ui::VLayout::new();
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
    fn new(context: &mut Context) -> GameResult<State> {
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
    }
}

fn context() -> GameResult<(Context, event::EventsLoop)> {
    let name = file!();
    let window_conf = conf::WindowSetup::default().title(name);
    let window_mode = conf::WindowMode::default().resizable(true);
    ContextBuilder::new(name, "ozkriff")
        .window_setup(window_conf)
        .window_mode(window_mode)
        .add_resource_path("resources")
        .build()
}

fn main() -> GameResult {
    let (mut context, mut events_loop) = context()?;
    let mut state = State::new(&mut context)?;
    event::run(&mut context, &mut events_loop, &mut state)
}
