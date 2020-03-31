use cgmath::Point2;
use gwg::{
    conf, event,
    graphics::{self, Font, Text},
    Context, GameResult,
};
use zgui as ui;

#[derive(Clone, Copy, Debug)]
enum Message {
    Command,
}

fn make_gui(context: &mut Context, font: Font) -> ui::Result<ui::Gui<Message>> {
    let font_size = 32.0;
    let mut gui = ui::Gui::new(context);
    let anchor = ui::Anchor(ui::HAnchor::Right, ui::VAnchor::Bottom);
    let text = Box::new(Text::new(("Button", font, font_size)));
    let button = ui::Button::new(context, text, 0.2, gui.sender(), Message::Command)?;
    gui.add(&ui::pack(button), anchor);
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

    fn resize_event(&mut self, context: &mut Context, w: f32, h: f32) {
        self.resize(context, w, h);
    }

    fn mouse_button_up_event(
        &mut self,
        context: &mut Context,
        _: event::MouseButton,
        x: f32,
        y: f32,
    ) {
        let window_pos = Point2::new(x, y);
        let pos = ui::window_to_screen(context, window_pos);
        let message = self.gui.click(pos);
        println!("[{},{}] -> {:?}: {:?}", x, y, pos, message);
    }
}

fn main() -> gwg::GameResult {
    gwg::start(
        conf::Conf {
            physical_root_dir: Some("resources".into()),
            ..Default::default()
        },
        |mut context| Box::new(State::new(&mut context).expect("Can't create the state")),
    )
}
