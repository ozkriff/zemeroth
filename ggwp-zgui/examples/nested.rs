use cgmath::Point2;
use ggwp_zgui as ui;
use gwg::{
    conf, event,
    graphics::{self, Font, Image, Rect, Text},
    Context, GameResult,
};

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
fn make_gui(context: &mut Context, font: Font) -> ui::Result<ui::Gui<Message>> {
    let font_size = 32.0;
    let mut gui = ui::Gui::new(context);
    {
        let image = Box::new(Image::new(context, "/fire.png")?);
        let button = ui::Button::new(context, image, 0.1, gui.sender(), Message::Image)?;
        let anchor = ui::Anchor(ui::HAnchor::Right, ui::VAnchor::Top);
        gui.add(&ui::pack(button), anchor);
    }
    {
        let text = Box::new(Text::new(("label", font, font_size)));
        let label = ui::Label::new_with_bg(context, text, 0.1)?;
        let anchor = ui::Anchor(ui::HAnchor::Left, ui::VAnchor::Bottom);
        gui.add(&ui::pack(label), anchor);
    }
    let v_layout_1 = {
        let text_a = Box::new(Text::new(("A", font, font_size)));
        let text_b = Box::new(Text::new(("A", font, font_size)));
        let text_c = Box::new(Text::new(("A", font, font_size)));
        let button_a = ui::Button::new(context, text_a, 0.1, gui.sender(), Message::A)?;
        let button_b = ui::Button::new(context, text_b, 0.1, gui.sender(), Message::B)?;
        let button_c = ui::Button::new(context, text_c, 0.1, gui.sender(), Message::C)?;
        let mut layout = ui::VLayout::new();
        layout.add(Box::new(button_a));
        layout.add(Box::new(button_b));
        layout.add(Box::new(button_c));
        layout
    };
    let v_layout_2 = {
        let image_i = Box::new(Image::new(context, "/fire.png")?);
        let text_x = Box::new(Text::new(("X", font, font_size)));
        let text_y = Box::new(Text::new(("Y", font, font_size)));
        let text_z = Box::new(Text::new(("Z", font, font_size)));
        let button_i = ui::Button::new(context, image_i, 0.1, gui.sender(), Message::Image)?;
        let button_x = ui::Button::new(context, text_x, 0.1, gui.sender(), Message::X)?;
        let button_y = ui::Button::new(context, text_y, 0.1, gui.sender(), Message::Y)?;
        let button_z = ui::Button::new(context, text_z, 0.1, gui.sender(), Message::Z)?;
        let mut layout = ui::VLayout::new();
        layout.add(Box::new(button_i));
        layout.add(Box::new(button_x));
        layout.add(Box::new(button_y));
        layout.add(Box::new(button_z));
        layout
    };
    {
        let text_a = Box::new(Text::new(("A", font, font_size)));
        let text_b = Box::new(Text::new(("A", font, font_size)));
        let image_i = Box::new(Image::new(context, "/fire.png")?);
        let button_a = ui::Button::new(context, text_a, 0.1, gui.sender(), Message::A)?;
        let button_b = ui::Button::new(context, text_b, 0.1, gui.sender(), Message::B)?;
        let button_i = ui::Button::new(context, image_i, 0.2, gui.sender(), Message::Image)?;
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
    fn new(context: &mut Context) -> ui::Result<State> {
        let font = Font::new(context, "/Karla-Regular.ttf")?;
        let gui = make_gui(context, font)?;
        Ok(State { gui })
    }

    fn resize(&mut self, context: &mut Context, w: f32, h: f32) -> GameResult {
        let aspect_ratio = w / h;
        let coordinates = Rect::new(-aspect_ratio, -1.0, aspect_ratio * 2.0, 2.0);
        graphics::set_screen_coordinates(context, coordinates)?;
        self.gui.resize(aspect_ratio);
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
        self.gui.draw(context)?;
        graphics::present(context)
    }

    fn resize_event(&mut self, context: &mut Context, w: f32, h: f32) {
        self.resize(context, w, h).expect("Can't resize the window");
    }

    fn mouse_button_up_event(
        &mut self,
        context: &mut Context,
        _: gwg::event::MouseButton,
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
