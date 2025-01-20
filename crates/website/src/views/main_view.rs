use crate::mixins::panel::panel_mixin;
use crate::model::palette::Palette;
use crate::views::color_panel::color_panel;
use crate::views::palette_controls::palette_controls;
use crate::widgets::menu_overlay::menu_overlay;
use dominator::Dom;
use dwind::prelude::*;
use futures_signals::signal::{always, Mutable, Signal, SignalExt};
use futures_signals::signal_vec::SignalVecExt;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PalettePalViewModel {
    pub palette: Mutable<Palette>,
    pub export_file_content: Mutable<Option<String>>,
    pub export_image_content: Mutable<Option<Vec<Vec<(u8, u8, u8)>>>>,
}

pub fn main_view() -> Dom {
    let palette = Mutable::new(Palette::new());

    let export_file_content: Mutable<Option<String>> = Mutable::new(None);
    let export_image_content: Mutable<Option<Vec<Vec<(u8, u8, u8)>>>> = Mutable::new(None);

    let vm = PalettePalViewModel {
        palette,
        export_file_content,
        export_image_content,
    };

    let inner = menu_overlay(
        always(palette_controls(&vm)),
        always(html!("div", {
            .dwclass!("flex justify-start w-full h-screen align-items-start")
            .child(palette_view(vm))
        })),
    );

    html!("body", {
        .dwclass!("bg-woodsmoke-950")
        .child(inner)
    })
}

pub fn palette_view(vm: PalettePalViewModel) -> Dom {
    html!("div", {
        .dwclass!("flex flex-col gap-4 justify-center m-t-16 w-full")
        .child_signal(vm.palette.signal_ref(clone!(vm => move |palette| {
            Some(html!("div", {
                .dwclass!("flex flex-row gap-4 flex-wrap flex-auto")
                .child_signal(vm.export_file_content.signal_cloned().map( move |content| {
                    content.map(|content| {html!("div", {
                        .apply(panel_mixin)
                        .dwclass!("p-4 overflow-auto @>sm:w-full @<sm:w-sm max-h-64")
                        .child(html!("pre", {
                            .text(&content)
                        }))
                    })})
                }))
                // Renders the palette as a PNG export
                // 1 row per color
                .child_signal(export_png_view(&vm))
                .children_signal_vec(vm.palette.get_cloned().tools_view_state.tools_children_signal(vm.clone()))
                .child(html!("div", {
                    .dwclass!("flex flex-wrap flex-row w-full")
                    .children_signal_vec(palette.colors.signal_vec_cloned().map(clone!(palette => move |color| {
                        color_panel(color, palette.sampling_curves.clone())
                    })))
                }))
            }))
        })))
    })
}

fn export_png_view(vm: &PalettePalViewModel) -> impl Signal<Item = Option<Dom>> {
    vm.export_image_content.signal_cloned().map(move |content| {
        content.map(|content| {
            if content.is_empty() || content[0].is_empty() {
                return html!("div", {})
            }

            let width = content[0].len();
            let height = content.len();

            html!("div", {
                .apply(panel_mixin)
                .dwclass!("p-4 @>sm:w-full @<sm:w-sm")
                .child(html!("canvas" => HtmlCanvasElement, {
                    .dwclass!("w-full h-12")
                    .style("image-rendering", "pixelated")
                    .attr("width", &width.to_string())
                    .attr("height", &height.to_string())
                    .after_inserted(move |node| {
                        let context = node.get_context("2d").unwrap().unwrap().dyn_into::<CanvasRenderingContext2d>().unwrap();
                        let mut x = 0;
                        let mut y = 0;

                        for shade in content {
                            for (r, g, b) in shade {
                                context.set_fill_style_str(&format!("rgb({r} {g} {b})"));
                                context.fill_rect(x as f64, y as f64, 1., 1.);

                                x += 1;
                            }

                            x = 0;
                            y += 1;
                        }
                    })
                }))
            })
        })
    })
}
