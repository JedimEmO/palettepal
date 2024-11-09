use anyhow::anyhow;
use dominator::Dom;
use dwind::prelude::*;
use dwui::prelude::*;
use dwui::{slider, text_input};
use futures_signals::signal::{Mutable, SignalExt};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext, WebGlProgram, WebGlShader};
use crate::views::geometry::ColorCake;
use crate::views::main_view::PaletteColor;

pub fn color_panel(color: &PaletteColor) -> Dom {
    let hue: Mutable<f32> = Mutable::new(0.5);

    html!("div", {
        .dwclass!("p-4 bg-woodsmoke-800 w-md")
        .dwclass!("flex flex-row gap-4")
        .child(html!("canvas" => HtmlCanvasElement, {
            .attr("width", "100px")
            .attr("height", "100px")

            .with_node!(canvas => {
                .apply(|b| {
                    let mut draw = webgl_mount(canvas).unwrap();

                    b.future(hue.signal().for_each(move |hue| {
                        draw(hue);
                        async move {}
                    }))
                })
            })
        }))
        .child(html!("canvas" => HtmlCanvasElement, {
            .attr("width", "100px")
            .attr("height", "100px")

            .with_node!(canvas => {
                .apply(|b| {
                    let context =  canvas
                        .get_context("webgl2")
                        .map_err(|_| anyhow!("failed to get context")).expect_throw("failed to get context")
                        .ok_or(anyhow!("failed to get context")).expect_throw("failed to get context")
                        .dyn_into::<WebGl2RenderingContext>().map_err(|_| {
                        anyhow!("failed to convert to webgl2 context")
                    }).expect_throw("failed to get context");

                    context.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
                    context.enable(WebGl2RenderingContext::CULL_FACE);

                    let mut cylinder = ColorCake::new(&context).expect_throw("failed to create cylinder");

                    b.future(hue.signal().for_each(move |hue| {
                        cylinder.draw(&context, hue).expect_throw("failed to draw cylinder");
                        async move {}
                    }))
                })
            })
        }))
        .child(html!("div", {
            .child(slider!({
                .label("hue".to_string())
                .max(1.)
                .min(0.)
                .step(0.01)
                .value(hue.clone())
            }))
        }))
    })
}

fn webgl_mount(canvas: HtmlCanvasElement) -> anyhow::Result<impl FnMut(f32) -> () + 'static> {
    let context = canvas
        .get_context("webgl2")
        .map_err(|_| anyhow!("failed to get context"))?
        .ok_or(anyhow!("failed to get context"))?
        .dyn_into::<WebGl2RenderingContext>().map_err(|_| {
        anyhow!("failed to convert to webgl2 context")
    })?;

    context.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

    let vert = compile_shader(&context, WebGl2RenderingContext::VERTEX_SHADER, include_str!("shaders/vertex.glsl"))?;
    let frag = compile_shader(&context, WebGl2RenderingContext::FRAGMENT_SHADER, include_str!("shaders/fragment.glsl"))?;
    let program = link_program(&context, &vert, &frag)?;

    context.use_program(Some(&program));

    // Make a quad for now
    let vertices = [
        -1.0, -1.0, 0.0, 0.0, 0.0, // bottom left
        -1.0, 1.0, 1.0, 1.0, 1.0, // top left
        1.0, -1.0, 0.0, 0.0, 0.0, // bottom right
        1.0, 1.0, 1.0, 0.0, 0.0 // top right
    ];

    let position_location = context.get_attrib_location(&program, "a_position");
    let color_location = context.get_attrib_location(&program, "a_color");
    let resolution_location = context.get_uniform_location(&program, "u_resolution");
    let hue_location = context.get_uniform_location(&program, "u_hue");
    let buffer = context.create_buffer().ok_or(anyhow!("failed to create buffer"))?;

    context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

    WebGl2RenderingContext::uniform2f(&context, resolution_location.as_ref(), canvas.width() as f32, canvas.height() as f32);

    unsafe {
        let position_view = js_sys::Float32Array::view(&vertices);
        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &position_view,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }

    let vertex_array_object = context.create_vertex_array()
        .ok_or(anyhow!("failed to create vertex array object"))?;

    context.bind_vertex_array(Some(&vertex_array_object));

    context.vertex_attrib_pointer_with_i32(
        position_location as u32,
        2,
        WebGl2RenderingContext::FLOAT,
        false,
        5 * size_of::<f32>() as i32,
        0,
    );

    context.vertex_attrib_pointer_with_i32(
        color_location as u32,
        3,
        WebGl2RenderingContext::FLOAT,
        false,
        5 * size_of::<f32>() as i32,
        2 * size_of::<f32>() as i32,
    );

    context.enable_vertex_attrib_array(position_location as u32);
    context.enable_vertex_attrib_array(color_location as u32);

    let vertex_count = (vertices.len() / 5) as i32;

    let draw = move |hue: f32| {
        WebGl2RenderingContext::uniform1f(&context, hue_location.as_ref(), hue);

        context.clear_color(0.0, 0.0, 0.0, 1.0);
        context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        context.draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, vertex_count);
    };

    Ok(draw)
}

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
