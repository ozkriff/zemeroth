use gfx;
use gfx::traits::FactoryExt;
use gfx_device_gl;

use context::Context;
use texture::Texture;
use geom::Size;
use pipeline::Vertex;

pub type VertexIndex = u16;

#[derive(Clone, Debug)]
pub struct Mesh {
    slice: gfx::Slice<gfx_device_gl::Resources>,
    vertex_buffer: gfx::handle::Buffer<gfx_device_gl::Resources, Vertex>,
    texture: Texture,
}

/// Rectangular mesh
#[derive(Clone, Debug)]
pub struct RMesh {
    mesh: Mesh,
    size: Size<f32>,
}

impl RMesh {
    pub fn new(context: &mut Context, texture: Texture, size: Size<f32>) -> Self {
        let w = size.w / 2.0;
        let h = size.h / 2.0;
        let vertices = &[
            Vertex {
                pos: [-w, -h],
                uv: [0.0, 1.0],
            },
            Vertex {
                pos: [-w, h],
                uv: [0.0, 0.0],
            },
            Vertex {
                pos: [w, -h],
                uv: [1.0, 1.0],
            },
            Vertex {
                pos: [w, h],
                uv: [1.0, 0.0],
            },
        ];
        let indices = &[0, 1, 2, 1, 2, 3];
        let mesh = Mesh::new(context, vertices, indices, texture);
        Self { mesh, size }
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn size(&self) -> Size<f32> {
        self.size
    }
}

impl Mesh {
    pub fn new(
        context: &mut Context,
        vertices: &[Vertex],
        indices: &[VertexIndex],
        texture: Texture,
    ) -> Mesh {
        let factory = context.factory_mut();
        let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(vertices, indices);
        Mesh {
            slice,
            vertex_buffer,
            texture,
        }
    }

    pub fn slice(&self) -> &gfx::Slice<gfx_device_gl::Resources> {
        &self.slice
    }

    pub fn vertex_buffer(&self) -> &gfx::handle::Buffer<gfx_device_gl::Resources, Vertex> {
        &self.vertex_buffer
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }
}
