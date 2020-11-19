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
    Command,
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
    let mut gui = ui::Gui::new();
    let text = ui::Drawable::text(" text", assets.font, font_size);
    let texture = ui::Drawable::Texture(assets.texture);
    let button_1 = ui::Button::new(texture, 0.2, gui.sender(), Message::Command)?;
    let button_2 = ui::Label::new(text, 0.1)?;
    let mut layout = ui::LayersLayout::new();
    layout.add(Box::new(button_1));
    layout.add(Box::new(button_2));
    let anchor = ui::Anchor(ui::HAnchor::Right, ui::VAnchor::Bottom);
    gui.add(&ui::pack(layout), anchor);
    Ok(gui)
}

#[macroquad::main("ZGui: Text Button Demo")]
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
