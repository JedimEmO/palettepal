use crate::views::geometry::shader_program::{ColorSpaceVertex, GeometryIndex, ShaderProgram};
use crate::widgets::shader_canvas::*;
use anyhow::anyhow;
use dominator::Dom;
use futures_signals::signal::{ReadOnlyMutable, SignalExt};
use wasm_bindgen::UnwrapThrowExt;
use web_sys::WebGl2RenderingContext;

pub struct ColorQuad {
    shader_program: ShaderProgram,
}

impl ColorQuad {
    pub fn new(context: &WebGl2RenderingContext) -> anyhow::Result<Self> {
        let vertices = vec![
            // A
            ColorSpaceVertex {
                pos: [1., 1., 0.],
                hsx: [0., 1., 1.],
            },
            ColorSpaceVertex {
                pos: [-1., 1., 0.],
                hsx: [0., 0., 1.],
            },
            ColorSpaceVertex {
                pos: [-1., -1., 0.],
                hsx: [0., 0., 0.],
            },
            // B
            ColorSpaceVertex {
                pos: [1., 1., 0.],
                hsx: [0., 1., 1.],
            },
            ColorSpaceVertex {
                pos: [-1., -1., 0.],
                hsx: [0., 0., 0.],
            },
            ColorSpaceVertex {
                pos: [1., -1., 0.],
                hsx: [0., 1., 0.],
            },
        ];

        let geometries = vec![GeometryIndex::Triangles {
            start_index: 0,
            count: vertices.len(),
        }];

        let shader_program = ShaderProgram::new(
            context,
            include_str!("vertex.glsl"),
            include_str!("fragment.glsl"),
            vertices,
            geometries,
        )?;

        Ok(Self { shader_program })
    }

    pub fn draw(
        &mut self,
        context: &WebGl2RenderingContext,
        hue: f32,
        resolution: (f32, f32),
    ) -> anyhow::Result<()> {
        let program = &self.shader_program.program;
        context.use_program(Some(&program));

        let position_location = context.get_attrib_location(&program, "a_position");
        let color_location = context.get_attrib_location(&program, "a_color");
        let resolution_location = context.get_uniform_location(&program, "u_resolution");
        let hue_location = context.get_uniform_location(&program, "u_hue");
        let buffer = context
            .create_buffer()
            .ok_or(anyhow!("failed to create buffer"))?;

        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

        WebGl2RenderingContext::uniform2f(
            &context,
            resolution_location.as_ref(),
            resolution.0,
            resolution.1,
        );

        unsafe {
            let position_view = js_sys::Float32Array::view_mut_raw(
                (&mut self.shader_program.vertices).as_mut_ptr() as *mut f32,
                self.shader_program.vertices.len() * size_of::<ColorSpaceVertex>(),
            );
            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &position_view,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        let vertex_array_object = context
            .create_vertex_array()
            .ok_or(anyhow!("failed to create vertex array object"))?;

        context.bind_vertex_array(Some(&vertex_array_object));

        context.vertex_attrib_pointer_with_i32(
            position_location as u32,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            6 * size_of::<f32>() as i32,
            0,
        );

        context.vertex_attrib_pointer_with_i32(
            color_location as u32,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            6 * size_of::<f32>() as i32,
            3 * size_of::<f32>() as i32,
        );

        context.enable_vertex_attrib_array(position_location as u32);
        context.enable_vertex_attrib_array(color_location as u32);

        let vertex_count = self.shader_program.vertices.len() as i32;

        WebGl2RenderingContext::uniform1f(&context, hue_location.as_ref(), hue);

        context.clear_color(0.0, 0.0, 0.0, 1.0);
        context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        context.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, vertex_count);

        Ok(())
    }
}

pub fn color_plane(hue: ReadOnlyMutable<f32>) -> Dom {
    shader_canvas!({
        .ctor(move|context, b| {
            let mut quad = ColorQuad::new(&context).unwrap_throw();

            b.future(async move {
                hue.signal().for_each(move |hue| {
                    let hue = hue /360.;
                    let _ = quad.draw(&context, hue, (100., 100.)).inspect_err(|e| {
                        log::error!("error: {:?}", e);
                    });

                    async move {}
                }).await;
            })
        })
    })
}
