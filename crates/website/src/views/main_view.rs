use crate::model::palette::{ColorSampler, ColorShades, Palette, PaletteColor};
use crate::views::color_panel::color_panel;
use dominator::Dom;
use dwind::prelude::*;
use futures_signals::signal::{always, Mutable, SignalExt};
use futures_signals::signal_vec::{MutableVec, SignalVecExt};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use crate::mixins::panel::panel_mixin;
use crate::views::palette_controls::palette_controls;
use crate::widgets::menu_overlay::menu_overlay;

pub fn main_view() -> Dom {
    let palette = Mutable::new(Palette {
        shades_per_color: Mutable::new(ColorShades::Tailwind),
        colors: MutableVec::new_with_values(vec![
            PaletteColor {
                name: "default-color".to_string().into(),
                hue: Default::default(),
                sampling_rect: Default::default(),
                sampler: Mutable::new(ColorSampler::DwindCurve),
            }
        ]),
    });

    let export_file_content: Mutable<Option<String>> = Mutable::new(None);
    let export_image_content: Mutable<Option<Vec<Vec<(u8, u8, u8)>>>> = Mutable::new(None);

    let inner = menu_overlay(
        always(palette_controls(palette.clone(), export_file_content.clone(), export_image_content.clone())),
        always(html!("div", {
            .dwclass!("flex justify-center w-full h-screen align-items-start")
            .child(palette_view(palette, export_file_content, export_image_content))
        })),
    );

    html!("div", {
        .dwclass!("linear-gradient-180 gradient-from-woodsmoke-700 gradient-to-woodsmoke-950")
        .child(inner)
    })
}

pub fn palette_view(palette: Mutable<Palette>, export_file_content: Mutable<Option<String>>, export_image_content: Mutable<Option<Vec<Vec<(u8, u8, u8)>>>>) -> Dom {
    html!("div", {
        .dwclass!("flex flex-col gap-4 justify-center m-t-16")
        .child_signal(palette.signal_ref(move |palette| {
            Some(html!("div", {
                .dwclass!("flex flex-col gap-4 ")
                .child_signal(export_file_content.signal_cloned().map( move |content| {
                    content.map(|content| {html!("div", {
                        .apply(panel_mixin)
                        .dwclass!("p-4 overflow-auto @>sm:w-md @<sm:w-sm max-h-64")
                        .child(html!("pre", {
                            .text(&content)
                        }))
                    })})
                }))
                .child_signal(export_image_content.signal_cloned().map(move |content| {
                    content.map(|content| {
                        if content.len() == 0 || content[0].len() == 0 {
                            return html!("div", {})
                        }

                        let width = content[0].len();
                        let height = content.len();

                        html!("div", {
                            .apply(panel_mixin)
                            .dwclass!("p-4 @>sm:w-md @<sm:w-sm ")
                            .child(html!("canvas" => HtmlCanvasElement, {
                                .dwclass!("w-full h-12")
                                .style("image-rendering", "pixelated")
                                .attr("width", &(width * height).to_string())
                                .attr("height", "1")
                                .after_inserted(move |node| {
                                    let context = node.get_context("2d").unwrap().unwrap().dyn_into::<CanvasRenderingContext2d>().unwrap();
                                    let mut x = 0;

                                    for shade in content {
                                        for (r, g, b) in shade {
                                            context.set_fill_style_str(&format!("rgb({r} {g} {b})"));
                                            context.fill_rect(x as f64, 0., 1., 1.);

                                            x += 1;
                                        }
                                    }
                                })
                            }))
                        })
                    })
                }))
                .children_signal_vec(palette.colors.signal_vec_cloned().map(clone!(palette => move |color| {
                    color_panel(color, palette.shades_per_color.read_only())
                })))
            }))
        }))
    })
}
