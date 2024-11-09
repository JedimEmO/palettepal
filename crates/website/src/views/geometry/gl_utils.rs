use anyhow::anyhow;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

pub fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> anyhow::Result<WebGlShader> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| anyhow!("failed to create shader"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    let compiled = context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false);

    if compiled {
        Ok(shader)
    } else {
        let log = context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| "unknown error creating shader".to_string());
        context.delete_shader(Some(&shader));
        Err(anyhow!("failed to compile shader: {}", log))
    }
}

pub fn link_program(context: &WebGl2RenderingContext, vert_shader: &WebGlShader, frag_shader: &WebGlShader) -> anyhow::Result<WebGlProgram> {
    let program = context.create_program().ok_or_else(|| anyhow!("failed to create program"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool().unwrap_or(false) {
        Ok(program)
    } else {
        Err(anyhow!("failed to link program"))
    }
}
