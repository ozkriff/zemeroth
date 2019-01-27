use ggez::{
    conf, event,
    nalgebra::Point2,
    graphics::{self, Font, Text},
    Context, ContextBuilder, GameResult,
};
use ggwp_zgui as ui;

#[derive(Clone, Copy, Debug)]
enum Message {
    Command1,
    Command2,
}

fn make_gui(context: &mut Context, font: Font) -> GameResult<ui::Gui<Message>> {
    let mut gui = ui::Gui::new(context);
    let text_1 = Box::new(Text::new(("[Button1]", font, 32.0)));
    let text_2 = Box::new(Text::new(("[Button1]", font, 64.0)));
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
        let (w, h) = graphics::drawable_size(context);
        let font = Font::new(context, "/Karla-Regular.ttf")?;
        let gui = make_gui(context, font)?;
        let mut this = State { gui };
        this.resize(context, w as _, h as _)?;
        Ok(this)
    }

    fn resize(&mut self, context: &mut Context, w: f32, h: f32) -> GameResult {
        let aspect_ratio = w / h;
        self.gui.resize(aspect_ratio);
        let rect = graphics::Rect::new(0.0, 0.0, w, h);
        graphics::set_screen_coordinates(context, rect)?;
        Ok(())
    }

    fn draw_scene(&self, context: &mut Context) -> GameResult {
        let circle = {
            let mode = graphics::DrawMode::Fill;
            let pos = Point2::new(150.0, 150.0);
            let radius = 100.0;
            let tolerance = 2.0;
            let color = [0.5, 0.5, 0.5, 1.0].into();
            graphics::Mesh::new_circle(context, mode, pos, radius, tolerance, color)?
        };
        let param = graphics::DrawParam::new();
        graphics::draw(context, &circle, param)?;
        Ok(())
    }
}

impl event::EventHandler for State {
    fn update(&mut self, _: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, context: &mut Context) -> GameResult {
        let bg_color = [1.0, 1.0, 1.0, 1.0].into();
        graphics::clear(context, bg_color);
        self.draw_scene(context)?;
        self.gui.draw(context)?;
        graphics::present(context)
    }

    fn resize_event(&mut self, context: &mut Context, w: f32, h: f32) {
        self.resize(context, w, h).expect("Can't resize the window");
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
        println!("[{},{}] -> {}: {:?}", x, y, pos, message);
    }
}

fn context() -> GameResult<(Context, event::EventsLoop)> {
    let name = file!();
    let window_conf = conf::WindowSetup::default()
        .title(name);
    let window_mode = conf::WindowMode::default()
        .resizable(true);
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
