use crate::model::palette::TAILWIND_NUMBERS;
use crate::model::sampling::{
    hsl_colors_u8, hsv_colors_u8, static_sample, static_sample_signal, SamplingRect,
};
use crate::model::sampling_curve::SamplingCurve;
use dwind_build::colors::Color;
use futures_signals::map_ref;
use futures_signals::signal::{Mutable, Signal, SignalExt};
use futures_signals::signal_map::{MutableBTreeMap, SignalMapExt};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use uuid::Uuid;
use wasm_bindgen::UnwrapThrowExt;

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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default)]
pub enum ColorSpace {
    #[default]
    HSV,
    HSL,
}

impl Display for ColorSpace {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorSpace::HSV => write!(f, "HSV"),
            ColorSpace::HSL => write!(f, "HSL"),
        }
    }
}

impl FromStr for ColorSpace {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HSV" => Ok(Self::HSV),
            "HSL" => Ok(Self::HSL),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PaletteColor {
    pub name: Mutable<String>,
    pub hue: Mutable<f32>,
    pub color_space: Mutable<ColorSpace>,
    pub sampling_rect: Mutable<SamplingRect>,
    pub sampling_curve_id: Mutable<Uuid>,
    pub cake_type: Mutable<CakeType>,
    pub color_plane_angle: Mutable<f32>,
}

impl PaletteColor {
    pub fn new(hue: f32) -> Self {
        Self {
            name: Mutable::new(format!("some-color-{hue}")),
            hue: Mutable::new(hue),
            color_space: Default::default(),
            sampling_rect: Default::default(),
            sampling_curve_id: Uuid::nil().into(),
            cake_type: Default::default(),
            color_plane_angle: Default::default(),
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
    ) -> impl Signal<Item = Vec<Vec2>> + 'static {
        let sampling_rect = self.sampling_rect.clone();
        let sampling_curve = self
            .sampling_curve_id
            .signal()
            .map(move |sampling_curve_key| {
                sampling_curves
                    .signal_map_cloned()
                    .key_cloned(sampling_curve_key)
            })
            .flatten();

        let out = sampling_curve.map(move |curve: Option<SamplingCurve>| {
            let matrices_signal = sampling_rect
                .signal_cloned()
                .map(|m| m.matrices_signal())
                .flatten();

            static_sample_signal(matrices_signal, curve.unwrap_throw().curve.signal_cloned())
        });

        out.flatten()
    }

    pub fn samples(&self, sampling_curves: &MutableBTreeMap<Uuid, SamplingCurve>) -> Vec<Vec2> {
        let sampling_rect = self.sampling_rect.get_cloned();
        let curve = sampling_curves
            .lock_ref()
            .get(&self.sampling_curve_id.get_cloned())
            .unwrap_throw()
            .clone();

        let matrices = sampling_rect.matrices();
        static_sample(&matrices, &curve.curve.get_cloned())
    }

    pub fn colors_u8_signal(
        &self,
        sampling_curves: &MutableBTreeMap<Uuid, SamplingCurve>,
    ) -> impl Signal<Item = Vec<(u8, u8, u8)>> {
        map_ref! {
            let shades = self.samples_signal(sampling_curves.clone()),
            let space = self.color_space.signal(),
            let angle = self.color_plane_angle.signal(),
            let hue = self.hue.signal() => {
                match space {
                    ColorSpace::HSL => hsl_colors_u8(*hue, *angle, shades),
                    ColorSpace::HSV => hsv_colors_u8(*hue, *angle, shades),
                }
            }
        }
    }

    pub fn colors_u8(&self, sample_coords: &Vec<Vec2>) -> Vec<(u8, u8, u8)> {
        match self.color_space.get() {
            ColorSpace::HSL => {
                hsl_colors_u8(self.hue.get(), self.color_plane_angle.get(), sample_coords)
            }
            ColorSpace::HSV => {
                hsv_colors_u8(self.hue.get(), self.color_plane_angle.get(), sample_coords)
            }
        }
    }

    pub fn is_tailwind_signal(
        &self,
        sampling_curves: MutableBTreeMap<Uuid, SamplingCurve>,
    ) -> impl Signal<Item = bool> + 'static {
        self.sampling_curve_id
            .signal_cloned()
            .map(move |sampling_curve_key| {
                sampling_curves
                    .lock_ref()
                    .get(&sampling_curve_key)
                    .unwrap_throw()
                    .curve
                    .signal_ref(|v| v.len() == 11)
            })
            .flatten()
    }
}

impl PaletteColor {
    pub fn into_dwind_color(
        self,
        sampling_curves: &MutableBTreeMap<Uuid, SamplingCurve>,
    ) -> Option<Color> {
        let samples = self.samples(sampling_curves);
        let color = self
            .colors_u8(&samples)
            .into_iter()
            .enumerate()
            .map(|(idx, (r, g, b))| {
                (
                    TAILWIND_NUMBERS[idx],
                    format!("{}", hex_color::HexColor::rgb(r, g, b).display_rgba()),
                )
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum CakeType {
    Cylinder,
    Brick,
}

impl Default for CakeType {
    fn default() -> Self {
        Self::Cylinder
    }
}

impl Display for CakeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CakeType::Cylinder => write!(f, "Cylinder"),
            CakeType::Brick => write!(f, "Brick"),
        }
    }
}

impl FromStr for CakeType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Cylinder" => Ok(Self::Cylinder),
            "Brick" => Ok(Self::Brick),
            _ => Err(()),
        }
    }
}
