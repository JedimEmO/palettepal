use std::f32::consts::PI;
use crate::model::palette::{ColorSampler, ColorShades, PaletteColor};
use crate::views::geometry::color_cake;
use dominator::Dom;
use dwind::prelude::*;
use dwui::prelude::*;
use dwui::{select, slider};
use futures_signals::signal::{Mutable, ReadOnlyMutable, SignalExt};
use futures_signals::signal_vec::SignalVecExt;
use once_cell::sync::Lazy;

static COPIED_COLOR: Lazy<Mutable<Option<PaletteColor>>> = Lazy::new(|| Mutable::new(None));

pub fn color_panel(color: PaletteColor, shades_per_color: ReadOnlyMutable<ColorShades>) -> Dom {
    let hue: Mutable<f32> = color.hue.clone();

    let shades_signal = color.colors_u8_signal(shades_per_color.signal_cloned());
    html!("div", {
        .dwclass!("p-4 bg-woodsmoke-800 @>sm:w-md @<sm:w-sm")
        .dwclass!("flex @sm:flex-row @<sm:flex-col @sm:align-items-start @<sm:align-items-center gap-4")
        .child(html!("div", {
            .dwclass!("flex flex-col gap-2")
            .children([
                color_cake(hue.clone(), color.samples_signal(shades_per_color.signal_cloned()), (512,512)),
                html!("div", {
                    .dwclass!("flex flex-row flex-wrap w-36")
                    .children_signal_vec(shades_signal.to_signal_vec().map(|shade| {
                        let color = format!("rgb({}, {}, {})", shade.0, shade.1, shade.2);

                        html!("div", {
                            .dwclass!("w-5 h-5")
                            .style("background-color", color)
                        })
                    }))
                })
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
        .child_signal(color.sampling_rect.signal_ref(|sampling_rect| {
            Some(html!("div", {
                .dwclass!("flex flex-row gap-4")
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
                })
            ])
        }))
    })
}
