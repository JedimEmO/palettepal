use crate::views::geometry::shader_program::{ColorSpaceVertex, GeometryIndex, ShaderProgram};
use crate::widgets::shader_canvas::*;
use anyhow::anyhow;
use dominator::{events, Dom};
use dwind::prelude::*;
use futures_signals::map_ref;
use futures_signals::signal::{Mutable, ReadOnlyMutable, Signal, SignalExt};
use glam::Vec3;
use log::{error, info};
use std::f32::consts::PI;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::WebGl2RenderingContext;

pub fn color_cake(
    hue: Mutable<f32>,
    sample_points: impl Signal<Item=Vec<(f32, f32)>> + 'static,
    size: (i32, i32),
) -> Dom {
    shader_canvas!({
        .apply(|b| {
            dwclass!(b, "w-32 h-32")
        })
        .canvas_width(size.0)
        .canvas_height(size.1)
        .ctor(move |context, b| {
            context.viewport(0, 0, size.0, size.1);
            context.enable(WebGl2RenderingContext::CULL_FACE);

            let mut color_cake = ColorCake::new(&context).unwrap_throw();

            let b = b.event(clone!(context, hue => move  |event: events::Click| {
                info!("click event");
                info!("offset: {}, {}", event.offset_x(), event.offset_y());
                let x = (event.offset_x() - 64) as f32 / 128.;
                let y = -(event.offset_y() - 64) as f32/ 128.;

                info!("click: {}, {}", x, y);

                // Skip the cutout of the cake
                if x > 0. && y < 0. {
                    return;
                }

                let angle = y.atan2(x);

                // convert the clicked angle into 3/4 angle space
                let angle = angle * 5. / 4.;
                let angle = hue.get() + angle.to_degrees();

                hue.set(angle.rem_euclid(360.).floor());
            }));

            b.future(async move {
                let draw_data_signal = map_ref! {
                    let hue = hue.signal(),
                    let samples = sample_points => {
                        (*hue, samples.clone())
                    }
                };

                draw_data_signal.for_each(move |(hue, samples)| {
                    let hue = hue / 360.;

                    let _ = color_cake.draw(&context, hue, samples).inspect_err(|e| {
                        error!("failed to draw color cake: {:?}", e);
                    });

                    async move {}
                }).await;
            })
        })
    })
}

pub struct ColorCake {
    shader_program: ShaderProgram,
    line_shader_program: ShaderProgram,
}

impl ColorCake {
    pub fn new(context: &WebGl2RenderingContext) -> anyhow::Result<Self> {
        let mut sides = place_cylinder_sides();
        let mut top_disk = place_cylinder_circle(true);
        let mut bottom_disk = place_cylinder_circle(false);

        let mut geometries = vec![];

        geometries.push(GeometryIndex::Triangles {
            start_index: 0,
            count: bottom_disk.len(),
        });
        geometries.push(GeometryIndex::Triangles {
            start_index: bottom_disk.len(),
            count: sides.len(),
        });
        geometries.push(GeometryIndex::Triangles {
            start_index: bottom_disk.len() + sides.len(),
            count: top_disk.len(),
        });

        let mut vertices = vec![];

        vertices.append(&mut bottom_disk);
        vertices.append(&mut sides);
        vertices.append(&mut top_disk);

        let shader_program = ShaderProgram::new(
            context,
            include_str!("shaders/cake_vertex.glsl"),
            include_str!("shaders/cake_fragment.glsl"),
            vertices,
            geometries,
        )?;

        let line_shader_program = ShaderProgram::new(
            context,
            include_str!("shaders/line_vertex.glsl"),
            include_str!("shaders/line_fragment.glsl"),
            vec![],
            vec![],
        )?;

        Ok(Self {
            shader_program,
            line_shader_program,
        })
    }

