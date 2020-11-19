use macroquad::{
    self as mq,
    camera::{set_camera, Camera2D},
    prelude::{Rect, Vec2, WHITE},
    text::{load_ttf_font, Font},
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

fn make_gui(font: Font) -> ui::Result<ui::Gui<Message>> {
    let font_size = 64;
    let mut gui = ui::Gui::new();
    let anchor = ui::Anchor(ui::HAnchor::Right, ui::VAnchor::Bottom);
    let text = ui::Drawable::text("Button", font, font_size);
    let button = ui::Button::new(text, 0.2, gui.sender(), Message::Command)?;
    gui.add(&ui::pack(button), anchor);
    Ok(gui)
}

#[macroquad::main("ZGui: Text Button Demo")]
async fn main() {
    let font = load_ttf_font("zgui/assets/Karla-Regular.ttf").await;
    let mut gui = make_gui(font).expect("TODO: err msg");
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
