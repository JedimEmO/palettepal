use dominator::{Dom};
use crate::model::palette::{Palette};
use dwind::prelude::*;
use dwui::prelude::*;
use futures_signals::signal::Mutable;
use web_sys::window;
use futures_signals::signal::SignalExt;
use crate::model::sampling::get_equidistant_points_in_range;

pub fn palette_controls(palette: Mutable<Palette>) -> Dom {
    let mut export_file_content: Mutable<Option<String>> = Mutable::new(None);

    html!("div", {
        .dwclass!("flex flex-col gap-2")
        .children([
            html!("div", {
                .dwclass!("@>sm:w-md @<sm:w-sm bg-woodsmoke-800 p-4")
                .dwclass!("flex @sm:flex-row @<sm:flex-col gap-4")
                .children([
                    html!("div", {
                        .dwclass!("flex flex-col gap-1")
                        .children([
                            button!({
                                .content(Some(html!("div", {
                                    .dwclass!("p-l-2 p-r-2")
                                    .text("Add color")
                                })))
                                .on_click(clone!(palette => move |_| {
                                    palette.lock_mut().add_new_color();
                                }))
                            })
                        ])
                    }),
                    html!("div", {
                        .dwclass!("flex flex-col gap-1")
                        .children([
                             button!({
                                .content(Some(html!("div", {
                                    .dwclass!("p-l-2 p-r-2")
                                    .text("Export to DWIND color file")
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
                                    .text("Export to TAILWIND color file")
                                })))
                                .on_click(clone!(palette => move |_| {
                                    window().unwrap().alert_with_message("TODO").unwrap()
                                }))
                            })
                        ])
                    }),
                    html!("div", {
                        .dwclass!("flex flex-col gap-1")
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
                    })
                ])
            })
        ])
        .child_signal(export_file_content.signal_cloned().map(|content| {
            content.map(|content| {html!("div", {
                .dwclass!("bg-woodsmoke-800 p-4 overflow-scroll @>sm:w-md @<sm:w-sm")
                .child(html!("div", {
                    .text(&content)
                }))
            })})
        }))
    })
}