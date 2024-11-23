use crate::mixins::panel::panel_mixin;
use crate::model::palette::{Palette, TAILWIND_NUMBERS};
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
use wasm_bindgen::UnwrapThrowExt;
use web_sys::HtmlElement;

pub fn dwui_example_container(palette: Palette) -> Dom {
    let curves = palette.sampling_curves.clone();
    let sampling_curves = palette.sampling_curves.clone();

    let tailwind_colors = palette
        .colors
        .signal_vec_cloned()
        .filter_signal_cloned(move |color| color.is_tailwind_signal(curves.clone()));

    let primary = Mutable::new(None::<PaletteColor>);
    let text_on_primary = Mutable::new(None::<PaletteColor>);
    let void = Mutable::new(None::<PaletteColor>);
    let error = Mutable::new(None::<PaletteColor>);

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

    html!("div", {
        .apply(panel_mixin)
        .children([
            // All colors, with type association
            html!("div", {
                .dwclass!("flex flex-col gap-4 @>sm:w-md @<sm:w-sm")
                .children_signal_vec(tailwind_colors.map(move |color| {
                    html!("div", {
                        .dwclass!("flex flex-row gap-4 align-items-center")
                        .dwclass!("[& > *]:p-l-2 [& > *]:p-r-2")
                        .children([
                            html!("div", {
                                .dwclass!("w-60")
                                .text(&color.name.get_cloned())
                            }),
                            html!("div", {
                                .dwclass!("flex flex-row gap-4")
                                .children([
                                    button!({
                                        .apply(|b| dwclass!(b, "w-32"))
                                        .content(Some(text("Primary")))
                                        .on_click(clone!(color, primary => move |_| { primary.set(Some(color.clone())) }))
                                    }),
                                    button!({
                                        .apply(|b| dwclass!(b, "w-40"))
                                        .content(Some(text("Text on Primary")))
                                        .on_click(clone!(color, text_on_primary => move |_| { text_on_primary.set(Some(color.clone())) }))
                                    }),
                                    button!({
                                        .apply(|b| dwclass!(b, "w-32"))
                                        .content(Some(text("Void")))
                                        .on_click(clone!(color, void => move |_| { void.set(Some(color.clone())) }))
                                    }),
                                    button!({
                                        .apply(|b| dwclass!(b, "w-32"))
                                        .content(Some(text("Error")))
                                        .on_click(clone!(color, error => move |_| { error.set(Some(color.clone())) }))
                                    })
                                ])
                            })
                        ])
                    })
                }))
            })
        ])
        .child_signal(colors_signal.map(move |(primary, text_on_primary, void, error)| {
            Some(html!("div", {
                .dwclass!("flex flex-col gap-8")
                .apply(color_variables_mixin(primary, "primary".to_string()))
                .apply(color_variables_mixin(text_on_primary, "text-on-primary".to_string()))
                .apply(color_variables_mixin(void, "void".to_string()))
                .apply(color_variables_mixin(error, "error".to_string()))
                .children([
                    example_ui(false),
                    example_ui(true),
                ])
            }))
        }))
    })
}

fn example_ui(light: bool) -> Dom {
    html!("div", {
        .class( if light { "light" } else {"dark"})
        .dwclass!("flex @sm:flex-col @<sm:flex-row p-8 justify-center gap-8 flex-1")
        .class(class!{
            .raw(if light { "background: var(--dwui-void-300)" } else { "background: var(--dwui-void-950)" })
        })
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
