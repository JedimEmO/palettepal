use crate::mixins::panel::panel_mixin;
use crate::model::palette::Palette;
use crate::views::main_view::PalettePalViewModel;
use dominator::{events, Dom, EventOptions};
use dwind::prelude::*;
use dwui::prelude::*;
use futures_signals::signal::{Mutable, SignalExt};
use futures_signals::signal_vec::SignalVecExt;
use std::rc::Rc;
use std::time::Duration;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub fn palette_overview(vm: PalettePalViewModel) -> Dom {
    let PalettePalViewModel { palette, .. } = vm;
    html!("div", {
        .apply(panel_mixin)
        .dwclass!("p-4 @>sm:w-md @<sm:w-sm flex flex-row justify-center gap-16")
        .child(preview_palette(palette.get_cloned()))
        .child(color_circle_preview(palette.get_cloned()))
    })
}

fn preview_palette(palette: Palette) -> Dom {
    let sampling_curves = palette.sampling_curves.clone();

    let colors = palette
        .colors
        .signal_vec_cloned()
        .map_signal(clone!(sampling_curves => move |color| {
            color.colors_u8_signal(&sampling_curves)
        }))
        .to_signal_cloned();

    const PIXEL_SIZE: f64 = 40.;

    html!("canvas" => HtmlCanvasElement, {
        .attr("width", "512")
        .attr("height", "512")
        .dwclass!("w-32 h-32")
        .with_node!(canvas => {
            .apply (|b| {
                let ctx = canvas.get_context("2d").unwrap_throw().unwrap_throw().dyn_into::<CanvasRenderingContext2d>().unwrap_throw();

                b.future(async move {
                    colors.throttle(|| gloo_timers::future::sleep(Duration::from_millis(100)))
                    .for_each(|colors| {
                        ctx.clear_rect(0., 0., 512., 512.);

                        for (y, color) in colors.into_iter().enumerate() {
                            for (x, (r, g, b)) in color.iter().enumerate() {
                                ctx.set_fill_style_str(&format!("rgb({}, {}, {})", r, g, b));
                                ctx.fill_rect(x as f64 * PIXEL_SIZE, y as f64 * PIXEL_SIZE, PIXEL_SIZE, PIXEL_SIZE);
                            }
                        }

                        async move {}
                    }).await;
                })
            })
        })
    })
}

fn color_circle_preview(palette: Palette) -> Dom {
    let colors = palette.colors.signal_vec_cloned();

    let dragging_hue: Mutable<Option<Mutable<f32>>> = Mutable::new(None);

    let on_move = Rc::new(
        clone!(dragging_hue, palette => move |x: f32, y: f32, all: bool| {
            if let Some(hue) = dragging_hue.get_cloned() {
                let dx = x  - 256.;
                let dy = y - 256.;
                let angle = dy.atan2(dx).to_degrees();
                let old_angle = hue.get();

                if all {
                    for color in palette.colors.lock_mut().iter() {
                        color.hue.set(color.hue.get() + angle - old_angle);
                    }
                } else {
                    hue.set(angle);
                }
            }
        }),
    );

    html!("div", {
        .dwclass!("grid")
        .child(html!("canvas" => HtmlCanvasElement, {
            .attr("width", "512")
            .attr("height", "512")
            .dwclass!("w-32 h-32 grid-col-1 grid-row-1")
            .after_inserted(|node| {
                let ctx = node.get_context("2d").unwrap_throw().unwrap_throw().dyn_into::<CanvasRenderingContext2d>().unwrap_throw();
                let mut angle = 0.;
                let num_sectors = 64;

                for i in 0..num_sectors {
                    let hsv = hsv::hsv_to_rgb(angle, 1., 1.);
                    let next_angle = angle + 360. / num_sectors as f64;
                    ctx.set_fill_style_str(&format!("rgb({}, {}, {})", hsv.0, hsv.1, hsv.2));

                    ctx.begin_path();
                    ctx.move_to(256., 256.);
                    ctx.arc(256., 256., 256., angle.to_radians(), next_angle.to_radians()).unwrap_throw();
                    ctx.fill();
                    ctx.close_path();

                    angle = next_angle;
                }
            })
        }))
        .child(svg!("svg", {
            .attr("viewBox", "0 0 512, 512")
            .dwclass!("w-32 h-32 grid-col-1 grid-row-1")
            .class(class! {
                .style("cursor", "crosshair")
            })
            .event(clone!(palette => move |event: events::DoubleClick| {
                let dx = 512. * event.offset_x() as f32 /  128. - 256.;
                let dy = 512. * event.offset_y() as f32 / 128. - 256.;

                let hue = dy.atan2(dx).to_degrees();

                palette.add_new_color_hue(hue);
            }))
            .event(clone!(on_move => move |event: events::MouseMove| {
                let ox = 512. * event.offset_x() as f32 /  128.;
                let oy = 512. * event.offset_y() as f32 / 128.;

                on_move(ox, oy, event.shift_key());
            }))
            .event(clone!(on_move => move |event: events::TouchMove| {
                let rect = event.target().unwrap().dyn_into::<HtmlCanvasElement>().unwrap().get_bounding_client_rect();
                let ox = 512. * (event.touches().next().unwrap().client_x() as f32 - rect.x() as f32) / 128.;
                let oy = 512. * (event.touches().next().unwrap().client_y() as f32 - rect.y() as f32) / 128.;
                on_move(ox, oy, false);
            }))
            .children_signal_vec(colors.map(clone!(dragging_hue, palette => move |color| {
                svg!("circle", {
                    .dwclass!("cursor-pointer")
                    .attr("r", "20px")
                    .attr_signal("cx", color.hue.signal().map(|v| (v.to_radians().cos() * 180. + 256.).to_string()))
                    .attr_signal("cy", color.hue.signal().map(|v| (v.to_radians().sin() * 180. + 256.).to_string()))
                    .event_with_options(&EventOptions {bubbles: true,preventable: true}, clone!(color, palette => move |event: events::DoubleClick| {
                        event.prevent_default();
                        event.stop_propagation();
                        palette.remove_hue(color.hue.get())
                    }))
                    .event(clone!(dragging_hue, color => move |_: events::MouseDown| {
                        dragging_hue.set(Some(color.hue.clone()));
                    }))
                    .event(clone!(dragging_hue, color => move |_: events::TouchStart| {
                        dragging_hue.set(Some(color.hue.clone()));
                    }))
                    .global_event(clone!(dragging_hue, color => move |_: events::MouseUp| {
                        dragging_hue.set(None);
                    }))
                    .global_event(clone!(dragging_hue, color => move |_: events::TouchEnd| {
                        dragging_hue.set(None);
                    }))
                })
            })))
        }))
    })
}
