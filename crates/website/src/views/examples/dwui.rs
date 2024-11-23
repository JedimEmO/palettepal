use crate::mixins::panel::panel_mixin;
use crate::model::palette::{Palette};
use crate::model::palette_color::PaletteColor;
use dominator::text;
use dominator::Dom;
use dominator::DomBuilder;
use dwind::prelude::*;
use dwui::prelude::*;
use futures_signals::map_ref;
use futures_signals::signal::Mutable;
use futures_signals::signal::SignalExt;
use futures_signals::signal_vec::SignalVecExt;
use web_sys::HtmlElement;
use futures_signals::signal::not;
use once_cell::sync::Lazy;

pub fn dwui_example_container(palette: Palette) -> Dom {
    let curves = palette.sampling_curves.clone();
    let sampling_curves = palette.sampling_curves.clone();

    let tailwind_colors = palette
        .colors
        .signal_vec_cloned()
        .filter_signal_cloned(move |color| color.is_tailwind_signal(curves.clone()));

    let colors_lock = palette.colors.lock_ref();
    let primary = Mutable::new(colors_lock.get(0).cloned());
    let text_on_primary = Mutable::new(colors_lock.get(0).cloned());
    let void = Mutable::new(colors_lock.get(0).cloned());
    let error = Mutable::new(colors_lock.get(0).cloned());

    let colors_signal = map_ref! {
        let primary = primary.signal_cloned(),
        let text_on_primary = text_on_primary.signal_cloned(),
        let void = void.signal_cloned(),
        let error = error.signal_cloned() => {
            (primary.clone(), text_on_primary.clone(), void.clone(), error.clone())
        }
    };

    let color_variables_mixin = move |color: Option<PaletteColor>,
                                      color_name: String|
          -> Box<
        dyn FnOnce(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement>,
    > {
        let Some(color) = color else {
            return Box::new(|b| b);
        };

        Box::new(
            clone!(color, sampling_curves => move |mut b: DomBuilder<HtmlElement>| {
                let colors = color.colors_u8_signal(&sampling_curves).broadcast();

                b
                .style_signal(format!("--dwui-{color_name}-50"), colors.signal_ref(|c| format!("{}", hex_color::HexColor::rgb(c[0].0, c[0].1, c[0].2).display_rgba())))
                .style_signal(format!("--dwui-{color_name}-100"), colors.signal_ref(|c| format!("{}", hex_color::HexColor::rgb(c[1].0, c[1].1, c[1].2).display_rgba())))
                .style_signal(format!("--dwui-{color_name}-200"), colors.signal_ref(|c| format!("{}", hex_color::HexColor::rgb(c[2].0, c[2].1, c[2].2).display_rgba())))
                .style_signal(format!("--dwui-{color_name}-300"), colors.signal_ref(|c| format!("{}", hex_color::HexColor::rgb(c[3].0, c[3].1, c[3].2).display_rgba())))
                .style_signal(format!("--dwui-{color_name}-400"), colors.signal_ref(|c| format!("{}", hex_color::HexColor::rgb(c[4].0, c[4].1, c[4].2).display_rgba())))
                .style_signal(format!("--dwui-{color_name}-500"), colors.signal_ref(|c| format!("{}", hex_color::HexColor::rgb(c[5].0, c[5].1, c[5].2).display_rgba())))
                .style_signal(format!("--dwui-{color_name}-600"), colors.signal_ref(|c| format!("{}", hex_color::HexColor::rgb(c[6].0, c[6].1, c[6].2).display_rgba())))
                .style_signal(format!("--dwui-{color_name}-700"), colors.signal_ref(|c| format!("{}", hex_color::HexColor::rgb(c[7].0, c[7].1, c[7].2).display_rgba())))
                .style_signal(format!("--dwui-{color_name}-800"), colors.signal_ref(|c| format!("{}", hex_color::HexColor::rgb(c[8].0, c[8].1, c[8].2).display_rgba())))
                .style_signal(format!("--dwui-{color_name}-900"), colors.signal_ref(|c| format!("{}", hex_color::HexColor::rgb(c[9].0, c[9].1, c[9].2).display_rgba())))
                .style_signal(format!("--dwui-{color_name}-950"), colors.signal_ref(|c| format!("{}", hex_color::HexColor::rgb(c[10].0, c[10].1, c[10].2).display_rgba())))
            }),
        )
    };

    let light_mode = Mutable::new(false);

    html!("div", {
        .apply(panel_mixin)
        .children([
            // All colors, with type association
            html!("div", {
                .dwclass!("flex flex-col gap-4")
                .child(html!("div", {
                    .children([
                        button!({
                            .apply(|b| dwclass!(b, "w-60"))
                            .content_signal(light_mode.signal().map(|m| {
                                text(if m { "Dark mode" } else { "Light Mode"})
                            }).map(Some))
                            .on_click(clone!(light_mode => move |_| {
                                light_mode.set(!light_mode.get());
                            }))
                        })
                    ])
                }))
                .child(html!("table", {
                    .dwclass!("text-woodsmoke-400 divide-y border-collapse border-woodsmoke-900 w-full text-left text-sm")
                    .child(html!("tr", {
                        .children([
                            html!("th", {
                                .dwclass!("p-b-2")
                                .text("Color name")
                            }),
                            html!("th", {
                                .dwclass!("p-b-2")
                                .text("Assignment")
                            }),
                        ])
                    }))
                    .children_signal_vec(tailwind_colors.map(move |color| {
                        html!("tr", {
                            .dwclass!("border-woodsmoke-900")
                            .children([
                                html!("td", {
                                    .dwclass!("w-60")
                                    .dwclass!("text-picton-blue-400 font-bold font-mono")
                                    .text_signal(color.name.signal_cloned())
                                }),
                                html!("td", {
                                    .dwclass!("flex flex-row gap-4 align-items-center")
                                    .dwclass!("[& > *]:p-l-2 [& > *]:p-r-2")
                                    .dwclass!("flex flex-row gap-4")
                                    .children([
                                        button!({
                                            .apply(|b| dwclass!(b, "w-24"))
                                            .content(Some(text("Primary")))
                                            .on_click(clone!(color, primary => move |_| { primary.set(Some(color.clone())) }))
                                        }),
                                        button!({
                                            .apply(|b| dwclass!(b, "w-40"))
                                            .content(Some(text("Text on Primary")))
                                            .on_click(clone!(color, text_on_primary => move |_| { text_on_primary.set(Some(color.clone())) }))
                                        }),
                                        button!({
                                            .apply(|b| dwclass!(b, "w-24"))
                                            .content(Some(text("Void")))
                                            .on_click(clone!(color, void => move |_| { void.set(Some(color.clone())) }))
                                        }),
                                        button!({
                                            .apply(|b| dwclass!(b, "w-24"))
                                            .content(Some(text("Error")))
                                            .on_click(clone!(color, error => move |_| { error.set(Some(color.clone())) }))
                                        })
                                    ])
                                })
                            ])
                        })
                    }))
                }))
            })
        ])
        .child_signal(colors_signal.map(clone!(light_mode => move |(primary, text_on_primary, void, error)| {
            Some(html!("div", {
                .dwclass!("flex flex-col gap-8")
                .apply(color_variables_mixin(primary, "primary".to_string()))
                .apply(color_variables_mixin(text_on_primary, "text-on-primary".to_string()))
                .apply(color_variables_mixin(void, "void".to_string()))
                .apply(color_variables_mixin(error, "error".to_string()))
                .child(example_ui(&light_mode))
            }))
        })))
    })
}
static SCHEME_CLASS: Lazy<String> = Lazy::new(|| {
    class! {
            .raw("background: var(--dwui-void-950)")
        }
});

static SCHEME_CLASS_LIGHT: Lazy<String> = Lazy::new(|| {
    class! {
            .raw("background: var(--dwui-void-300)")
        }
});

fn example_ui(light: &Mutable<bool>) -> Dom {
    html!("div", {
        .class_signal("light", light.signal())
        .class_signal("dark", not(light.signal()))
        .dwclass!("flex @sm:flex-col @<sm:flex-row p-8 justify-center gap-8 flex-1")
        .class_signal(&*SCHEME_CLASS_LIGHT, light.signal())
        .class_signal(&*SCHEME_CLASS, not(light.signal()))
        .children([
            card!({
                .scheme(ColorScheme::Void)
                .apply(|b| {
                    dwclass!(b, "p-8 flex flex-col gap-4")
                   .children([
                        button!({
                            .content(Some(text("Primary button")))
                        }),
                        text_input!({
                            .label("Text input".to_string())
                        }),
                        text_input!({
                            .label("Invalid text input".to_string())
                            .is_valid(ValidationResult::Invalid {message : "Something wrong".to_string()})
                        })
                   ])
                })
            })
        ])
    })
}
