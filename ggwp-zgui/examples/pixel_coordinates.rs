extern crate ggez;
extern crate ggwp_zgui as ui;

use ggez::conf;
use ggez::event;
use ggez::graphics::{self, Font, Point2, Text};
use ggez::{Context, ContextBuilder, GameResult};

#[derive(Clone, Copy, Debug)]
enum Message {
    Command1,
    Command2,
}

fn make_gui(context: &mut Context, font: &Font) -> GameResult<ui::Gui<Message>> {
    let mut gui = ui::Gui::new(context);
    let image_1 = Text::new(context, "[Button1]", font)?.into_inner();
    let image_2 = Text::new(context, "[Button2]", font)?.into_inner();
    let button_1 = ui::Button::new(image_1, 0.2, gui.sender(), Message::Command1);
    let button_2 = ui::Button::new(image_2, 0.2, gui.sender(), Message::Command2);
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
        let (w, h) = graphics::get_drawable_size(context);
        let font = graphics::Font::new(context, "/Karla-Regular.ttf", 32)?;
        let gui = make_gui(context, &font)?;
        let mut this = State { gui };
        this.resize(context, w, h)?;
        Ok(this)
    }

    fn resize(&mut self, context: &mut Context, w: u32, h: u32) -> GameResult<()> {
        let aspect_ratio = w as f32 / h as f32;
        self.gui.resize(aspect_ratio);
        let rect = graphics::Rect::new(0.0, 0.0, w as f32, h as f32);
        graphics::set_screen_coordinates(context, rect)?;
        Ok(())
    }

    fn draw_scene(&self, context: &mut Context) -> GameResult<()> {
        let pos = Point2::new(150.0, 150.0);
        graphics::circle(context, ggez::graphics::DrawMode::Fill, pos, 100.0, 2.0)?;
        self.gui.draw(context)?;
        Ok(())
    }
}

impl event::EventHandler for State {
    fn update(&mut self, _: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, context: &mut Context) -> GameResult<()> {
        graphics::clear(context);
        self.draw_scene(context)?;
        self.gui.draw(context)?;
        graphics::present(context);
        Ok(())
    }

    fn resize_event(&mut self, context: &mut Context, w: u32, h: u32) {
        self.resize(context, w, h).expect("Can't resize the window");
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
    }
}

fn context() -> ggez::Context {
    let name = "ggwp_zgui example pixel_coordinates";
    let window_conf = conf::WindowSetup::default().resizable(true).title(name);
    ContextBuilder::new(name, "ozkriff")
        .window_setup(window_conf)
        .add_resource_path("resources")
        .build()
        .expect("Can't build context")
}

fn main() -> GameResult<()> {
    let mut context = context();
    let mut state = State::new(&mut context).expect("Can't create state");
    event::run(&mut context, &mut state)
}
