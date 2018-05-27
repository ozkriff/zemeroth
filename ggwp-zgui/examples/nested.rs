extern crate ggez;
extern crate ggwp_zgui as ui;

use ggez::conf;
use ggez::event;
use ggez::graphics::{self, Font, Image, Point2, Rect, Text};
use ggez::{Context, ContextBuilder, GameResult};

#[derive(Clone, Copy, Debug)]
enum Message {
    A,
    B,
    C,
    Image,
    X,
    Y,
    Z,
}

// TODO: rework this into some more game-like
fn make_gui(context: &mut Context, font: &Font) -> GameResult<ui::Gui<Message>> {
    let mut gui = ui::Gui::new(context);
    {
        let image = Image::new(context, "/dragon1.png")?;
        let button = ui::Button::new(image, 0.1, gui.sender(), Message::Image);
        let anchor = ui::Anchor(ui::HAnchor::Right, ui::VAnchor::Top);
        gui.add(&ui::pack(button), anchor);
    }
    {
        let image = Text::new(context, "[label]", font)?.into_inner();
        let label = ui::Label::new(image, 0.1);
        let anchor = ui::Anchor(ui::HAnchor::Left, ui::VAnchor::Bottom);
        gui.add(&ui::pack(label), anchor);
    }
    let v_layout_1 = {
        let image_a = Text::new(context, "[A]", font)?.into_inner();
        let image_b = Text::new(context, "[B]", font)?.into_inner();
        let image_c = Text::new(context, "[C]", font)?.into_inner();
        let button_a = ui::Button::new(image_a, 0.1, gui.sender(), Message::A);
        let button_b = ui::Button::new(image_b, 0.1, gui.sender(), Message::B);
        let button_c = ui::Button::new(image_c, 0.1, gui.sender(), Message::C);
        let mut layout = ui::VLayout::new();
        layout.add(Box::new(button_a));
        layout.add(Box::new(button_b));
        layout.add(Box::new(button_c));
        layout
    };
    let v_layout_2 = {
        let image_i = Image::new(context, "/player.png")?;
        let image_x = Text::new(context, "[X]", font)?.into_inner();
        let image_y = Text::new(context, "[Y]", font)?.into_inner();
        let image_z = Text::new(context, "[Z]", font)?.into_inner();
        let button_i = ui::Button::new(image_i, 0.1, gui.sender(), Message::Image);
        let button_x = ui::Button::new(image_x, 0.1, gui.sender(), Message::X);
        let button_y = ui::Button::new(image_y, 0.1, gui.sender(), Message::Y);
        let button_z = ui::Button::new(image_z, 0.1, gui.sender(), Message::Z);
        let mut layout = ui::VLayout::new();
        layout.add(Box::new(button_i));
        layout.add(Box::new(button_x));
        layout.add(Box::new(button_y));
        layout.add(Box::new(button_z));
        layout
    };
    {
        let image_a = Text::new(context, "[A]", font)?.into_inner();
        let image_b = Text::new(context, "[B]", font)?.into_inner();
        let image_i = Image::new(context, "/dragon1.png")?;
        let button_a = ui::Button::new(image_a, 0.1, gui.sender(), Message::A);
        let button_b = ui::Button::new(image_b, 0.1, gui.sender(), Message::B);
        let button_i = ui::Button::new(image_i, 0.2, gui.sender(), Message::Image);
        let mut layout = ui::HLayout::new();
        layout.add(Box::new(button_a));
        layout.add(Box::new(button_i));
        layout.add(Box::new(v_layout_1));
        layout.add(Box::new(v_layout_2));
        layout.add(Box::new(button_b));
        let anchor = ui::Anchor(ui::HAnchor::Right, ui::VAnchor::Bottom);
        gui.add(&ui::pack(layout), anchor);
    }
    Ok(gui)
}

struct State {
    gui: ui::Gui<Message>,
}

impl State {
    fn new(context: &mut Context) -> GameResult<State> {
        let font = graphics::Font::new(context, "/Karla-Regular.ttf", 32)?;
        let gui = make_gui(context, &font)?;
        Ok(State { gui })
    }

    fn resize(&mut self, context: &mut Context, w: u32, h: u32) {
        let aspect_ratio = w as f32 / h as f32;
        let coordinates = Rect::new(-aspect_ratio, -1.0, aspect_ratio * 2.0, 2.0);
        graphics::set_screen_coordinates(context, coordinates).unwrap();
        self.gui.resize(aspect_ratio);
    }
}

impl event::EventHandler for State {
    fn update(&mut self, _: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, context: &mut Context) -> GameResult<()> {
        graphics::clear(context);
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
    }
}

fn context() -> ggez::Context {
    let name = "ggwp_zgui example nested";
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
