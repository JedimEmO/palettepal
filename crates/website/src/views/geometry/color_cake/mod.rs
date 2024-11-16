pub mod transform;

use crate::views::geometry::shader_program::{ColorSpaceVertex, GeometryIndex, ShaderProgram};
use crate::widgets::shader_canvas::*;
use anyhow::anyhow;
use dominator::{events, Dom};
use dwind::prelude::*;
use futures_signals::map_ref;
use futures_signals::signal::{Mutable, ReadOnlyMutable, SignalExt};
use glam::{Mat4, Vec2, Vec3, Vec4};
use log::{error, info};
use std::f32::consts::PI;
use std::rc::Rc;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, WebGl2RenderingContext};
use transform::{Transform, AABB};
use crate::model::palette::{ColorShades, PaletteColor};
use crate::views::geometry::transform::Plane;

#[derive(Copy, Clone, Debug)]
enum DragPoint {
    TopLeft,
    BottomRight,
    Center,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Cursor {
    Resize,
    Move,
}


fn sample_line_point_to_world_color_space_pos(pos: (f32, f32)) -> [f32; 3] {
    [pos.0, 2. * pos.1 - 1., 0.]
}

pub fn color_cake(
    hue: Mutable<f32>,
    color: PaletteColor,
    shades_per_color: ReadOnlyMutable<ColorShades>,
    size: (i32, i32),
) -> Dom {
    let sample_points = color.samples_signal(shades_per_color.signal_cloned());

    fn get_curve_aabb(curve: &Vec<(f32, f32)>) -> AABB {
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);

        for point in curve.iter() {
            min.x = min.x.min(point.0);
            min.y = min.y.min(point.1);
            max.x = max.x.max(point.0);
            max.y = max.y.max(point.1);
        }

        AABB::new(min.x, min.y, max.x, max.y)
    }

    let sample_curve_bb_signal = color.samples_signal(shades_per_color.signal_cloned()).map(|s| {
        if s.is_empty() {
            return None;
        }

        Some(get_curve_aabb(&s))
    });

