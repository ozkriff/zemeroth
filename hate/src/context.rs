use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::path::{Path, PathBuf};
use std::time;
use cgmath::{self, Vector2, Matrix4, SquareMatrix, Zero, InnerSpace};
use glutin::{self, Api, MouseButton};
use glutin::ElementState::{Pressed, Released};
use rusttype;
use gfx::traits::{FactoryExt, Device};
use gfx::handle::Program;
use gfx;
use gfx_device_gl;
use gfx_window_glutin;
use ::{Settings, Time, Event};
use ::{screen, fs};
use geom::{Size, Point};
use pipeline::pipe;
use mesh::Mesh;
use texture::{self, Texture};

fn shader_version_string(api: Api) -> String {
    match api {
        Api::OpenGl => "#version 120\n".into(),
        Api::OpenGlEs | Api::WebGl => "#version 100\n".into(),
    }
}

fn vertex_shader(api: Api) -> String {
    shader_version_string(api) + &fs::load_as_string("shader/v.glsl")
}

fn fragment_shader(api: Api) -> String {
    let mut text = shader_version_string(api);
    if api == Api::OpenGlEs || api == Api::WebGl {
        text += "precision mediump float;\n";
    }
    text + &fs::load_as_string("shader/f.glsl")
}

fn new_shader(
    window: &glutin::Window,
    factory: &mut gfx_device_gl::Factory,
) -> Program<gfx_device_gl::Resources> {
    let api = window.get_api();
    let vs = vertex_shader(api);
    let fs = fragment_shader(api);
    factory.link_program(vs.as_bytes(), fs.as_bytes()).unwrap()
}

fn new_pso(
    factory: &mut gfx_device_gl::Factory,
    program: &Program<gfx_device_gl::Resources>,
    primitive: gfx::Primitive,
) -> gfx::PipelineState<gfx_device_gl::Resources, pipe::Meta> {
    let rasterizer = gfx::state::Rasterizer::new_fill();
    let pso = factory.create_pipeline_from_program(program, primitive, rasterizer, pipe::new());
    pso.unwrap()
}

fn new_font<P: AsRef<Path>>(path: P) -> rusttype::Font<'static> {
    let collection = rusttype::FontCollection::from_bytes(fs::load(path));
    collection.into_font().unwrap()
}

fn new_window_builder() -> glutin::WindowBuilder<'static> {
    let gl_version = glutin::GlRequest::GlThenGles {
        opengles_version: (2, 0),
        opengl_version: (2, 1),
    };
    glutin::WindowBuilder::new()
        .with_title("Zemeroth".to_string())
        .with_pixel_format(24, 8)
        .with_gl(gl_version)
}

fn get_win_size(window: &glutin::Window) -> Size<i32> {
    let (w, h) = window.get_inner_size().unwrap();
    Size {
        w: w as i32,
        h: h as i32,
    }
}

fn window_to_screen(context: &Context, x: f32, y: f32) -> Point {
    let w = context.win_size.w as f32;
    let h = context.win_size.h as f32;
    let aspect_ratio = w / h;
    Point(Vector2 {
        x: (2.0 * x / w - 1.0) * aspect_ratio,
        y: 1.0 - 2.0 * y / h,
    })
}

pub fn projection_matrix(win_size: Size<i32>) -> Matrix4<f32> {
    let aspect_ratio = win_size.w as f32 / win_size.h as f32;
    cgmath::Ortho {
        left: -aspect_ratio,
        right: aspect_ratio,
        bottom: -1.0,
        top: 1.0,
        near: -1.0,
        far: 1.0,
    }.into()
}

#[derive(Clone, Debug)]
pub struct MouseState {
    pub last_press_pos: Point,
    pub pos: Point,
}

// TODO: use gfx-rs generics, not gfx_device_gl types
pub struct Context {
    events_loop: glutin::EventsLoop,
    win_size: Size<i32>,
    projection_matrix: Matrix4<f32>,
    mouse: MouseState,
    should_close: bool,
    commands_tx: Sender<screen::Command>,
    window: glutin::Window,
    clear_color: [f32; 4],
    device: gfx_device_gl::Device,
    encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
    data: pipe::Data<gfx_device_gl::Resources>,
    pso: gfx::PipelineState<gfx_device_gl::Resources, pipe::Meta>,
    factory: gfx_device_gl::Factory,
    font: rusttype::Font<'static>,
    start_time: time::Instant,
    events: Vec<Event>,
    settings: Settings,
    texture_cache: HashMap<PathBuf, Texture>,
}

