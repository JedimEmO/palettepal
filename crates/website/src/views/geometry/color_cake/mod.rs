use std::f32::consts::PI;
use anyhow::anyhow;
use dominator::Dom;
use futures_signals::signal::{ReadOnlyMutable, SignalExt};
use glam::Vec3;
use log::error;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::WebGl2RenderingContext;
use crate::views::geometry::shader_program::{ColorSpaceVertex, GeometryIndex, ShaderProgram};
use crate::widgets::shader_canvas::*;

pub fn color_cake(hue: ReadOnlyMutable<f32>) -> Dom {
    shader_canvas!({
        .ctor(move |context, b| {
            context.viewport(0, 0, 100, 100);
            context.enable(WebGl2RenderingContext::CULL_FACE);

            let mut color_cake = ColorCake::new(&context).unwrap_throw();

            b.future(async move {
                hue.signal().for_each(move |hue| {
                    let hue = hue / 360.;

                    let _ = color_cake.draw(&context, hue).inspect_err(|e| {
                        error!("failed to draw color cake: {:?}", e);
                    });

                    async move {}
                }).await;
            })
        })
    })
}

pub struct ColorCake {
    shader_program: ShaderProgram
}

impl ColorCake {
    pub fn new(context: &WebGl2RenderingContext) -> anyhow::Result<Self> {

        let mut sides = place_cylinder_sides();
        let mut top_disk = place_cylinder_circle(true);
        let mut bottom_disk = place_cylinder_circle(false);

        let mut geometries = vec![];

        geometries.push(GeometryIndex::Triangles { start_index: 0, count: bottom_disk.len() });
        geometries.push(GeometryIndex::Triangles { start_index: bottom_disk.len(), count: sides.len() });
        geometries.push(GeometryIndex::Triangles { start_index: bottom_disk.len() + sides.len(), count: top_disk.len() });

        let mut vertices = vec![];

        vertices.append(&mut bottom_disk);
        vertices.append(&mut sides);
        vertices.append(&mut top_disk);


        let shader_program = ShaderProgram::new(
            context,
            include_str!("vertex.glsl"),
            include_str!("fragment.glsl"),
            vertices,
            geometries
        )?;

        Ok(Self {
            shader_program
        })
    }

