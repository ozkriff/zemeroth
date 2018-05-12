use std::io;

use png;
use gfx;
use gfx_device_gl;

use geom::Size;
use pipeline::ColorFormat;
use Context;

#[derive(Debug, Clone)]
pub struct Texture {
    pub raw: gfx::handle::ShaderResourceView<gfx_device_gl::Resources, [f32; 4]>,
    pub size: Size<i32>,
}

pub fn load(context: &mut Context, data: &[u8]) -> Texture {
    let decoder = png::Decoder::new(io::Cursor::new(data));
    let (info, mut reader) = decoder.read_info().expect("Can't decode the image");
    let size = Size {
        w: info.width as i32,
        h: info.height as i32,
    };
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf).expect("Can't read the frame");
    load_raw(context.factory_mut(), size, &buf)
}

pub fn load_raw<F>(factory: &mut F, size: Size<i32>, data: &[u8]) -> Texture
where
    F: gfx::Factory<gfx_device_gl::Resources>,
{
    let kind = gfx::texture::Kind::D2(
        size.w as gfx::texture::Size,
        size.h as gfx::texture::Size,
        gfx::texture::AaMode::Single,
    );
    let mipmap = gfx::texture::Mipmap::Provided;
    let (_, view) = factory
        .create_texture_immutable_u8::<ColorFormat>(kind, mipmap, &[data])
        .unwrap();
    Texture { raw: view, size }
}