impl Context {
    pub(crate) fn new(tx: Sender<screen::Command>, settings: Settings) -> Context {
        let events_loop = glutin::EventsLoop::new();
        let (window, device, mut factory, out, out_depth) =
            gfx_window_glutin::init(new_window_builder(), &events_loop);
        let encoder = factory.create_command_buffer().into();
        let program = new_shader(&window, &mut factory);
        let primitive = gfx::Primitive::TriangleList;
        let pso = new_pso(&mut factory, &program, primitive);
        let sampler = factory.create_sampler_linear();
        let win_size = get_win_size(&window);
        let projection_matrix = projection_matrix(win_size);
        let fake_texture = texture::load_raw(&mut factory, Size { w: 2, h: 2 }, &[0; 4]);
        let fake_mesh = [];
        let data = pipe::Data {
            basic_color: [1.0, 1.0, 1.0, 1.0],
            vbuf: factory.create_vertex_buffer(&fake_mesh),
            texture: (fake_texture.0, sampler),
            out,
            out_depth,
            mvp: Matrix4::identity().into(),
        };
        let mouse = MouseState {
            last_press_pos: Point(Vector2::zero()),
            pos: Point(Vector2::zero()),
        };
        let font = new_font(&settings.font);
        Context {
            settings,
            events_loop,
            data,
            win_size,
            projection_matrix,
            clear_color: [1.0, 1.0, 1.0, 1.0],
            window,
            device,
            factory,
            encoder,
            pso,
            should_close: false,
            commands_tx: tx,
            font,
            mouse,
            start_time: time::Instant::now(),
            events: Vec::new(),
            texture_cache: HashMap::new(),
        }
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Loads a texture with caching
    pub(crate) fn load_texture<P: AsRef<Path>>(&mut self, path: P) -> Texture {
        let path = path.as_ref().to_path_buf();
        if let Some(texture) = self.texture_cache.get(&path) {
            return texture.clone();
        }
        let texture = texture::load(self, &fs::load(&path));
        self.texture_cache.insert(path, texture.clone());
        texture
    }

    pub(crate) fn clear(&mut self) {
        self.encoder.clear(&self.data.out, self.clear_color);
        self.encoder.clear_depth(&self.data.out_depth, 1.0);
    }

    pub fn now(&self) -> Time {
        Time::from_duration(time::Instant::now() - self.start_time)
    }

    pub(crate) fn should_close(&self) -> bool {
        self.should_close
    }

    pub(crate) fn flush(&mut self) {
        self.encoder.flush(&mut self.device);
        self.window.swap_buffers().expect("Can`t swap buffers");
        self.device.cleanup();
    }

    pub(crate) fn font(&self) -> &rusttype::Font {
        &self.font
    }

    pub(crate) fn factory_mut(&mut self) -> &mut gfx_device_gl::Factory {
        &mut self.factory
    }

    // TODO: add `set_bg_color`
    pub fn set_color(&mut self, color: [f32; 4]) {
        self.data.basic_color = color;
    }

    fn mouse(&self) -> &MouseState {
        &self.mouse
    }

    pub(crate) fn draw_mesh(&mut self, mvp: Matrix4<f32>, mesh: &Mesh) {
        self.data.mvp = mvp.into();
        self.data.texture.0 = mesh.texture().0.clone();
        self.data.vbuf = mesh.vertex_buffer().clone();
        self.encoder.draw(mesh.slice(), &self.pso, &self.data);
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.win_size.w as f32 / self.win_size.h as f32
    }

    pub fn add_command(&mut self, command: screen::Command) {
        self.commands_tx.send(command).unwrap();
    }

    pub fn pull_events(&mut self) -> Vec<Event> {
        let mut raw_events = Vec::new();
        self.events_loop.poll_events(|e| raw_events.push(e));
        for event in &raw_events {
            let event = match *event {
                glutin::Event::WindowEvent { ref event, .. } => event,
            };
            self.handle_event(event);
        }
        self.events.split_off(0)
    }

    fn handle_event(&mut self, event: &glutin::WindowEvent) {
        match *event {
            glutin::WindowEvent::Closed => {
                self.should_close = true;
            }
            glutin::WindowEvent::MouseInput(Released, MouseButton::Left) => {
                if self.is_tap() {
                    self.events.push(Event::Click {
                        pos: self.mouse.pos,
                    });
                }
            }
            glutin::WindowEvent::MouseInput(Pressed, MouseButton::Left) => {
                self.mouse.last_press_pos = self.mouse.pos;
            }
            glutin::WindowEvent::MouseMoved(x, y) => {
                self.mouse.pos = window_to_screen(self, x as f32, y as f32);
            }
            glutin::WindowEvent::Touch(touch_event) => {
                let (x, y) = touch_event.location;
                let pos = window_to_screen(self, x as f32, y as f32);
                match touch_event.phase {
                    glutin::TouchPhase::Moved => {
                        self.mouse.pos = pos;
                    }
                    glutin::TouchPhase::Started => {
                        self.mouse.pos = pos;
                        self.mouse.last_press_pos = pos;
                    }
                    glutin::TouchPhase::Ended => {
                        self.mouse.pos = pos;
                        if self.is_tap() {
                            // TODO: check that this is an actual position!
                            self.events.push(Event::Click {
                                pos: self.mouse.pos,
                            });
                        }
                    }
                    glutin::TouchPhase::Cancelled => {
                        unimplemented!();
                    }
                }
            }
            glutin::WindowEvent::Resized(w, h) => {
                if w == 0 || h == 0 {
                    return;
                }
                self.win_size = Size {
                    w: w as i32,
                    h: h as i32,
                };
                self.projection_matrix = projection_matrix(self.win_size);
                gfx_window_glutin::update_views(
                    &self.window,
                    &mut self.data.out,
                    &mut self.data.out_depth,
                );
                let aspect_ratio = self.aspect_ratio();
                self.events.push(Event::Resize { aspect_ratio });
            }
            _ => {}
        }
    }

    /// Check if this was a tap or a swipe
    fn is_tap(&self) -> bool {
        let mouse = self.mouse();
        let diff = mouse.pos.0 - mouse.last_press_pos.0;
        diff.magnitude() < self.settings.tap_tolerance
    }

    pub fn projection_matrix(&self) -> Matrix4<f32> {
        self.projection_matrix
    }
}
