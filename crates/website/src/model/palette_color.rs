use dwind_build::colors::Color;
use std::collections::HashMap;
use futures_signals::signal::{Mutable, Signal, SignalExt};
use futures_signals::map_ref;
use futures_signals::signal_map::{MutableBTreeMap, SignalMapExt};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasm_bindgen::UnwrapThrowExt;
use crate::model::palette::TAILWIND_NUMBERS;
use crate::model::sampling::{colors_u8, static_sample, static_sample_signal, SamplingRect};
use crate::model::sampling_curve::SamplingCurve;

pub const DWIND_CURVE: [(f32, f32); 11] = [
    (0., 1.),
    (0.2, 0.9),
    (0.3, 0.8),
    (0.4, 0.67),
    (0.47, 0.57),
    (0.55, 0.47),
    (0.62, 0.4),
    (0.7, 0.36),
    (0.8, 0.3),
    (0.9, 0.2),
    (1., 0.),
];

pub const DWIND_CURVE2: [(f32, f32); 11] = [
    (0., 1.),
    (0.2, 0.95),
    (0.3, 0.92),
    (0.4, 0.86),
    (0.47, 0.8),
    (0.55, 0.72),
    (0.62, 0.63),
    (0.7, 0.53),
    (0.8, 0.4),
    (0.9, 0.2),
    (1., 0.),
];

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum ColorSpace {
    #[default]
    HSV,
    HSL
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PaletteColor {
    pub name: Mutable<String>,
    pub hue: Mutable<f32>,
    pub sampling_rect: Mutable<SamplingRect>,
    pub sampling_curve_id: Mutable<Uuid>,
}

impl PaletteColor {
    pub fn new(hue: f32) -> Self {
        Self {
            name: Mutable::new(format!("some-color-{hue}")),
            hue: Mutable::new(hue),
            sampling_rect: Default::default(),
            sampling_curve_id: Uuid::nil().into(),
        }
    }

    /// Returns the points at which the color should be sampled in its color plane.
    /// Coordinates in this list are in the color space (hsv) coordinates
    ///
    /// The color curve is transformed to the sampling rects coordinate space, i.e.
    /// you can have a rotated or smaller rectangle to apply the full curve inside
    pub fn samples_signal(
        &self,
        sampling_curves: MutableBTreeMap<Uuid, SamplingCurve>,
    ) -> impl Signal<Item=Vec<Vec2>> + 'static {
        let sampling_rect = self.sampling_rect.clone();
        let sampling_curve = self.sampling_curve_id.signal().map(move |sampling_curve_key| {
            sampling_curves.signal_map_cloned().key_cloned(sampling_curve_key)
        }).flatten();

        let out = sampling_curve.map(move |curve: Option<SamplingCurve>| {
            let matrices_signal = sampling_rect.signal_cloned().map(|m| m.matrices_signal()).flatten();

            static_sample_signal(matrices_signal, curve.unwrap_throw().curve.signal_cloned())
        });

        out.flatten()
    }

    pub fn samples(&self, sampling_curves: &MutableBTreeMap<Uuid, SamplingCurve>) -> Vec<Vec2> {
        let sampling_rect = self.sampling_rect.get_cloned();
        let curve = sampling_curves.lock_ref().get(&self.sampling_curve_id.get_cloned()).unwrap_throw().clone();

        let matrices = sampling_rect.matrices();
        static_sample(&matrices, &curve.curve.get_cloned())
    }

    pub fn colors_u8_signal(
        &self,
        sampling_curves: &MutableBTreeMap<Uuid, SamplingCurve>,
    ) -> impl Signal<Item=Vec<(u8, u8, u8)>> {
        map_ref! {
            let shades = self.samples_signal(sampling_curves.clone()),
            let hue = self.hue.signal() => {
                colors_u8(*hue, shades)
            }
        }
    }

    pub fn colors_u8(
        &self,
        sample_coords: &Vec<Vec2>,
    ) -> Vec<(u8, u8, u8)> {
        colors_u8(self.hue.get(), sample_coords)
    }
}

impl PaletteColor {
    pub fn into_dwind_color(self, sampling_curves: &MutableBTreeMap<Uuid, SamplingCurve>) -> Option<Color> {
        let samples = self.samples(sampling_curves);
        let color = self.colors_u8(&samples).into_iter().enumerate().map(|(idx, (r, g, b))| {
            (TAILWIND_NUMBERS[idx], format!("{}", hex_color::HexColor::rgb(r, g, b).display_rgba()))
        });

        let mut shades = HashMap::new();

        for color in color {
            shades.insert(color.0, color.1);
        }

        Some(Color {
            name: self.name.get_cloned(),
            shades,
        })
    }
}
