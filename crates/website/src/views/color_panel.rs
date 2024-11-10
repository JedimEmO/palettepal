use std::f32::consts::PI;
use crate::model::palette::{ColorSampler, ColorShades, PaletteColor};
use crate::views::geometry::color_cake;
use dominator::Dom;
use dwind::prelude::*;
use dwui::prelude::*;
use dwui::{select, slider, text_input};
use futures_signals::signal::{Mutable, ReadOnlyMutable, SignalExt};
use futures_signals::signal_vec::SignalVecExt;

pub fn color_panel(color: PaletteColor, shades_per_color: ReadOnlyMutable<ColorShades>) -> Dom {
    let hue: Mutable<f32> = color.hue.clone();

    let shades_signal = color.colors_u8_signal(shades_per_color.signal_cloned());

    html!("div", {
        .children([
            html!("div", {
                .dwclass!("p-4 bg-woodsmoke-800 w-lg")
                .dwclass!("flex flex-row gap-4")
                .child(html!("div", {
                    .dwclass!("flex flex-col gap-2")
                    .children([
                        color_cake(hue.read_only(), color.samples_signal(shades_per_color.signal_cloned()), (1024,1024))
                    ])
                }))
                .child(html!("div", {
                    .dwclass!("flex flex-col gap-2 w-52")
                    .children([
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
                                ("Diagonal".to_string(), "Diagonal".to_string())
                            ])
                        })
                    ])
                }))
                .child_signal(color.sampler.signal_cloned().map(|sampler| {
                    match sampler {
                        ColorSampler::Sigmoid{amplification  } => {

                            Some(html!("div", {
                                .dwclass!("flex flex-col gap-2 w-52")
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
                        ColorSampler::Diagonal => {
                            None
                        }
                    }
                }))
                .child_signal(color.sampling_rect.signal_ref(|sampling_rect| {
                    Some(html!("div", {
                        .dwclass!("flex flex-row gap-4")
                        .child(html!("div", {
                            .dwclass!("flex flex-col gap-2")
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
                                })
                            ])
                        }))
                        .child(html!("div", {
                            .dwclass!("flex flex-col gap-2")
                            .children([
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
            }),
            html!("div", {
                .dwclass!("p-4 bg-woodsmoke-800 w-lg")
                .dwclass!("flex flex-row gap-4")
                .child(html!("div", {
                    .dwclass!("flex flex-row flex-wrap")
                    .children_signal_vec(shades_signal.to_signal_vec().map(|shade| {
                        let color = format!("rgb({}, {}, {})", shade.0, shade.1, shade.2);

                        html!("div", {
                            .dwclass!("w-5 h-5")
                            .style("background-color", color)
                        })
                    }))
                }))
            })
        ])
    })
}
