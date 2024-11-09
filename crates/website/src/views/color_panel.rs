use dominator::Dom;
use dwind::prelude::*;
use dwui::prelude::*;
use dwui::{select, slider};
use futures_signals::map_ref;
use futures_signals::signal::{Mutable, ReadOnlyMutable, SignalExt};
use crate::views::geometry::{color_cake, color_plane};
use crate::views::main_view::{ColorSampler, ColorShades, PaletteColor};
use futures_signals::signal_vec::SignalVecExt;
use hsv::hsv_to_rgb;
use js_sys::Math::sqrt;
use log::info;

const TAILWIND_CURVE: [(f32, f32); 11] = [
    (0.0, 1.0),
    (0.09090909, 0.9090909),
    (0.18181818, 0.8181818),
    (0.27272727, 0.72727275),
    (0.36363637, 0.6363636),
    (0.45454547, 0.54545456),
    (0.54545456, 0.45454547),
    (0.6363636, 0.36363637),
    (0.72727275, 0.27272727),
    (0.8181818, 0.18181818),
    (1.0, 0.0),
];

fn get_equidistant_points_in_range(start: f32, end: f32, count: usize) -> Vec<f32> {
    let mut points = vec![];

    for idx in 0..count {
        let t = idx as f32 / (count as f32 - 1.);
        let x = start + t * (end - start);

        points.push(x);
    }

    points
}

fn algebraic_simple(x: f64) -> f64 {
    x / sqrt(1. + x.powi(2))
}

pub fn color_panel(color: PaletteColor, shades_per_color: ReadOnlyMutable<ColorShades>) -> Dom {
    let hue: Mutable<f32> = color.hue.clone();

    let shades_signal = map_ref! {
        let sampler = color.sampler.signal_cloned(),
        let shades_settings = shades_per_color.signal_cloned(),
        let hue = hue.signal() => {
            let mut colors = vec![];

            match shades_settings {
                ColorShades::Tailwind => {
                    let points = get_equidistant_points_in_range(0., 1., 11);

                    for x in points {
                        match sampler {
                            ColorSampler::Sigmoid => {
                                let y = 1. - algebraic_simple(x as f64)*1.4;

                                let hsv_color = hsv_to_rgb((*hue as f64) % 360., x as f64, y);

                                colors.push(hsv_color);
                            }
                            ColorSampler::Diagonal => {
                                let y = 1. - x;

                                let hsv_color = hsv_to_rgb((*hue as f64) % 360., x as f64, y as f64);

                                colors.push(hsv_color);
                            }
                        }
                    }
                }
                ColorShades::Custom(_) => {}
            }

            colors
        }
    };

    html!("div", {
        .dwclass!("p-4 bg-woodsmoke-800 w-md")
        .dwclass!("flex flex-row gap-4")
        // .child(color_plane(hue.read_only()))
        .child(color_cake(hue.read_only()))
        .child(html!("div", {
            .dwclass!("flex flex-col gap-2")
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
        .child(html!("div", {
            .dwclass!("flex flex-row gap-2")
            .children_signal_vec(shades_signal.to_signal_vec().map(|shade| {
                let color = format!("rgb({}, {}, {})", shade.0, shade.1, shade.2);

                html!("div", {
                    .dwclass!("w-5 h-5")
                    .text(" ")
                    .style("background-color", color)
                })
            }))
        }))
    })
}

