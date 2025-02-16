use crate::mixins::panel::widget_panel_mixin;
use crate::model::palette::Palette;
use crate::views::main_view::PalettePalViewModel;
use crate::views::tools::Tool;
use dominator::events::MouseButton;
use dominator::{events, Dom};
use dwind::prelude::*;
use futures::{FutureExt, StreamExt};
use futures_signals::signal::SignalExt;
use futures_signals::signal::{always, Mutable, Signal};
use futures_signals::signal_vec::{MutableVec, SignalVecExt, VecDiff};
use std::time::Duration;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use wasm_bindgen_futures::spawn_local;
use web_sys::console::info;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub fn pixel_art_tool(vm: &PalettePalViewModel) -> impl Signal<Item = Dom> {
    vm.palette.signal_ref(|palette| {
        let close_cb = palette
            .tools_view_state
            .create_close_tool_handler(Tool::PixelArt);
        html!("div", {
        .dwclass!("flex-1 p-2 relative")
        .apply(widget_panel_mixin(always("Pixel Art Preview".to_string()), Some(close_cb)))
            .child(pixel_art_impl(palette))
        })
    })
}

fn pixel_art_impl(palette: &Palette) -> Dom {
    let selected_color = Mutable::new(0);

    html!("div", {
        .dwclass!("flex flex-row")
        .child(pixel_art_palette(palette, selected_color.clone()))
        .child(pixel_editor(palette, selected_color.clone()))
    })
}

