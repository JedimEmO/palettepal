use std::iter::once;
use dominator::{events, Dom};
use crate::model::palette::{Palette};
use dwind::prelude::*;
use dwui::prelude::*;
use futures_signals::signal::{not, Mutable};
use web_sys::{window, CanvasRenderingContext2d, HtmlAnchorElement, HtmlCanvasElement, Url};
use futures_signals::signal::SignalExt;
use log::info;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use crate::mixins::panel::panel_mixin;
use crate::model::sampling::get_equidistant_points_in_range;

fn export_menu(palette: Mutable<Palette>, export_file_content: Mutable<Option<String>>, export_image_content: Mutable<Option<Vec<Vec<(u8, u8, u8)>>>>) -> Dom {
    let expanded = Mutable::new(false);

    html!("div", {
        .dwclass!("transition-all flex flex-col gap-2 w-96 align-items-center p-2 justify-start")
        .dwclass_signal!("h-12", not(expanded.signal()))
        .dwclass_signal!("h-64", expanded.signal())
        .apply(panel_mixin)
        .child(html!("div", {
            .dwclass!("font-bold text-lg text-woodsmoke-300 hover:text-picton-blue-500 cursor-pointer w-full h-12 text-center")
            .text("Export")
            .event(clone!(expanded => move |_: events::Click| {
                expanded.set(!expanded.get());
            }))
        }))
        .child_signal(expanded.signal().map(move |is_expanded| {
            if !is_expanded {
                return None;
            }

            Some(html!("div", {
                .dwclass!("flex flex-col gap-2 justify-start")
                .children([
                    button!({
                        .content(Some(html!("div", {
                            .dwclass!("p-l-2 p-r-2")
                            .text("Export to DWIND")
                        })))
                        .on_click(clone!(palette, export_file_content => move |_| {
                            let sampling_coords = get_equidistant_points_in_range(0., 1., 11);
                            let mut color_file = dwind_build::colors::ColorFile {colors: vec![]};

                            for color in palette.lock_mut().colors.lock_mut().iter() {
                                color_file.colors.push(color.clone().into_dwind_color(sampling_coords.clone()))
                            }

                            let color_file_string = serde_json::to_string_pretty(&color_file).unwrap();

                            export_file_content.set(Some(color_file_string));
                        }))
                    }),
                    button!({
                        .content(Some(html!("div", {
                            .dwclass!("p-l-2 p-r-2")
                            .text("Export to TAILWIND")
                        })))
                        .on_click(clone!(palette => move |_| {
                            window().unwrap().alert_with_message("TODO").unwrap()
                        }))
                    }),
                    button!({
                        .content(Some(html!("div", {
                            .dwclass!("p-l-2 p-r-2")
                            .text("Export to PNG")
                        })))
                        .on_click(clone!(palette, export_image_content => move |_| {
                            let samples = palette.lock_mut().shades_per_color.get_cloned();
                            let sampling_coords = get_equidistant_points_in_range(0., 1., 11);
                            let mut colors = vec![];

                            for color in palette.lock_mut().colors.lock_mut().iter() {
                                let curve = color.samples(samples.clone());
                                colors.push(color.colors_u8(&curve));
                            }

                            export_image_content.set(Some(colors));
                        }))
                    }),
                    button!({
                        .content(Some(html!("div", {
                            .dwclass!("p-l-2 p-r-2")
                            .text("Export to PAL(JASC)")
                        })))
                        .on_click(clone!(palette, export_file_content => move |_| {
                            let jasc_palette = palette.lock_mut().to_jasc_pal();

                            let string = JsValue::from_str(jasc_palette.as_str());

                            let sequence = js_sys::Array::from_iter(once(string));
                            let blob = web_sys::Blob::new_with_str_sequence(&sequence).unwrap_throw();

                            let file_url = Url::create_object_url_with_blob(&blob).unwrap_throw();
                            let mut dl_link = window().unwrap().document().unwrap().create_element("a").unwrap_throw().dyn_into::<HtmlAnchorElement>().unwrap_throw();

                            dl_link.set_attribute("href", &file_url).unwrap_throw();
                            dl_link.set_attribute("download", "palette.pal").unwrap_throw();

                            window().unwrap().document().unwrap().body().unwrap_throw().append_child(&dl_link).unwrap_throw();

                            dl_link.click();

                            window().unwrap().document().unwrap().body().unwrap_throw().remove_child(&dl_link).unwrap_throw();
                            Url::revoke_object_url(&file_url).unwrap_throw();

                            export_file_content.set(Some(jasc_palette));
                        }))
                    })
                ])
            }))
        }))
    })
}

