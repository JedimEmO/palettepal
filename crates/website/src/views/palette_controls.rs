use crate::mixins::click_outside_collapse::click_outside_collapse_mixin;
use crate::mixins::panel::panel_mixin;
use crate::model::palette::Palette;
use crate::views::main_view::PalettePalViewModel;
use dominator::{events, Dom};
use dwind::prelude::*;
use dwui::prelude::*;
use futures_signals::signal::SignalExt;
use futures_signals::signal::{not, Mutable};
use gloo_file::futures::read_as_text;
use gloo_file::Blob;
use std::iter::once;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, HtmlAnchorElement, HtmlInputElement, Url};
use crate::views::tools::Tool;

pub fn palette_controls(vm: &PalettePalViewModel) -> Dom {
    html!("div", {
        .dwclass!("flex flex-col gap-2 pointer-events-auto align-items-center")
        .children([
            html!("div", {
                .dwclass!("@>sm:w-full @<sm:w-sm flex @sm:flex-row @<sm:flex-col")
                .dwclass!("flex @sm:flex-row @<sm:flex-col gap-4")
                .children([
                    tools_menu(vm.clone()),
                    export_menu(vm.clone()),
                    save_menu(vm.palette.clone())
                ])
            })
        ])
    })
}

fn tools_menu(vm: PalettePalViewModel) -> Dom {
    application_menu(
        "Tools",
        move || {
            html!("div", {
                .dwclass!("flex flex-col gap-2")
                .children([
                    tool_menu_entry(&vm, Tool::PaletteOverview),
                    tool_menu_entry(&vm, Tool::CurveEditor),
                    tool_menu_entry(&vm, Tool::WcagContrast),
                    tool_menu_entry(&vm, Tool::DwuiExample),
                    tool_menu_entry(&vm, Tool::PixelArt),
                    tool_menu_entry(&vm, Tool::ColorImport),
                ])
            })
        }
    )
}

fn tool_menu_entry(vm:&PalettePalViewModel, tool: Tool) -> Dom {
    html!("div", {
        .dwclass!("font-bold text-base text-woodsmoke-300 hover:text-picton-blue-500 cursor-pointer w-full h-full text-center")
        .dwclass_signal!("text-picton-blue-400", vm.palette.get_cloned().tools_view_state.tool_state_signal(tool))
        .text(&format!("{tool}"))
        .event(clone!(vm => move |_: events::Click| {
            vm.palette.get_cloned().tools_view_state.toggle(tool)
        }))
    })
}

fn application_menu(label: &str, mut content_factory: impl FnMut() -> Dom + 'static) -> Dom {
    let expanded = Mutable::new(false);
    html!("div", {
        .apply(panel_mixin)
        .dwclass!("transition-all flex flex-col flex-1 gap-2 align-items-center p-2 justify-start h-12")
        .dwclass_signal!("h-12", not(expanded.signal()))
        .dwclass_signal!("h-64", expanded.signal())
        .apply(click_outside_collapse_mixin(clone!(expanded => move || expanded.set(false))))
        .child(html!("div", {
            .dwclass!("font-bold text-base text-woodsmoke-300 hover:text-picton-blue-500 cursor-pointer w-full h-12 text-center")
            .text(label)
            .event(clone!(expanded => move |_: events::Click| {
                expanded.set(!expanded.get());
            }))
        }))
        .child_signal(expanded.signal().map(move |is_expanded| {
            if !is_expanded {
                return None;
            }

            Some(content_factory())
        }))
    })
}

