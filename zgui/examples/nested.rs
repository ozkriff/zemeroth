use mq::color::WHITE;
use zgui as ui;

mod common;

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

fn make_gui(assets: common::Assets) -> ui::Result<ui::Gui<Message>> {
    let font_size = 64;
    let text = |s| ui::Drawable::text(s, assets.font, font_size);
    let texture = || ui::Drawable::Texture(assets.texture);
    let mut gui = ui::Gui::new();
    {
        let button = ui::Button::new(texture(), 0.1, gui.sender(), Message::Image)?;
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
        let button_i = ui::Button::new(texture(), 0.1, gui.sender(), Message::Image)?;
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
        let button_i = ui::Button::new(texture(), 0.2, gui.sender(), Message::Image)?;
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

#[mq::main("ZGui: Nested Layouts Demo")]
#[macroquad(crate_rename = "mq")]
async fn main() {
    let assets = common::Assets::load().await.expect("Can't load assets");
    let mut gui = make_gui(assets).expect("Can't create the gui");
    loop {
        // Update the camera and the GUI.
        let aspect_ratio = common::aspect_ratio();
        let camera = common::make_and_set_camera(aspect_ratio);
        gui.resize_if_needed(aspect_ratio);
        // Handle cursor updates.
        let pos = common::get_world_mouse_pos(&camera);
        gui.move_mouse(pos);
        if mq::input::is_mouse_button_pressed(mq::input::MouseButton::Left) {
            let message = gui.click(pos);
            println!("{:?}", message);
        }
        // Draw the GUI.
        mq::window::clear_background(WHITE);
        gui.draw();
        mq::window::next_frame().await;
    }
}
