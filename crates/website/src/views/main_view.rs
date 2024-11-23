use crate::mixins::panel::panel_mixin;
use crate::model::palette::Palette;
use crate::views::color_panel::color_panel;
use crate::views::curve_editor::sampling_curve_editor;
use crate::views::examples::dwui::dwui_example_container;
use crate::views::palette_controls::palette_controls;
use crate::views::palette_overview::palette_overview;
use crate::widgets::menu_overlay::menu_overlay;
use dominator::Dom;
use dwind::prelude::*;
use futures_signals::signal::{always, Mutable, SignalExt};
use futures_signals::signal_vec::SignalVecExt;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PalettePalViewModel {
    pub show_sampling_curve_editor: Mutable<bool>,
    pub palette: Mutable<Palette>,
    pub export_file_content: Mutable<Option<String>>,
    pub export_image_content: Mutable<Option<Vec<Vec<(u8, u8, u8)>>>>,
}

pub fn main_view() -> Dom {
    let palette = Mutable::new(Palette::new());

    let export_file_content: Mutable<Option<String>> = Mutable::new(None);
    let export_image_content: Mutable<Option<Vec<Vec<(u8, u8, u8)>>>> = Mutable::new(None);

    let vm = PalettePalViewModel {
        show_sampling_curve_editor: Default::default(),
        palette,
        export_file_content,
        export_image_content,
    };

    let inner = menu_overlay(
        always(palette_controls(vm.clone())),
        always(html!("div", {
            .dwclass!("flex justify-center w-full h-screen align-items-start")
            .child(palette_view(vm))
        })),
    );

    html!("div", {
        .dwclass!("linear-gradient-180 gradient-from-woodsmoke-700 gradient-to-woodsmoke-950")
        .child(inner)
    })
}

pub fn palette_view(vm: PalettePalViewModel) -> Dom {
    html!("div", {
        .dwclass!("flex flex-col gap-4 justify-center m-t-16")
        .child(palette_overview(vm.clone()))
        .child_signal(vm.palette.signal_ref(clone!(vm => move |palette| {
            Some(html!("div", {
                .dwclass!("flex flex-col gap-4 ")
                .child_signal(vm.export_file_content.signal_cloned().map( move |content| {
                    content.map(|content| {html!("div", {
                        .apply(panel_mixin)
                        .dwclass!("p-4 overflow-auto @>sm:w-md @<sm:w-sm max-h-64")
                        .child(html!("pre", {
                            .text(&content)
                        }))
                    })})
                }))
                // Renders the palette as a PNG export
                // 1 row per color
                .child_signal(vm.export_image_content.signal_cloned().map(move |content| {
                    content.map(|content| {
                        if content.len() == 0 || content[0].len() == 0 {
                            return html!("div", {})
                        }

                        let width = content[0].len();
                        let height = content.len();

                        html!("div", {
                            .apply(panel_mixin)
                            .dwclass!("p-4 @>sm:w-md @<sm:w-sm")
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
                }))
                .child(dwui_example_container(palette.clone()))
                .child_signal(vm.show_sampling_curve_editor.signal().map(clone!(vm => move |v| if v { Some(sampling_curve_editor(vm.clone())) } else { None })))
                .children_signal_vec(palette.colors.signal_vec_cloned().map(clone!(palette => move |color| {
                    color_panel(color, palette.sampling_curves.clone())
                })))
            }))
        })))
    })
}
