use std::f32::consts::PI;
use crate::model::palette::ColorShades;
use crate::views::geometry::color_cake;
use dominator::Dom;
use dwind::prelude::*;
use dwui::prelude::*;
use dwui::{select, slider};
use futures_signals::map_ref;
use futures_signals::signal_vec::SignalVecExt;
use futures_signals::signal::{Mutable, ReadOnlyMutable, Signal, SignalExt};
use once_cell::sync::Lazy;
use crate::mixins::panel::panel_mixin;
use crate::model::palette_color::PaletteColor;
use crate::model::sampling::ColorSampler;

static COPIED_COLOR: Lazy<Mutable<Option<PaletteColor>>> = Lazy::new(|| Mutable::new(None));

pub fn color_panel(color: PaletteColor, shades_per_color: ReadOnlyMutable<ColorShades>) -> Dom {
    let hue: Mutable<f32> = color.hue.clone();

    let shades_signal = color.colors_u8_signal(shades_per_color.signal_cloned());
    let show_advanced = Mutable::new(false);

    let advanced_settings = map_ref! {
        let show_advanced = show_advanced.signal(),
        let sampling_rect = color.sampling_rect.signal_cloned() => {
            if !show_advanced {
                None
            } else {
                Some(html!("div", {
                    .dwclass!("flex flex-col gap-2 m-t-8")
                    .child(html!("div", {
                        .dwclass!("flex flex-col gap-1")
                        .children([
                            slider!({
                                .label("X".to_string())
                                .value(sampling_rect.x.clone())
                                .min(-0.5)
                                .max(1.5)
                                .step(0.1)
                            }),
                            slider!({
                                .label("Y".to_string())
                                .value(sampling_rect.y.clone())
                                .min(-0.5)
                                .max(1.5)
                                .step(0.1)
                            }),
                            slider!({
                                .label("Width".to_string())
                                .value(sampling_rect.width.clone())
                                .min(-0.5)
                                .max(1.5)
                                .step(0.1)
                            }),
                            slider!({
                                .label("Height".to_string())
                                .value(sampling_rect.height.clone())
                                .min(-0.5)
                                .max(1.5)
                                .step(0.1)
                            }),
                            slider!({
                                .label("Rotation".to_string())
                                .value(sampling_rect.rotation.clone())
                                .min(0.)
                                .max(2. * PI)
                                .step(0.1)
                            }),
                        ])
                    }))
                }))
            }
        }
    };

    html!("div", {
        .apply(panel_mixin)
        .dwclass!("p-4 @>sm:w-md @<sm:w-sm rounded")
        .child(html!("div", {
            .dwclass!("grid")
            .child(html!("div", {
                .dwclass!("flex w-full @sm:flex-row @<sm:flex-col @sm:align-items-start justify-center @<sm:align-items-center gap-4")
                .child(html!("div", {
                    .dwclass!("flex flex-col gap-2")
                    .children([
                        color_cake(hue.clone(), color.clone(), shades_per_color.clone(), (512,512)),
                    ])
                }))
                .child(html!("div", {
                    .dwclass!("flex flex-col gap-1")
                    .children([
                        text_input!({
                            .label("Color name".to_string())
                            .value(color.name.clone())
                        }),
                        slider!({
                            .label("hue".to_string())
                            .max(360.)
                            .min(0.)
                            .step(1.)
                            .value(hue.clone())
                        }),
                        select!({
                            .label("Sampler".to_string())
                            .value(color.sampler.clone())
                            .options(vec![
                                ("Sigmoid".to_string(), "Sigmoid".to_string()),
                                ("Diagonal".to_string(), "Diagonal".to_string()),
                                ("DwindCurve".to_string(), "DWIND Curve".to_string()),
                                ("DwindCurve2".to_string(), "DWIND Curve 2".to_string()),
                            ])
                        })
                    ])
                    .child_signal(color.sampler.signal_cloned().map(|sampler| {
                        match sampler {
                            ColorSampler::Sigmoid{amplification  } => {

                                Some(html!("div", {
                                    .dwclass!("flex flex-col gap-2")
                                    .children([
                                        slider!({
                                            .max(13.)
                                            .min(-13.)
                                            .step(0.1)
                                            .label("Amplification".to_string())
                                            .value(amplification.clone())
                                        })
                                    ])
                                }))

                            }
                            _ => {
                                None
                            }
                        }
                    }))
                }))
                .child(html!("div", {
                    .dwclass!("flex flex-col gap-2")
                    .children([
                        button!({
                            .content(Some(html!("div", {
                                .dwclass!("p-l-2 p-r-2")
                                .text("Copy Shape")
                            })))
                            .on_click(clone!(color => move |_| {
                                COPIED_COLOR.set(Some(color.clone()));
                            }))
                        }),
                        button!({
                            .content(Some(html!("div", {
                                .dwclass!("p-l-2 p-r-2")
                                .text("Paste")
                            })))
                            .disabled_signal(COPIED_COLOR.signal_cloned().map(|v| v.is_none()))
                            .on_click(clone!(color => move |_| {
                                let Some(copied) = COPIED_COLOR.get_cloned() else {
                                    return;
                                };

                                color.sampler.set(serde_json::from_str(&serde_json::to_string(&copied.sampler.get_cloned()).unwrap()).unwrap());
                                color.sampling_rect.set(serde_json::from_str(&serde_json::to_string(&copied.sampling_rect.get_cloned()).unwrap()).unwrap());
                            }))
                        }),
                        button!({
                            .content(Some(html!("div", { .dwclass!("p-l-2 p-r-2") .text("Advanced Settings")})))
                            .on_click(move |_| {
                                show_advanced.set(!show_advanced.get())
                            })
                        })
                    ])
                }))
            }))
            .child(horizontal_color_bar(shades_signal))
            .child_signal(advanced_settings)
        }))
    })
}

fn horizontal_color_bar(shades_signal: impl Signal<Item=Vec<(u8, u8, u8)>> + 'static) -> Dom {
    html!("div", {
        .dwclass!("flex flex-row w-full justify-center m-t-4 p-l-16 p-r-16")
        .children_signal_vec(shades_signal.to_signal_vec().map(|shade| {
            let color = format!("rgb({}, {}, {})", shade.0, shade.1, shade.2);

            html!("div", {
                .dwclass!("@sm:aspect-video @<sm:aspect-square flex-1")
                .style("background-color", color)
            })
        }))
    })
}