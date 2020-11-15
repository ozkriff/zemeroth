use zgui as ui;

use macroquad::prelude::*; // TODO: expand

#[derive(Clone, Copy, Debug)]
enum Message {
    Command,
}

fn make_gui(font: Font) -> ui::Result<ui::Gui<Message>> {
    let font_size = 32;
    let mut gui = ui::Gui::new();
    let anchor = ui::Anchor(ui::HAnchor::Right, ui::VAnchor::Bottom);
    let text = ui::Drawable::text("Button", font, font_size);
    let button = ui::Button::new(text, 0.2, gui.sender(), Message::Command)?;
    gui.add(&ui::pack(button), anchor);
    Ok(gui)
}

struct State {
    gui: ui::Gui<Message>,
}

impl State {
    async fn new() -> ui::Result<State> {
        let font = load_ttf_font("./resources/Karla-Regular.ttf").await;
        let gui = make_gui(font)?;
        Ok(Self { gui })
    }

    fn resize(&mut self, w: f32, h: f32) {
        let aspect_ratio = w / h;
        self.gui.resize(aspect_ratio);
    }
}

#[macroquad::main("TextButton")]
async fn main() {
    let mut state = State::new().await.expect("Can't create the state");

    loop {
        clear_background(WHITE);

        state.resize(screen_width(), screen_height());

        state.gui.draw();

        if is_mouse_button_down(MouseButton::Left) {
            let (x, y) = mouse_position();
            let window_pos = vec2(x, y);
            let pos = ui::window_to_screen(window_pos);
            let message = state.gui.click(pos);
            println!("[{},{}] -> {:?}: {:?}", x, y, pos, message);
        }

        next_frame().await;
    }
}