    pub fn draw(&mut self, context: &WebGl2RenderingContext, hue: f32) -> anyhow::Result<()> {
        let program = &self.shader_program.program;

        context.use_program(Some(&program));

        let position_location = context.get_attrib_location(&program, "a_position");
        let color_location = context.get_attrib_location(&program, "a_color");
        let hue_location = context.get_uniform_location(&program, "u_hue");
        let matrix_location = context.get_uniform_location(&program, "u_matrix");

        let buffer = context.create_buffer().ok_or(anyhow!("failed to create buffer"))?;

        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

        unsafe {
            let position_view = js_sys::Float32Array::view_mut_raw((&mut self.shader_program.vertices).as_mut_ptr() as *mut f32, self.shader_program.vertices.len() * size_of::<ColorSpaceVertex>());
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

        let scale = glam::f32::Mat4::from_scale(Vec3::new(0.6, 0.6, 0.6));

        let matrix = glam::f32::Mat4::look_at_rh(
            Vec3::new(0., 0.2, -0.3),
            Vec3::ZERO,
            Vec3::Y,
        );

        let matrix = matrix * scale;

        WebGl2RenderingContext::uniform1f(&context, hue_location.as_ref(), hue);
        WebGl2RenderingContext::uniform_matrix4fv_with_f32_array(&context, matrix_location.as_ref(), false, matrix.as_ref());

        context.clear_color(0.0, 0.0, 0.0, 0.0);
        context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        context.clear(WebGl2RenderingContext::DEPTH_BUFFER_BIT);
        context.clear_depth(0.);

        for geometry in self.shader_program.geometries.iter() {
            match geometry {
                GeometryIndex::Triangles { start_index, count } => {
                    context.draw_arrays(WebGl2RenderingContext::TRIANGLES, *start_index as i32, *count as i32);
                }
            }
        }

        Ok(())
    }
}

fn make_cutaway_cylinder() -> Vec<ColorSpaceVertex> {
    place_cylinder_circle(false)
}

fn place_cylinder_circle(top: bool) -> Vec<ColorSpaceVertex> {
    let y = if top { 1. } else { -1. };
    let l = (y + 1.) / 2.;

    let mut out = vec![];

    let num_verts = 32;
    let slice_radius = (3. * PI / 2.) / num_verts as f32;
    let start_angle = 3. * PI / 2.;
    let pct = start_angle / (PI * 2.);

    for sector in 0..num_verts {
        let angle = start_angle - (sector as f32 * slice_radius);
        let next_angle = start_angle - ((sector + 1) as f32 * slice_radius);

        let h = angle / (2. * PI) / pct;
        let next_h = next_angle / (2. * PI) / pct;

        let x = -angle.cos()*1.5;
        let z = angle.sin()*1.5;
        let next_x = -next_angle.cos()*1.5;
        let next_z = next_angle.sin()*1.5;

        out.push(ColorSpaceVertex { pos: [0., y, 0.], hsx: [next_h, 0., l] });

        out.push(ColorSpaceVertex {
            pos: [next_x, y, next_z],
            hsx: [next_h, 1., l],
        });

        out.push(ColorSpaceVertex {
            pos: [x, y, z],
            hsx: [h, 1., l],
        });

    }

    out
}

fn place_cylinder_sides() -> Vec<ColorSpaceVertex> {
    let mut out = vec![];

    let num_verts = 32;
    let slice_radius = (3. * PI / 2.) / num_verts as f32;
    let start_angle = 3. * PI / 2.;
    let pct = start_angle / (PI * 2.);

    for sector in 0..num_verts {
        let angle = start_angle - (sector as f32 * slice_radius);
        let next_angle = start_angle - ((sector + 1) as f32 * slice_radius);

        let h = angle / (2. * PI) / pct;
        let next_h = next_angle / (2. * PI) / pct;

        let x = -angle.cos()*1.5;
        let z = angle.sin()*1.5;
        let next_x = -next_angle.cos()*1.5;
        let next_z = next_angle.sin()*1.5;

        // Cylinder side triangles

        //A
        out.push(ColorSpaceVertex {
            pos: [x, 1., z],
            hsx: [h, 1., 1.0],
        });

        out.push(ColorSpaceVertex {
            pos: [next_x, -1., next_z],
            hsx: [next_h, 1., 0.],
        });

        out.push(ColorSpaceVertex {
            pos: [x, -1., z],
            hsx: [h, 1., 0.],
        });


        // B
        out.push(ColorSpaceVertex {
            pos: [next_x, -1., next_z],
            hsx: [next_h, 1., 0.],
        });

        out.push(ColorSpaceVertex {
            pos: [x, 1., z],
            hsx: [h, 1., 1.0],
        });

        out.push(ColorSpaceVertex {
            pos: [next_x, 1., next_z],
            hsx: [next_h, 1., 1.0],
        });
    }

    // Cylinder slice
    // A
    out.push(ColorSpaceVertex {
        pos: [-1.5, 1., 0.],
        hsx: [0., 1., 1.0],
    });

    out.push(ColorSpaceVertex {
        pos: [0., 1., 0.],
        hsx: [0., 0., 1.],
    });

    out.push(ColorSpaceVertex {
        pos: [0., -1., 0.],
        hsx: [0., 0., 0.],
    });

    // B
    out.push(ColorSpaceVertex {
        pos: [-1.5, 1., 0.],
        hsx: [0., 1., 1.],
    });

    out.push(ColorSpaceVertex {
        pos: [0., -1., 0.],
        hsx: [0., 0., 0.],
    });

    out.push(ColorSpaceVertex {
        pos: [-1.5, -1., 0.],
        hsx: [0., 1., 0.],
    });

    out
}