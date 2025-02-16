use crate::views::geometry::gl_utils::{compile_shader, link_program};
use web_sys::{WebGl2RenderingContext, WebGlProgram};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ColorSpaceVertex {
    pub pos: [f32; 3],
    pub hsx: [f32; 3],
}

pub struct ShaderProgram {
    pub program: WebGlProgram,
    pub vertices: Vec<ColorSpaceVertex>,
}

impl ShaderProgram {
    pub fn new(
        context: &WebGl2RenderingContext,
        vertex_shader: &str,
        fragment_shader: &str,
        vertices: Vec<ColorSpaceVertex>,
    ) -> anyhow::Result<Self> {
        let vert = compile_shader(
            context,
            WebGl2RenderingContext::VERTEX_SHADER,
            vertex_shader,
        )?;

        let frag = compile_shader(
            context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            fragment_shader,
        )?;
        let program = link_program(context, &vert, &frag)?;

        Ok(Self { program, vertices })
    }
}