    let cake = shader_canvas!({
        .apply(|b| {
            dwclass!(b, "w-32 h-32 grid-col-1 grid-row-1")
        })
        .canvas_width(size.0)
        .canvas_height(size.1)
        .ctor(move |context, b| {
            context.viewport(0, 0, size.0, size.1);
            context.enable(WebGl2RenderingContext::CULL_FACE);

            let mut color_cake = ColorCake::new(&context).unwrap_throw();

            let b = b.event(clone!(hue => move  |event: events::Click| {
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

                    let _ = color_cake.draw(&context, hue, samples.clone()).inspect_err(|e| {
                        error!("failed to draw color cake: {:?}", e);
                    });

                    async move {}
                }).await;
            })
        })
    });

    let edit_box = html!("canvas" => HtmlCanvasElement, {
        .dwclass!("grid-col-1 grid-row-1 w-32 h-32")
        .attr("width", &format!("{}px", size.0))
        .attr("height", &format!("{}px", size.1))
        .with_node!(canvas => {
            .apply(move |b| {
                let ctx = canvas.get_context("2d").unwrap_throw().unwrap_throw().dyn_into::<CanvasRenderingContext2d>().unwrap_throw();
                let transform = Transform::default();

                const BOX_SIZE: f64  = 32.;

                let hover_cursor: Mutable<Option<Cursor>> = Mutable::new(None);

                let top_left_pos = Mutable::new(Vec2::ZERO);
                let bottom_right_pos = Mutable::new(Vec2::ZERO);
                let bottom_left_pos = Mutable::new(Vec2::ZERO);
                let top_right_pos = Mutable::new(Vec2::ZERO);

                let dragging_corner: Mutable<Option<DragPoint >> = Mutable::new(None);
                let prev_drag_point = Mutable::new(None::<Vec2>);

                let get_hovered_drag_point= Rc::new(clone!(top_left_pos, bottom_right_pos, bottom_left_pos, top_right_pos => move |screen: Vec2| {
                    let top_left = top_left_pos.get();
                    let bottom_right = bottom_right_pos.get();
                    let bottom_left = bottom_left_pos.get();
                    let top_right = top_right_pos.get();

                    if (top_left - screen).length() < BOX_SIZE as f32 {
                        Some(DragPoint::TopLeft)
                    } else if (bottom_right - screen).length() < BOX_SIZE as f32 {
                        Some(DragPoint::BottomRight)
                    } else if screen.x >= top_left.x && screen.y <= bottom_right.x && screen.y >= bottom_right.y && screen.y <= top_left.y {
                        Some(DragPoint::Center)
                    } else {
                        None
                    }
                }));

                let b = b.event(clone!(dragging_corner, prev_drag_point, get_hovered_drag_point => move |event: events::MouseDown| {
                    let x = 512. * event.offset_x() as f32 / 128.;
                    let y = 512. * event.offset_y() as f32 / 128.;

                    let xy_plane_position = transform.project_screen_pos_on_clipped_plane(Vec2::new(x, y), Plane::xy(), AABB::new(0., -1., 1., 1.));

                    let _prev_point = prev_drag_point.replace(xy_plane_position);

                    let corner = get_hovered_drag_point(Vec2::new(x, y));

                    dragging_corner.set(corner);
                })).global_event(clone!(dragging_corner => move |_: events::MouseUp| {
                    dragging_corner.set(None);
                }));

                let b = b.event(clone!(dragging_corner, hover_cursor => move |event: events::MouseMove| {
                    let x = 512. * event.offset_x() as f32 / 128.;
                    let y = 512. * event.offset_y() as f32 / 128.;

                    if let Some(hovered) = get_hovered_drag_point(Vec2::new(x, y)) {
                        match hovered {
                            DragPoint::Center => {
                                hover_cursor.set(Some(Cursor::Move))
                            }
                            _ => {
                                hover_cursor.set(Some(Cursor::Resize))
                            }
                        }
                    } else {
                        hover_cursor.set(None);
                    }

                    let Some(corner) = dragging_corner.get() else {
                        return
                    };

                    let xy_plane_position = transform.project_screen_pos_on_clipped_plane(Vec2::new(x, y), Plane::xy(), AABB::new(0., -1., 1., 1.));

                    let prev_point = prev_drag_point.replace(xy_plane_position);

                    let (Some(new_point), Some(prev_point)) = (xy_plane_position, prev_point) else {
                        return;
                    };

                    let delta = new_point - prev_point;

                    match corner {
                        DragPoint::TopLeft => {
                            color.sampling_rect.lock_mut().x.replace_with(|v| { *v + delta.x });
                            color.sampling_rect.lock_mut().width.replace_with(|v| { *v - delta.x });
                            color.sampling_rect.lock_mut().height.replace_with(|v| { *v + delta.y / 2.});
                        }
                        DragPoint::BottomRight => {
                            color.sampling_rect.lock_mut().width.replace_with(|v| { *v + delta.x });
                            color.sampling_rect.lock_mut().y.replace_with(|v| { *v + delta.y / 2.});
                            color.sampling_rect.lock_mut().height.replace_with(|v| { *v - delta.y / 2. });
                        }
                        _ => {}
                    }
                }));

                let b = b.style_signal("cursor", hover_cursor.signal().map(|v| {
                    match v {
                        Some(Cursor::Resize) => {"nwse-resize"}
                        Some(Cursor::Move) => { "move" }
                        _ => { "default" }
                    }
                }));

                b.future(async move {
                    sample_curve_bb_signal.for_each(|pos| {
                        if pos.is_some() {

                            let mut pos = pos.unwrap();
                            // transform the curve to world space
                            pos.corner.y = 2. * (pos.corner.y - 0.5);
                            pos.dimension.y *= 2.;

                            let top_left = transform.world_to_screen(Vec3::new(pos.corner.x, pos.corner.y + pos.dimension.y, 0.));
                            let bottom_right = transform.world_to_screen(Vec3::new(pos.corner.x + pos.dimension.x, pos.corner.y, 0.));

                            ctx.clear_rect(0., 0., size.0 as f64, size.1 as f64);

                            ctx.set_line_width(4.0);
                            ctx.set_stroke_style_str("black");
                            ctx.stroke_rect(top_left.x as f64, top_left.y as f64, (bottom_right.x - top_left.x) as f64, (bottom_right.y - top_left.y) as f64);

                            // All corners
                            ctx.set_fill_style_str("rgba(128, 128, 255, 1.0)");
                            ctx.fill_rect(top_left.x as f64 - BOX_SIZE/2., top_left.y as f64 - BOX_SIZE/2., BOX_SIZE, BOX_SIZE);
                            ctx.fill_rect(bottom_right.x as f64 - BOX_SIZE/2., bottom_right.y as f64 - BOX_SIZE/2., BOX_SIZE, BOX_SIZE);

                            top_left_pos.set(top_left);
                            bottom_right_pos.set(bottom_right);
                        }
                        async move {}
                    }).await;
                })
            })
        })
    });

    html!("div", {
        .dwclass!("grid w-32 h-32")
        .children([
            cake,
            edit_box
        ])
    })
}