    pub fn draw(
        &mut self,
        context: &WebGl2RenderingContext,
        hue: f32,
        sample_points: Vec<(f32, f32)>,
    ) -> anyhow::Result<()> {
        let program = &self.shader_program.program;

        context.use_program(Some(&program));

        let position_location = context.get_attrib_location(&program, "a_position");
        let color_location = context.get_attrib_location(&program, "a_color");
        let hue_location = context.get_uniform_location(&program, "u_hue");
        let matrix_location = context.get_uniform_location(&program, "u_matrix");

        let buffer = context
            .create_buffer()
            .ok_or(anyhow!("failed to create buffer"))?;

        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

        unsafe {
            let data_view = js_sys::Float32Array::view_mut_raw(
                (&mut self.shader_program.vertices).as_mut_ptr() as *mut f32,
                self.shader_program.vertices.len() * 6,
            );

            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &data_view,
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

        let scale = glam::f32::Mat4::from_scale(Vec3::new(0.6, 0.6, 0.6));

        let matrix = glam::f32::Mat4::look_at_rh(Vec3::new(0., 0.2, -0.3), Vec3::ZERO, Vec3::Y);

        let matrix = matrix * scale;

        WebGl2RenderingContext::uniform1f(&context, hue_location.as_ref(), hue);
        WebGl2RenderingContext::uniform_matrix4fv_with_f32_array(
            &context,
            matrix_location.as_ref(),
            false,
            matrix.as_ref(),
        );

        context.clear_color(0.0, 0.0, 0.0, 0.0);
        context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        context.clear(WebGl2RenderingContext::DEPTH_BUFFER_BIT);
        context.clear_depth(0.);

        for geometry in self.shader_program.geometries.iter() {
            match geometry {
                GeometryIndex::Triangles { start_index, count } => {
                    context.draw_arrays(
                        WebGl2RenderingContext::TRIANGLES,
                        *start_index as i32,
                        *count as i32,
                    );
                }
            }
        }

        // Sample curve lines
        let mut lines_points = vec![];

        let mut prev_point = None;

        for point in sample_points {
            let Some(prev) = prev_point else {
                prev_point = Some(point);
                continue;
            };

            prev_point = Some(point);
            lines_points.push(ColorSpaceVertex {
                pos: [prev.0 * -1.5, 2. * prev.1 - 1., 0.],
                hsx: [0., 0., 0.],
            });
            lines_points.push(ColorSpaceVertex {
                pos: [point.0 * -1.5, 2. * point.1 - 1., 0.],
                hsx: [0., 0., 0.],
            });
        }

        let program = &self.line_shader_program.program;

        context.use_program(Some(&program));

        let position_location = context.get_attrib_location(&program, "a_position");
        let color_location = context.get_attrib_location(&program, "a_color");
        let matrix_location = context.get_uniform_location(&program, "u_matrix");

        let line_buffer = context
            .create_buffer()
            .ok_or(anyhow!("failed to create buffer"))?;

        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&line_buffer));

        unsafe {
            let data_view = js_sys::Float32Array::view_mut_raw(
                (&mut lines_points).as_mut_ptr() as *mut f32,
                lines_points.len() * 6,
            );

            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &data_view,
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

        WebGl2RenderingContext::uniform_matrix4fv_with_f32_array(
            &context,
            matrix_location.as_ref(),
            false,
            matrix.as_ref(),
        );

        context.line_width(8.);
        context.draw_arrays(WebGl2RenderingContext::LINES, 0, lines_points.len() as i32);
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

        let x = -angle.cos() * 1.5;
        let z = angle.sin() * 1.5;
        let next_x = -next_angle.cos() * 1.5;
        let next_z = next_angle.sin() * 1.5;

        out.push(ColorSpaceVertex {
            pos: [0., y, 0.],
            hsx: [next_h, 0., l],
        });

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

        let x = -angle.cos() * 1.5;
        let z = angle.sin() * 1.5;
        let next_x = -next_angle.cos() * 1.5;
        let next_z = next_angle.sin() * 1.5;

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
