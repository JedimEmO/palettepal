pub mod transform;
pub mod cylinder_geometry;
pub mod color_cake_renderer;

use crate::widgets::shader_canvas::*;
use dominator::{events, Dom};
use dwind::prelude::*;
use futures_signals::map_ref;
use futures_signals::signal::{Mutable, ReadOnlyMutable, SignalExt};
use glam::{Vec2, Vec3};
use std::rc::Rc;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, WebGl2RenderingContext};
use transform::{Transform, AABB};
use crate::model::palette::ColorShades;
use crate::model::palette_color::PaletteColor;
use crate::views::geometry::color_cake_renderer::ColorCake;
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

pub fn color_cake(
    hue: Mutable<f32>,
    color: PaletteColor,
    shades_per_color: ReadOnlyMutable<ColorShades>,
    resolution: (i32, i32),
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

    let sample_curve_world_pos_signal = color.samples_signal(shades_per_color.signal_cloned()).map(|s| {
        s.into_iter().map(|(x, y)| {
            Vec2::new(x, 2. * y - 1.)
        }).collect::<Vec<_>>()
    });

    let render_data_signal = map_ref! {
        let sample_curve_aabb = sample_curve_bb_signal,
        let sample_curve = sample_curve_world_pos_signal => {
            (*sample_curve_aabb, sample_curve.clone())
        }
    };

    let cake = shader_canvas!({
        .apply(|b| {
            dwclass!(b, "w-32 h-32 grid-col-1 grid-row-1")
        })
        .canvas_width(resolution.0)
        .canvas_height(resolution.1)
        .ctor(move |context, b| {
            context.viewport(0, 0, resolution.0, resolution.1);
            context.enable(WebGl2RenderingContext::CULL_FACE);

            let mut color_cake = ColorCake::new(&context).unwrap_throw();

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
        .attr("width", &format!("{}px", resolution.0))
        .attr("height", &format!("{}px", resolution.1))
        .with_node!(canvas => {
            .apply(move |b| {
                let ctx = canvas.get_context("2d").unwrap_throw().unwrap_throw().dyn_into::<CanvasRenderingContext2d>().unwrap_throw();
                let transform = Transform::default();

                const BOX_SIZE: f64  = 47.;

                let hover_cursor: Mutable<Option<Cursor>> = Mutable::new(None);

                let top_left_pos = Mutable::new(Vec2::ZERO);
                let bottom_right_pos = Mutable::new(Vec2::ZERO);

                let dragging_corner: Mutable<Option<DragPoint >> = Mutable::new(None);
                let prev_drag_point = Mutable::new(None::<Vec2>);

                let get_hovered_drag_point= Rc::new(clone!(top_left_pos, bottom_right_pos => move |screen: Vec2| {
                    let top_left = top_left_pos.get();
                    let bottom_right = bottom_right_pos.get();

                    if (top_left - screen).length() < BOX_SIZE as f32 {
                        Some(DragPoint::TopLeft)
                    } else if (bottom_right - screen).length() < BOX_SIZE as f32 {
                        Some(DragPoint::BottomRight)
                    } else if screen.x >= top_left.x && screen.x <= bottom_right.x && screen.y <= bottom_right.y && screen.y >= top_left.y {
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
                })).event(clone!(dragging_corner, prev_drag_point, get_hovered_drag_point => move |event: events::TouchStart| {
                    let rect = event.target().unwrap().dyn_into::<HtmlCanvasElement>().unwrap().get_bounding_client_rect();
                    let x = 512. * (event.touches().next().unwrap().client_x() as f32 - rect.x() as f32) / 128.;
                    let y = 512. * (event.touches().next().unwrap().client_y() as f32 - rect.y() as f32) / 128.;

                    let xy_plane_position = transform.project_screen_pos_on_clipped_plane(Vec2::new(x, y), Plane::xy(), AABB::new(0., -1., 1., 1.));

                    let _prev_point = prev_drag_point.replace(xy_plane_position);

                    let corner = get_hovered_drag_point(Vec2::new(x, y));

                    dragging_corner.set(corner);
                })).event(clone!(dragging_corner => move |_: events::TouchEnd| {
                    dragging_corner.set(None);
                })).global_event(clone!(dragging_corner => move |_: events::MouseUp| {
                    dragging_corner.set(None);
                }));

                let move_event_handler = Rc::new(clone!(dragging_corner, hover_cursor => move |pos: Vec2| {
                    if let Some(hovered) = get_hovered_drag_point(pos) {
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

                    let xy_plane_position = transform.project_screen_pos_on_clipped_plane(pos, Plane::xy(), AABB::new(0., -1., 1., 1.));

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
                        DragPoint::Center => {
                            color.sampling_rect.lock_mut().x.replace_with(|v| { *v + delta.x });
                            color.sampling_rect.lock_mut().y.replace_with(|v| { *v + delta.y / 2.});
                        }
                    }
                }));

                let b = b.event(clone!(move_event_handler => move |event: events::MouseMove| {
                    let x = 512. * event.offset_x() as f32 / 128.;
                    let y = 512. * event.offset_y() as f32 / 128.;

                    move_event_handler(Vec2::new(x, y))
                }));

                let b = b.event(clone!(move_event_handler => move |event: events::TouchMove| {
                    let rect = event.target().unwrap().dyn_into::<HtmlCanvasElement>().unwrap().get_bounding_client_rect();
                    let x = 512. * (event.touches().next().unwrap().client_x() as f32 - rect.x() as f32) / 128.;
                    let y = 512. * (event.touches().next().unwrap().client_y() as f32 - rect.y() as f32) / 128.;

                    move_event_handler(Vec2::new(x, y))
                }));

                let b = b.style_signal("cursor", hover_cursor.signal().map(|v| {
                    match v {
                        Some(Cursor::Resize) => {"nwse-resize"}
                        Some(Cursor::Move) => { "move" }
                        _ => { "default" }
                    }
                }));

                b.future(async move {
                    render_data_signal.for_each(|(pos, curve)| {
                        if pos.is_some() {

                            let mut pos = pos.unwrap();
                            // transform the curve to world space
                            pos.corner.y = 2. * (pos.corner.y - 0.5);
                            pos.dimension.y *= 2.;

                            let top_left = transform.world_to_screen(Vec3::new(pos.corner.x, pos.corner.y + pos.dimension.y, 0.));
                            let bottom_right = transform.world_to_screen(Vec3::new(pos.corner.x + pos.dimension.x, pos.corner.y, 0.));

                            ctx.clear_rect(0., 0., resolution.0 as f64, resolution.1 as f64);

                            ctx.set_line_width(4.0);
                            ctx.set_stroke_style_str("rgba(0, 0, 0, 0.95)");
                            ctx.stroke_rect(top_left.x as f64, top_left.y as f64, (bottom_right.x - top_left.x) as f64, (bottom_right.y - top_left.y) as f64);

                            // All corners
                            ctx.set_fill_style_str("rgba(128, 128, 255, 1.0)");
                            ctx.fill_rect(top_left.x as f64 - BOX_SIZE/2., top_left.y as f64 - BOX_SIZE/2., BOX_SIZE, BOX_SIZE);
                            ctx.fill_rect(bottom_right.x as f64 - BOX_SIZE/2., bottom_right.y as f64 - BOX_SIZE/2., BOX_SIZE, BOX_SIZE);

                            top_left_pos.set(top_left);
                            bottom_right_pos.set(bottom_right);

                            // Render curve
                            ctx.set_stroke_style_str("black");
                            ctx.set_line_width(8.0);
                            ctx.begin_path();

                            for (i, point) in curve.iter().enumerate() {
                                let screen_pos = transform.world_to_screen(Vec3::new(point.x, point.y, 0.));

                                if i == 0 {
                                    ctx.move_to(screen_pos.x as f64, screen_pos.y as f64);
                                } else {
                                    ctx.line_to(screen_pos.x as f64, screen_pos.y as f64);
                                }
                            }

                            ctx.stroke();
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