fn pixel_art_palette(palette: &Palette, selected_color: Mutable<usize>) -> Dom {
    html!("div", {
        .dwclass!("flex flex-row flex-wrap w-32")
        .children_signal_vec(palette.palette_colors_signal().enumerate().map(move |(color_idx, (r, g, b))| {
            let rgba = format!("rgba({}, {}, {}, 1)", r, g, b);
            html!("div", {
                .dwclass!("w-8 h-8 cursor-pointer")
                .style("background-color", rgba)
                .dwclass_signal!("border border-w-5px border-red-500", selected_color.signal().map(clone!(color_idx => move |s_idx| {
                    color_idx.signal().map(move |color_idx| {
                        color_idx == Some(s_idx)
                    })
                })).flatten())
                .event(clone!(selected_color => move |_: events::Click| {
                    selected_color.set(color_idx.get().unwrap_or(0));
                }))
            })
        }))
    })
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum ToolMode {
    Draw,
    Erase,
}

fn pixel_editor(palette: &Palette, selected_color_index: Mutable<usize>) -> Dom {
    let pixels = Mutable::new([u16::MAX; 80 * 4 * 80 * 4].into_iter().collect::<Vec<_>>());
    let colors = palette.palette_colors_signal().to_signal_cloned();
    let tool_mode = Mutable::new(ToolMode::Draw);

    html!("div", {
        .child(html!("canvas" => HtmlCanvasElement, {
            .attr("width", "320")
            .attr("height", "320")
            .dwclass!("w-80 h-80")
            .with_node!(canvas => {
                .apply (|b| {
                    let ctx = canvas.get_context("2d").unwrap_throw().unwrap_throw().dyn_into::<CanvasRenderingContext2d>().unwrap_throw();

                    let draw_commands = Mutable::<Option<Vec<(usize, usize, u16)>>>::new(None);

                    let is_painting = Mutable::new(false);
                    let prev_point = Mutable::<Option<(usize, usize)>>::new(None);

                    let b = b.event(clone!(draw_commands, is_painting, prev_point, selected_color_index => move |event: events::MouseMove| {
                        if is_painting.get() {
                            let x = event.offset_x();
                            let y = event.offset_y();

                            if x < 0 || x >= 320 || y < 0 || y >= 320 {
                                return;
                            }

                            let x = x as usize;
                            let y = y as usize;

                            let cur = (x, y);
                            let color_index = selected_color_index.get();

                            let mut commands = draw_commands.lock_mut();
                            let mut commands_vec = commands.take().unwrap_or_default();

                            if let Some(prev) = prev_point.get() {
                                // Bresenham's line algorithm to interpolate between `prev` and `cur`
                                let mut x0 = prev.0 as isize;
                                let mut y0 = prev.1 as isize;
                                let x1 = cur.0 as isize;
                                let y1 = cur.1 as isize;

                                let dx = (x1 - x0).abs();
                                let dy = (y1 - y0).abs();
                                let sx = if x0 < x1 { 1 } else { -1 };
                                let sy = if y0 < y1 { 1 } else { -1 };
                                let mut err = dx - dy;

                                while x0 != x1 || y0 != y1 {
                                    commands_vec.push((x0 as usize, y0 as usize, color_index as u16));

                                    let e2 = 2 * err;
                                    if e2 > -dy {
                                        err -= dy;
                                        x0 += sx;
                                    }
                                    if e2 < dx {
                                        err += dx;
                                        y0 += sy;
                                    }
                                }
                            }

                            commands_vec.push((cur.0, cur.1, color_index as u16));
                            commands_vec.dedup();
                            prev_point.set(Some(cur));
                            *commands = Some(commands_vec);
                        }
                    }));

                    // use Q and E as global events for decreasing and increasing selected color index
                    let b = b.global_event(clone!(selected_color_index => move |event: events::KeyDown| {
                        if event.key() == "q" {
                            let idx = selected_color_index.get();
                            selected_color_index.set(idx.saturating_sub(1));
                        } else if event.key() == "e" {
                            let idx = selected_color_index.get();
                            selected_color_index.set(idx.saturating_add(1));
                        }
                    }));

                    let b= b.event(clone!(is_painting, prev_point => move |e:events::MouseDown| {
                        if e.button() == MouseButton::Left {
                            is_painting.set(true);
                            prev_point.set(None);
                        } else {
                            is_painting.set(false);
                        }
                    }));

                    let b = b.global_event(clone!(is_painting => move |_:events::MouseUp| {
                        is_painting.set(false);
                    }));

                    let b = b.future(clone!(pixels => async move {
                        draw_commands.signal_cloned()
                        .throttle(|| gloo_timers::future::sleep(Duration::from_millis(30)))
                        .for_each(clone!(draw_commands => move |cmds| {
                            if let Some(cmds) = cmds {
                                let mut new_pixels = pixels.get_cloned();

                                cmds.iter().for_each(|(x, y, color_index)| {
                                    let pixel_index = y * 320 + x;
                                    // draw a 5x5 circle
                                    for dx in 0..13 {
                                        for dy in 0..13 {
                                            let nx = x + dx;
                                            let ny = y + dy;
                                            if nx < 320 && ny < 320 {
                                                new_pixels[ny * 320 + nx] = *color_index;
                                            }
                                        }
                                    }
                                });

                                pixels.set(new_pixels);
                                draw_commands.set(None);
                            } else {
                            }
                            async {}
                        })).await;
                    }));

                    let palette = palette.clone();

                    b.future(async move {
                        let mut colors = vec![];
                        let mut pixel_values = vec![];

                        let colors_signal = palette.palette_colors_signal().to_signal_cloned().throttle(||gloo_timers::future::sleep(Duration::from_millis(1000)));
                        let mut pixels_stream = pixels.signal_cloned().to_stream();
                        let mut colors_stream = colors_signal.to_stream();

                        loop {
                            let mut next_color = colors_stream.next().fuse();
                            let mut next_pixels = pixels_stream.next().fuse();

                            futures::select! {
                                new_colors = next_color => {
                                    colors = new_colors.unwrap();
                                }
                                new_pixels = next_pixels => {
                                    pixel_values = new_pixels.unwrap();
                                }
                            }

                            let mut px_color = (0,0,0);

                            for x in 0usize..320 {
                                for y in 0usize..320 {
                                    let pixel_index = pixel_values[y * 320 + x];
                                    let (r, g, b) = colors.get(pixel_index as usize).unwrap_or(&(0,0,0));

                                    if (*r, *g, *b) != px_color {
                                        px_color = (*r, *g, *b);
                                        ctx.set_fill_style_str(&format!("rgb({}, {}, {})", r, g, b));
                                    }

                                    ctx.fill_rect(x as f64, y as f64, 1., 1.);
                                }
                            }
                        }
                    })
                })
            })
        }))
    })
}
