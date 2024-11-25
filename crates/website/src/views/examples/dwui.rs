use crate::mixins::panel::panel_mixin;
use crate::model::palette::Palette;
use crate::model::palette_color::PaletteColor;
use dominator::text;
use dominator::Dom;
use dominator::DomBuilder;
use dwind::prelude::*;
use dwui::prelude::*;
use futures_signals::map_ref;
use futures_signals::signal::Mutable;
use futures_signals::signal::SignalExt;
use futures_signals::signal::{always, not, Signal};
use futures_signals::signal_vec::SignalVecExt;
use once_cell::sync::Lazy;
use std::rc::Rc;
use web_sys::HtmlElement;

pub fn dwui_example_container(palette: Palette) -> Dom {
    let curves = palette.sampling_curves.clone();
    let sampling_curves = palette.sampling_curves.clone();

    let palette = Rc::new(palette);

    struct ColorAdapter {
        palette: Rc<Palette>,
        color: Mutable<Option<PaletteColor>>,
    }

    impl InputValueWrapper for ColorAdapter {
        fn set(&self, value: String) -> ValidationResult {
            let Some(color) = self
                .palette
                .colors
                .lock_ref()
                .iter()
                .find(|v| v.name.get_cloned() == value)
                .cloned()
            else {
                return ValidationResult::Invalid {
                    message: "invalid color".to_string(),
                };
            };

            self.color.set(Some(color.clone()));

            ValidationResult::Valid
        }

        fn value_signal_cloned(&self) -> impl Signal<Item = String> + 'static {
            self.color
                .signal_cloned()
                .map(|v| {
                    v.map(|v| v.name.signal_cloned().boxed())
                        .unwrap_or(always("".to_string()).boxed())
                })
                .flatten()
        }
    }

    let tailwind_colors = palette
        .colors
        .signal_vec_cloned()
        .filter_signal_cloned(move |color| color.is_tailwind_signal(curves.clone()))
        .map_signal(|c| c.name.signal_cloned().map(|v| (v.clone(), v)))
        .to_signal_cloned()
        .broadcast();

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
            clone!(color, sampling_curves => move |b: DomBuilder<HtmlElement>| {
                let colors = color.colors_u8_signal(&sampling_curves).broadcast();

                b
                .style_signal(format!("--dwui-{color_name}-50"), colors.signal_ref(|c| format!("{}", hex_color::HexColor::rgb(c.get(0).map(|c| c.0).unwrap_or(0), c.get(0).map(|c| c.1).unwrap_or(0), c.get(0).map(|c| c.2).unwrap_or(0)).display_rgba())))
                .style_signal(format!("--dwui-{color_name}-100"), colors.signal_ref(|c| format!("{}", hex_color::HexColor::rgb(c.get(1).map(|c| c.0).unwrap_or(0), c.get(1).map(|c| c.1).unwrap_or(0), c.get(1).map(|c| c.2).unwrap_or(0)).display_rgba())))
                .style_signal(format!("--dwui-{color_name}-200"), colors.signal_ref(|c| format!("{}", hex_color::HexColor::rgb(c.get(2).map(|c| c.0).unwrap_or(0), c.get(2).map(|c| c.1).unwrap_or(0), c.get(2).map(|c| c.2).unwrap_or(0)).display_rgba())))
                .style_signal(format!("--dwui-{color_name}-300"), colors.signal_ref(|c| format!("{}", hex_color::HexColor::rgb(c.get(3).map(|c| c.0).unwrap_or(0), c.get(3).map(|c| c.1).unwrap_or(0), c.get(3).map(|c| c.2).unwrap_or(0)).display_rgba())))
                .style_signal(format!("--dwui-{color_name}-400"), colors.signal_ref(|c| format!("{}", hex_color::HexColor::rgb(c.get(4).map(|c| c.0).unwrap_or(0), c.get(4).map(|c| c.1).unwrap_or(0), c.get(4).map(|c| c.2).unwrap_or(0)).display_rgba())))
                .style_signal(format!("--dwui-{color_name}-500"), colors.signal_ref(|c| format!("{}", hex_color::HexColor::rgb(c.get(5).map(|c| c.0).unwrap_or(0), c.get(5).map(|c| c.1).unwrap_or(0), c.get(5).map(|c| c.2).unwrap_or(0)).display_rgba())))
                .style_signal(format!("--dwui-{color_name}-600"), colors.signal_ref(|c| format!("{}", hex_color::HexColor::rgb(c.get(6).map(|c| c.0).unwrap_or(0), c.get(6).map(|c| c.1).unwrap_or(0), c.get(6).map(|c| c.2).unwrap_or(0)).display_rgba())))
                .style_signal(format!("--dwui-{color_name}-700"), colors.signal_ref(|c| format!("{}", hex_color::HexColor::rgb(c.get(7).map(|c| c.0).unwrap_or(0), c.get(7).map(|c| c.1).unwrap_or(0), c.get(7).map(|c| c.2).unwrap_or(0)).display_rgba())))
                .style_signal(format!("--dwui-{color_name}-800"), colors.signal_ref(|c| format!("{}", hex_color::HexColor::rgb(c.get(8).map(|c| c.0).unwrap_or(0), c.get(8).map(|c| c.1).unwrap_or(0), c.get(8).map(|c| c.2).unwrap_or(0)).display_rgba())))
                .style_signal(format!("--dwui-{color_name}-900"), colors.signal_ref(|c| format!("{}", hex_color::HexColor::rgb(c.get(9).map(|c| c.0).unwrap_or(0), c.get(9).map(|c| c.1).unwrap_or(0), c.get(9).map(|c| c.2).unwrap_or(0)).display_rgba())))
                .style_signal(format!("--dwui-{color_name}-950"), colors.signal_ref(|c| format!("{}", hex_color::HexColor::rgb(c.get(10).map(|c| c.0).unwrap_or(0), c.get(10).map(|c| c.1).unwrap_or(0), c.get(10).map(|c| c.2).unwrap_or(0)).display_rgba())))
            }),
        )
    };

    let light_mode = Mutable::new(false);

    html!("div", {
        .dwclass!("flex-1")
        .apply(panel_mixin)
        .children([
            // All colors, with type association
            html!("div", {
                .dwclass!("flex flex-col gap-2 p-4")
                .child(html!("div", {
                    .dwclass!("flex flex-row gap-2")
                    .dwclass!("[& > :not(:nth-child(1))]:w-40")
                    .children([
                        button!({
                            .apply(|b| dwclass!(b, "w-32"))
                            .content_signal(light_mode.signal().map(|m| {
                                text(if m { "Dark mode" } else { "Light Mode"})
                            }).map(Some))
                            .on_click(clone!(light_mode => move |_| {
                                light_mode.set(!light_mode.get());
                            }))
                        }),
                        select!({
                            .label("Primary".to_string())
                            .value(ColorAdapter {palette: palette.clone(), color: primary })
                            .options_signal_vec(tailwind_colors.signal_cloned().to_signal_vec())
                        }),
                        select!({
                            .label("Void".to_string())
                            .value(ColorAdapter {palette: palette.clone(), color: void })
                            .options_signal_vec(tailwind_colors.signal_cloned().to_signal_vec())
                        }),
                        select!({
                            .label("Text on Primary".to_string())
                            .value(ColorAdapter {palette: palette.clone(), color: text_on_primary })
                            .options_signal_vec(tailwind_colors.signal_cloned().to_signal_vec())
                        }),
                        select!({
                            .label("Error".to_string())
                            .value(ColorAdapter {palette: palette.clone(), color: error })
                            .options_signal_vec(tailwind_colors.signal_cloned().to_signal_vec())
                        })
                    ])
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
