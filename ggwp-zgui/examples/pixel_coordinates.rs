use cgmath::Point2;
use ggwp_zgui as ui;
use gwg::{
    conf, event,
    graphics::{self, Font, Text},
    Context, GameResult,
};

#[derive(Clone, Copy, Debug)]
enum Message {
    Command1,
    Command2,
}

fn make_gui(context: &mut Context, font: Font) -> ui::Result<ui::Gui<Message>> {
    let mut gui = ui::Gui::new(context);
    let text_1 = Box::new(Text::new(("Button1", font, 32.0)));
    let text_2 = Box::new(Text::new(("Button1", font, 64.0)));
    let button_1 = ui::Button::new(context, text_1, 0.2, gui.sender(), Message::Command1)?;
    let button_2 = ui::Button::new(context, text_2, 0.2, gui.sender(), Message::Command2)?;
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
    fn new(context: &mut Context) -> ui::Result<State> {
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
            let mode = graphics::DrawMode::fill();
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
