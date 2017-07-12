use gfx;
use gfx::preset::blend::ALPHA as ALPHA_PRESET;

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
    }

    pipeline pipe {
        basic_color: gfx::Global<[f32; 4]> = "u_Basic_color",
        mvp: gfx::Global<[[f32; 4]; 4]> = "u_ModelViewProj",
        vbuf: gfx::VertexBuffer<Vertex> = (),
        texture: gfx::TextureSampler<[f32; 4]> = "t_Tex",
        out: gfx::BlendTarget<ColorFormat> = ("Target0", gfx::state::MASK_ALL, ALPHA_PRESET),
        out_depth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}