pub struct ColorCake {
    shader_program: ShaderProgram,
    line_shader_program: ShaderProgram,
    transform: Transform,
    sample_curve: Mutable<Vec<(f32, f32)>>,
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
            transform: Default::default(),
            sample_curve: Default::default(),
        })
    }

    pub fn color_plane_screen_coordinates(&self) -> (Vec2, Vec2) {
        let top_left = Vec3::new(1.5, 1., 0.);
        let bottom_right = Vec3::new(0., -1., 0.);

        let viewport = Mat4::from_cols_array(&[
            1., 0., 0., 512. / 2.,
            0., 1., 0., 512. / 2.,
            0., 0., 1., 0.,
            0., 0., 0., 1.,
        ]) * Mat4::from_cols_array(&[
            512. / 2., 0., 0., 0.,
            0., 512. / 2., 0., 0.,
            0., 0., 1., 0.,
            0., 0., 0., 1.,
        ]);

        let mat = self.transform.projection * self.transform.scale * viewport;

        let top_left = mat.transform_vector3(top_left);
        let bottom_right = mat.transform_vector3(bottom_right);

        (bottom_right.truncate() + Vec2::new(256., 256.), top_left.truncate() + Vec2::new(256., 256.))
    }

    pub fn draw(
        &mut self,
        context: &WebGl2RenderingContext,
        hue: f32,
        sample_points: Vec<(f32, f32)>,
    ) -> anyhow::Result<()> {
        self.sample_curve.set(sample_points.clone());
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

        let scale = self.transform.scale;

        let view_matrix = self.transform.projection;

        let matrix = scale * view_matrix;

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
                pos: sample_line_point_to_world_color_space_pos(prev),
                hsx: [0., 0., 0.],
            });

            lines_points.push(ColorSpaceVertex {
                pos: sample_line_point_to_world_color_space_pos(point),
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

        let x = angle.cos() * 1.;
        let z = angle.sin() * 1.;
        let next_x = next_angle.cos() * 1.;
        let next_z = next_angle.sin() * 1.;

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

        let x = angle.cos() * 1.;
        let z = angle.sin() * 1.;
        let next_x = next_angle.cos() * 1.;
        let next_z = next_angle.sin() * 1.;

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
        pos: [1., 1., 0.],
        hsx: [0., 1., 1.],
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
        pos: [1., 1., 0.],
        hsx: [0., 1., 1.],
    });

    out.push(ColorSpaceVertex {
        pos: [0., -1., 0.],
        hsx: [0., 0., 0.],
    });

    out.push(ColorSpaceVertex {
        pos: [1., -1., 0.],
        hsx: [0., 1., 0.],
    });

    out
}