fn save_menu(palette: Mutable<Palette>, export_file_content: Mutable<Option<String>>, export_image_content: Mutable<Option<Vec<Vec<(u8, u8, u8)>>>>) -> Dom {
    let expanded = Mutable::new(false);

    html!("div", {
        .dwclass!("transition-all flex flex-col gap-2 w-96 align-items-center p-2 justify-start")
        .dwclass_signal!("h-12", not(expanded.signal()))
        .dwclass_signal!("h-64", expanded.signal())
        .apply(panel_mixin)
        .child(html!("div", {
            .dwclass!("font-bold text-lg text-woodsmoke-300 hover:text-picton-blue-500 cursor-pointer w-full h-12 text-center")
            .text("Save")
            .event(clone!(expanded => move |_: events::Click| {
                expanded.set(!expanded.get());
            }))
        }))
        .child_signal(expanded.signal().map(move |is_expanded| {
            if !is_expanded {
                return None;
            }

            Some(html!("div", {
                .dwclass!("flex flex-col gap-2")
                .children([
                    button!({
                        .content(Some(html!("div", {
                            .dwclass!("p-l-2 p-r-2")
                            .text("Save to local storage")
                        })))
                        .on_click(clone!(palette => move |_| {
                            let palette_json = serde_json::to_string(&*palette.lock_ref()).unwrap();
                            window().unwrap().local_storage().unwrap().unwrap().set_item("palettepal-palette", &palette_json).unwrap();
                        }))
                    }),
                    button!({
                        .content(Some(html!("div", {
                            .dwclass!("p-l-2 p-r-2")
                            .text("Reload from local storage")
                        })))
                        .on_click(clone!(palette => move |_| {
                            let Some(palette_json) = window().unwrap().local_storage().unwrap().unwrap().get_item("palettepal-palette").unwrap() else {
                                return;
                            };

                            let loaded_palette: Palette = serde_json::from_str(&palette_json).unwrap();
                            palette.set(loaded_palette);
                        }))
                    }),
                    button!({
                        .content(Some(html!("div", {
                            .dwclass!("p-l-2 p-r-2")
                            .text("Clear all")
                        })))
                        .on_click(clone!(palette => move |_| {
                            let Ok(v) = window().unwrap().confirm_with_message("Are you sure? This will delete all colors and settings") else {
                                return;
                            };

                            if !v {
                                return;
                            }

                            palette.set(Palette::default());
                        }))
                    }),
                    button!({
                        .content(Some(html!("div", {
                            .dwclass!("p-l-2 p-r-2")
                            .text("Clear local storage")
                        })))
                        .on_click(clone!(palette => move |_| {
                            let Ok(v) = window().unwrap().confirm_with_message("Are you sure? This will permanently delete all colors and settings") else {
                                return;
                            };

                            if !v {
                                return;
                            }

                            window().unwrap().local_storage().unwrap().unwrap().remove_item("palettepal-palette").unwrap();
                            palette.set(Palette::default());
                        }))
                    })
                ])
            }))
        }))
    })
}

pub fn palette_controls(palette: Mutable<Palette>) -> Dom {
    let mut export_file_content: Mutable<Option<String>> = Mutable::new(None);
    let mut export_image_content: Mutable<Option<Vec<Vec<(u8, u8, u8)>>>> = Mutable::new(None);

    html!("div", {
        .dwclass!("flex flex-col gap-2 pointer-events-auto align-items-center")
        .children([
            html!("div", {
                .dwclass!("@>sm:w-md @<sm:w-sm")
                .dwclass!("flex @sm:flex-row @<sm:flex-col gap-4")
                .children([
                    html!("div", {
                        .apply(panel_mixin)
                        .dwclass!("transition-all flex flex-col gap-2 w-96 align-items-center p-2 justify-start h-12")
                        .children([
                            html!("div", {
                                .dwclass!("font-bold text-lg text-woodsmoke-300 hover:text-picton-blue-500 cursor-pointer w-full h-full text-center")
                                .text("Add Color")
                                .event(clone!(palette => move |_: events::Click| {
                                    palette.lock_mut().add_new_color();
                                }))
                            })
                        ])
                    }),
                    export_menu(palette.clone(), export_file_content.clone(), export_image_content.clone()),
                    save_menu(palette.clone(), export_file_content.clone(), export_image_content.clone())
                ])
            })
        ])
        .child_signal(export_file_content.signal_cloned().map(|content| {
            content.map(|content| {html!("div", {
                .apply(panel_mixin)
                .dwclass!("p-4 overflow-auto @>sm:w-md @<sm:w-sm max-h-64")
                .child(html!("pre", {
                    .text(&content)
                }))
            })})
        }))
        .child_signal(export_image_content.signal_cloned().map(|content| {
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
    })
}