fn export_menu(vm: PalettePalViewModel) -> Dom {
    let PalettePalViewModel {
        palette,
        export_file_content,
        export_image_content,
        ..
    } = vm;

    application_menu("Export", move || {
        html!("div", {
            .dwclass!("flex flex-col gap-2 justify-start")
            .children([
                button!({
                    .content(Some(html!("div", {
                        .dwclass!("p-l-2 p-r-2")
                        .text("Export to DWIND")
                    })))
                    .on_click(clone!(palette, export_file_content => move |_| {
                        let mut color_file = dwind_build::colors::ColorFile {colors: vec![]};
                        let palette = palette.lock_mut();

                        for color in palette.colors.lock_mut().iter() {
                            color_file.colors.push(color.clone().into_dwind_color(&palette.sampling_curves).unwrap_throw())
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
                    .on_click(|_| {
                        window().unwrap().alert_with_message("TODO").unwrap()
                    })
                }),
                button!({
                    .content(Some(html!("div", {
                        .dwclass!("p-l-2 p-r-2")
                        .text("Export to PNG")
                    })))
                    .on_click(clone!(palette, export_image_content => move |_| {
                        let palette = palette.lock_mut();
                        let samples = palette.sampling_curves.clone();
                        let mut colors = vec![];

                        for color in palette.colors.lock_mut().iter() {
                            let curve = color.samples(&samples);
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
                        download_file("palette.pal", jasc_palette.clone());

                        export_file_content.set(Some(jasc_palette));
                    }))
                })
            ])
        })
    })
}

fn save_menu(palette: Mutable<Palette>) -> Dom {
    let expanded = Mutable::new(false);

    html!("div", {
        .dwclass!("transition-all flex flex-col gap-2 align-items-center p-2 justify-start min-w-60")
        .dwclass_signal!("h-12", not(expanded.signal()))
        .dwclass_signal!("h-64", expanded.signal())
        .apply(panel_mixin)
        .child(html!("div", {
            .dwclass!("font-bold text-base text-woodsmoke-300 hover:text-picton-blue-500 cursor-pointer w-full h-12 text-center")
            .text("File")
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
                .apply(click_outside_collapse_mixin(clone!(expanded => move || expanded.set(false))))
                .children([
                    button!({
                        .content(Some(html!("div", {
                            .dwclass!("p-l-2 p-r-2")
                            .text("Save file")
                        })))
                        .on_click(clone!(palette => move |_| {
                            let palette_json = serde_json::to_string(&*palette.lock_ref()).unwrap();
                            download_file("palette.palettepal", palette_json);
                        }))
                    }),
                    html!("div", {
                        .child(html!("input" => HtmlInputElement, {
                            .attr("id", "uploadpalettefile")
                            .attr("type", "file")
                            .with_node!(file => {
                                .event(clone!(palette => move |_: events::Change| {
                                    let file = file.files().unwrap().get(0).unwrap();

                                    spawn_local(clone!(palette => async move {
                                        let content = read_as_text(&Blob::from(file)).await.unwrap_throw();
                                        let loaded_palette: Palette = serde_json::from_str(&content).unwrap_throw();
                                        palette.set(loaded_palette);
                                    }));
                                }))
                            })
                            .attr("hidden", "hidden")
                        }))
                        .child(button!({
                            .content(Some(html!("div", {
                                .dwclass!("p-l-2 p-r-2")
                                .text("Load File")
                            })))
                            .on_click(move |_| {
                                let input = window().unwrap().document().unwrap().query_selector("#uploadpalettefile").unwrap().unwrap().dyn_into::<HtmlInputElement>().unwrap();
                                input.click();
                            })
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
                    })
                ])
            }))
        }))
    })
}

fn download_file(filename: &str, content: String) {
    let string = JsValue::from_str(content.as_str());

    let sequence = js_sys::Array::from_iter(once(string));
    let blob = web_sys::Blob::new_with_str_sequence(&sequence).unwrap_throw();

    let file_url = Url::create_object_url_with_blob(&blob).unwrap_throw();
    let dl_link = window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("a")
        .unwrap_throw()
        .dyn_into::<HtmlAnchorElement>()
        .unwrap_throw();

    dl_link.set_attribute("href", &file_url).unwrap_throw();
    dl_link.set_attribute("download", filename).unwrap_throw();

    window()
        .unwrap()
        .document()
        .unwrap()
        .body()
        .unwrap_throw()
        .append_child(&dl_link)
        .unwrap_throw();

    dl_link.click();

    window()
        .unwrap()
        .document()
        .unwrap()
        .body()
        .unwrap_throw()
        .remove_child(&dl_link)
        .unwrap_throw();
    Url::revoke_object_url(&file_url).unwrap_throw();
}
