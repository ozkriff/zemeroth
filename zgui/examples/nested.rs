use macroquad::{
    self as mq,
    camera::{set_camera, Camera2D},
    prelude::{Rect, Vec2, WHITE},
    text::{load_ttf_font, Font},
    texture::{self, Texture2D},
};
use zgui as ui;

fn aspect_ratio() -> f32 {
    mq::window::screen_width() / mq::window::screen_height()
}

fn make_and_set_camera(aspect_ratio: f32) -> Camera2D {
    let display_rect = Rect {
        x: -aspect_ratio,
        y: -1.0,
        w: aspect_ratio * 2.0,
        h: 2.0,
    };
    let camera = Camera2D::from_display_rect(display_rect);
    set_camera(camera);
    camera
}

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

struct Assets {
    font: Font,
    texture: Texture2D,
}

impl Assets {
    async fn load() -> Self {
        let font = load_ttf_font("zgui/assets/Karla-Regular.ttf").await;
        let texture = texture::load_texture("zgui/assets/fire.png").await;
        Self { font, texture }
    }
}

fn make_gui(assets: Assets) -> ui::Result<ui::Gui<Message>> {
    let font_size = 64;
    let text = |s| ui::Drawable::text(s, assets.font, font_size);
    let image = || ui::Drawable::Texture(assets.texture);
    let mut gui = ui::Gui::new();
    {
        let button = ui::Button::new(image(), 0.1, gui.sender(), Message::Image)?;
        let anchor = ui::Anchor(ui::HAnchor::Right, ui::VAnchor::Top);
        gui.add(&ui::pack(button), anchor);
    }
    {
        let label = ui::Label::new_with_bg(text("label"), 0.1)?;
        let anchor = ui::Anchor(ui::HAnchor::Left, ui::VAnchor::Bottom);
        gui.add(&ui::pack(label), anchor);
    }
    let v_layout_1 = {
        let button_a = ui::Button::new(text("A"), 0.1, gui.sender(), Message::A)?;
        let button_b = ui::Button::new(text("B"), 0.1, gui.sender(), Message::B)?;
        let button_c = ui::Button::new(text("C"), 0.1, gui.sender(), Message::C)?;
        let mut layout = ui::VLayout::new();
        layout.add(Box::new(button_a));
        layout.add(Box::new(button_b));
        layout.add(Box::new(button_c));
        layout
    };
    let v_layout_2 = {
        let button_i = ui::Button::new(image(), 0.1, gui.sender(), Message::Image)?;
        let button_x = ui::Button::new(text("X"), 0.1, gui.sender(), Message::X)?;
        let button_y = ui::Button::new(text("Y"), 0.1, gui.sender(), Message::Y)?;
        let button_z = ui::Button::new(text("Z"), 0.1, gui.sender(), Message::Z)?;
        let mut layout = ui::VLayout::new();
        layout.add(Box::new(button_i));
        layout.add(Box::new(button_x));
        layout.add(Box::new(button_y));
        layout.add(Box::new(button_z));
        layout
    };
    {
        let button_a = ui::Button::new(text("A"), 0.1, gui.sender(), Message::A)?;
        let button_b = ui::Button::new(text("B"), 0.1, gui.sender(), Message::B)?;
        let button_i = ui::Button::new(image(), 0.2, gui.sender(), Message::Image)?;
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

#[macroquad::main("ZGui: Nested Layouts Demo")]
async fn main() {
    let assets = Assets::load().await;
    let mut gui = make_gui(assets).expect("TODO: err msg");
    loop {
        // Update the camera and the GUI.
        let aspect_ratio = aspect_ratio();
        let camera = make_and_set_camera(aspect_ratio);
        gui.resize(aspect_ratio);
        // Handle cursor updates.
        let (x, y) = mq::input::mouse_position();
        let window_pos = Vec2::new(x, y);
        let pos = camera.screen_to_world(window_pos);
        gui.move_mouse(pos);
        if mq::input::is_mouse_button_pressed(mq::input::MouseButton::Left) {
            let message = gui.click(pos);
            println!("[{},{}] -> {:?}: {:?}", x, y, pos, message);
        }
        // Draw the GUI.
        mq::window::clear_background(WHITE);
        gui.draw();
        mq::window::next_frame().await;
    }
}
