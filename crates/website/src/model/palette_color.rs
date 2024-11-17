use dwind_build::colors::Color;
use std::collections::HashMap;
use futures_signals::signal::{always, Mutable, Signal, SignalExt};
use futures_signals::map_ref;
use serde::{Deserialize, Serialize};
use crate::model::palette::{ColorShades, DWIND_CURVE, DWIND_CURVE2, TAILWIND_NUMBERS};
use crate::model::sampling::{colors_u8, get_equidistant_points_in_range, sigmoid_sample, sigmoid_sample_signal, static_sample, static_sample_signal, ColorSampler, SamplingRect};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PaletteColor {
    pub name: Mutable<String>,
    pub hue: Mutable<f32>,
    pub sampling_rect: Mutable<SamplingRect>,
    pub sampler: Mutable<ColorSampler>,
}

impl PaletteColor {
    pub fn new(hue: f32) -> Self {
        Self {
            name: Mutable::new(format!("some-color-{hue}")),
            hue: Mutable::new(hue),
            sampling_rect: Default::default(),
            sampler: Default::default(),
        }
    }

    /// Returns the points at which the color should be sampled in its color plane.
    /// Coordinates in this list are in the color space (hsv) coordinates
    ///
    /// The color curve is transformed to the sampling rects coordinate space, i.e.
    /// you can have a rotated or smaller rectangle to apply the full curve inside
    pub fn samples_signal(
        &self,
        shades_per_color: impl Signal<Item=ColorShades> + 'static,
    ) -> impl Signal<Item=Vec<(f32, f32)>> + 'static {
        let sampling_rect = self.sampling_rect.clone();

        let out = map_ref! {
            let sampler = self.sampler.signal_cloned(),
            let shades = shades_per_color => {
                let sample_x_points = match shades {
                    ColorShades::Tailwind => {
                        get_equidistant_points_in_range(0., 1., 11)
                    }
                    ColorShades::Custom(coords) => { coords.clone() }
                };

                let matrices_signal = sampling_rect.signal_cloned().map(|m| m.matrices_signal()).flatten();

                match sampler {
                    ColorSampler::Sigmoid {amplification } => {
                        sigmoid_sample_signal(amplification.clone(), matrices_signal, always(sample_x_points.clone())).boxed()
                    }
                    ColorSampler::Diagonal => {
                        let points = sample_x_points.into_iter().map(|v| (v, 1. - v)).collect::<Vec<_>>();
                        static_sample_signal(matrices_signal, always(points)).boxed()
                    }
                    ColorSampler::DwindCurve => {
                        static_sample_signal(matrices_signal, always(DWIND_CURVE.to_vec()).boxed()).boxed()
                    }
                    ColorSampler::DwindCurve2 => {
                        static_sample_signal(matrices_signal, always(DWIND_CURVE2.to_vec()).boxed()).boxed()
                    }
                }
            }
        };

        out.flatten()
    }

    pub fn samples(&self, shades_per_color: ColorShades) -> Vec<(f32, f32)> {
        let sampling_rect = self.sampling_rect.get_cloned();
        let sample_x_points = match shades_per_color {
            ColorShades::Tailwind => {
                get_equidistant_points_in_range(0., 1., 11)
            }
            ColorShades::Custom(coords) => { coords }
        };

        let matrices = sampling_rect.matrices();
        let points = match self.sampler.get_cloned() {
            ColorSampler::Sigmoid { amplification } => {
                sigmoid_sample(&matrices, &amplification.get(), sample_x_points)
            }
            ColorSampler::Diagonal => {
                let points = sample_x_points.into_iter().map(|v| (v, 1. - v)).collect::<Vec<_>>();
                static_sample(&matrices, &points)
            }
            ColorSampler::DwindCurve => {
                static_sample(&matrices, &DWIND_CURVE.to_vec())
            }
            ColorSampler::DwindCurve2 => {
                static_sample(&matrices, &DWIND_CURVE2.to_vec())
            }
        };

        points
    }

    pub fn colors_u8_signal(
        &self,
        shades_per_color: impl Signal<Item=ColorShades> + 'static,
    ) -> impl Signal<Item=Vec<(u8, u8, u8)>> {
        map_ref! {
            let shades = self.samples_signal(shades_per_color),
            let hue = self.hue.signal() => {
                colors_u8(*hue, shades)
            }
        }
    }

    pub fn colors_u8(
        &self,
        sample_coords: &Vec<(f32, f32)>,
    ) -> Vec<(u8, u8, u8)> {
        colors_u8(self.hue.get(), sample_coords)
    }
}

impl PaletteColor {
    pub fn into_dwind_color(self, x_sample_coords: Vec<f32>) -> Color {
        let colors = match self.sampler.get_cloned() {
            ColorSampler::Sigmoid { amplification } => {
                let matrices = self.sampling_rect.get_cloned().matrices();
                let points = sigmoid_sample(&matrices, &amplification.get(), x_sample_coords);

                self.colors_u8(&points)
            }
            ColorSampler::Diagonal => {
                let matrices = self.sampling_rect.get_cloned().matrices();
                let points = x_sample_coords.into_iter().map(|v| (v, 1. - v)).collect::<Vec<_>>();
                let curve = static_sample(&matrices, &points);
                self.colors_u8(&curve)
            }
            ColorSampler::DwindCurve => {
                let matrices = self.sampling_rect.get_cloned().matrices();
                let curve = static_sample(&matrices, &DWIND_CURVE.to_vec());
                self.colors_u8(&curve)
            }
            ColorSampler::DwindCurve2 => {
                let matrices = self.sampling_rect.get_cloned().matrices();
                let curve = static_sample(&matrices, &DWIND_CURVE2.to_vec());
                self.colors_u8(&curve)
            }
        }.into_iter().enumerate().map(|(idx, (r, g, b))| {
            (TAILWIND_NUMBERS[idx], format!("{}", hex_color::HexColor::rgb(r, g, b).display_rgba()))
        });

        let mut shades = HashMap::new();

        for color in colors {
            shades.insert(color.0, color.1);
        }
        Color {
            name: self.name.get_cloned(),
            shades,
        }
    }
}
