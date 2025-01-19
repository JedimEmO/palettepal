use crate::mixins::panel::{widget_panel_mixin};
use crate::model::palette::Palette;
use crate::model::palette_color::PaletteColor;
use crate::views::tools::examples::color_inputs::color_input;
use color::contrast::{is_enhanced_text_contrast, is_minimum_text_contrast, SwatchColorContrast, RGBA};
use dominator::Dom;
use dwind::prelude::*;
use futures_signals::map_ref;
use futures_signals::signal::{always, Mutable, Signal, SignalExt};
use futures_signals::signal_vec::{SignalVec, SignalVecExt};
use std::rc::Rc;
use crate::views::main_view::PalettePalViewModel;
use crate::views::tools::Tool;

pub fn wcag_tool(vm: &PalettePalViewModel, palette: &Palette) -> Dom {
    let color_a = Mutable::new(palette.colors.lock_ref().get(0).cloned());
    let color_b = Mutable::new(palette.colors.lock_ref().get(0).cloned());

    let color_a_shades_signal = color_shades_signal(color_a.clone(), palette);
    let color_b_shades_signal = color_shades_signal(color_b.clone(), palette);

    let contrasts_signal = map_ref! {
        let color_a_shades = color_a_shades_signal,
        let color_b_shades = color_b_shades_signal => {
            color::contrast::swatch_color_contrast(color_a_shades.clone(), color_b_shades.clone())
        }
    }
    .broadcast();

    let text_minimum_contrasts_signal = contrasts_signal.signal_cloned().to_signal_vec().filter(|contrast| {
        is_minimum_text_contrast(contrast.color_a, contrast.color_b) && !is_enhanced_text_contrast(contrast.color_a, contrast.color_b)
    });

    let text_enhanced_contrast_signal = contrasts_signal.signal_cloned().to_signal_vec().filter(|contrast| {
        is_enhanced_text_contrast(contrast.color_a, contrast.color_b)
    });

    let color_names_signal_vec = palette
        .colors
        .signal_vec_cloned()
        .map_signal(|color| color.name.signal_cloned().map(|v| (v.clone(), v)))
        .to_signal_cloned()
        .broadcast();

    let palette = Rc::new(palette.clone());

    let body = html!("div", {
        .dwclass!("flex-1 flex flex-col gap-4 p-4")
        .child(html!("div", {
            .dwclass!("flex flex-row flex-1")
            .children([
                color_input("Color A", &palette, color_a.clone(), color_names_signal_vec.signal_cloned().to_signal_vec()),
                color_input("Color B", &palette, color_b.clone(), color_names_signal_vec.signal_cloned().to_signal_vec()),
            ])
        }))
        .child(html!("div", {
            .dwclass!("overflow-y-scroll flex h-80")
            .child(html!("table", {
                .dwclass!("divide-y border-woodsmoke-500 border-collapse")
                .children([
                    html!("tr", {
                        .dwclass!("border-woodsmoke-500")
                        .children([
                            html!("th", {
                                .text("Fit")
                            }),
                            html!("th", {
                                .text("Color pairs")
                            })
                        ])
                    }),
                    html!("tr", {
                        .dwclass!("border-woodsmoke-500")
                        .children([
                            html!("td", {
                                .text("Text, enhanced")
                            }),
                            html!("td", {
                                .dwclass!("flex flex-wrap flex-row gap-2")
                                .children_signal_vec(contrasts_display(text_enhanced_contrast_signal))
                            })
                        ])
                    }),
                    html!("tr", {
                        .dwclass!("border-woodsmoke-500")
                        .children([
                            html!("td", {
                                .text("Text, minimum")
                            }),
                            html!("td", {
                                .dwclass!("flex flex-wrap flex-row gap-2")
                                .children_signal_vec(contrasts_display(text_minimum_contrasts_signal))
                            })
                        ])
                    })
                ])
            }))
        }))
    });

    let close_cb = vm.tools_view_state.create_close_tool_handler(Tool::WcagContrast);
    html!("div", {
        .dwclass!("flex-1 p-2 relative")
        .apply(widget_panel_mixin(always("WCAG Text Contrast Analysis".to_string()), Some(close_cb)))
        .child(body)
    })
}

fn contrasts_display(contrasts: impl SignalVec<Item=SwatchColorContrast> ) -> impl SignalVec<Item=Dom> {
    contrasts.map(|contrast| {
        let first_color = format!("rgb({}, {}, {})", contrast.color_a.0, contrast.color_a.1, contrast.color_a.2);
        let first_hex = format!("#{:02x}{:02x}{:02x}", contrast.color_a.0, contrast.color_a.1, contrast.color_a.2);
        let first_tailwind = index_to_tailwind_number(contrast.swatch_a_idx);

        let second_color = format!("rgb({}, {}, {})", contrast.color_b.0, contrast.color_b.1, contrast.color_b.2);
        let second_hex = format!("#{:02x}{:02x}{:02x}", contrast.color_b.0, contrast.color_b.1, contrast.color_b.2);
        let second_tailwind = index_to_tailwind_number(contrast.swatch_b_idx);

        html!("div", {
            .dwclass!("flex w-40 flex-row gap-2 bg-woodsmoke-600 rounded-sm p-1")
            .children([
                html!("div", {
                    .dwclass!("w-20 h-10 flex justify-center align-items-center")
                    .style("background", &first_color)
                    .style("color", &second_color)
                    .child(html!("div", {
                        .text(&format!("b-{second_tailwind}"))
                        .attr("title", &second_hex)
                    }))
                }),
                html!("div", {
                    .dwclass!("w-20 h-10 flex justify-center align-items-center")
                    .style("background", &second_color)
                    .style("color", &first_color)
                    .child(html!("div", {
                        .text(&format!("a-{first_tailwind}"))
                        .attr("title", &first_hex)
                    }))
                }),
            ])
        })
    })
}

fn color_shades_signal(
    color: Mutable<Option<PaletteColor>>,
    palette: &Palette,
) -> impl Signal<Item = Vec<RGBA>> {
    let curves = palette.sampling_curves.clone();
    color
        .signal_cloned()
        .map(move |color| {
            color
                .map(|v| v.colors_u8_signal(&curves).boxed_local())
                .unwrap_or(always(vec![]).boxed_local())
        })
        .flatten()
}


fn index_to_tailwind_number(index: usize) -> usize {
    match index {
        0 => 50,
        1 => 100,
        2 => 200,
        3 => 300,
        4 => 400,
        5 => 500,
        6 => 600,
        7 => 700,
        8 => 800,
        9 => 900,
        10 => 950,
        _ => 0,
    }
}