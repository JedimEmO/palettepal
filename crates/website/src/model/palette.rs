use crate::model::sampling::{algebraic_simple, get_equidistant_points_in_range};
use futures_signals::map_ref;
use futures_signals::signal::{always, Mutable, Signal, SignalExt};
use futures_signals::signal_vec::MutableVec;
use hsv::hsv_to_rgb;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use log::info;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ColorSampler {
    Sigmoid {
        amplification: Mutable<f32>,
    },
    Diagonal,
}

impl Default for ColorSampler {
    fn default() -> Self {
        Self::Sigmoid { amplification: (4.).into() }
    }
}

impl ToString for ColorSampler {
    fn to_string(&self) -> String {
        match self {
            Self::Sigmoid { .. } => "Sigmoid".to_string(),
            Self::Diagonal => "Diagonal".to_string(),
        }
    }
}

impl FromStr for ColorSampler {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Sigmoid" => Ok(Self::Sigmoid { amplification: Mutable::new(1.) }),
            "Diagonal" => Ok(Self::Diagonal),
            _ => Err(()),
        }
    }
}

/// Defines the rectangle within a color plane for which the sampling method applies.
/// The sampling coordinate space (0..1, 0..1) is in this rectangles local coordinate system
///
/// Note: This rectangle can be rotated...
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SamplingRect {
    pub x: Mutable<f32>,
    pub y: Mutable<f32>,
    pub width: Mutable<f32>,
    pub height: Mutable<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PaletteColor {
    pub name: Mutable<String>,
    pub hue: Mutable<f32>,
    pub sampler: Mutable<ColorSampler>,
}

impl PaletteColor {
    pub fn new(hue: f32) -> Self {
        Self {
            name: Mutable::new("Some color".to_string()),
            hue: Mutable::new(hue),
            sampler: Default::default(),
        }
    }

    /// Returns the points at which the color should be sampled in its color plane.
    ///
    /// The color curve is transformed to the sampling rects coordinate space, i.e.
    /// you can have a rotated or smaller rectangle to apply the full curve inside
    pub fn samples_signal(
        &self,
        shades_per_color: impl Signal<Item=ColorShades>,
    ) -> impl Signal<Item=Vec<(f32, f32)>> {
        map_ref! {
            let sampler = self.sampler.signal_cloned(),
            let shades = shades_per_color => {
                match shades {
                    ColorShades::Tailwind => {
                            match sampler {
                                ColorSampler::Sigmoid {amplification } => {
                                    amplification.signal().map(|amplification| {
                                        let mut points = vec![];
                                        let x_coords = get_equidistant_points_in_range(0., 1., 11);
                                        for x in x_coords {
                                            let x_samp = (x - 0.5) * amplification;
                                            let y = 0.5 - algebraic_simple(x_samp as f64)*1./2.;

                                            points.push((x.clamp(0., 1.), y.clamp(0., 1.) as f32));
                                        }

                                        points
                                    }).boxed()
                                }
                                ColorSampler::Diagonal => {
                                    let mut points = vec![];
                                    let x_coords = get_equidistant_points_in_range(0., 1., 11);

                                    for x in x_coords {
                                        let y = 1. - x;
                                        points.push((x, y));
                                    }

                                    always(points).boxed()
                                }
                            }
                    }
                    ColorShades::Custom(_) => { always(vec![]).boxed() }
                }
            }
        }.flatten()
    }

    pub fn colors_u8_signal(
        &self,
        shades_per_color: impl Signal<Item=ColorShades>,
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

fn colors_u8(hue: f32, sample_coords: &Vec<(f32, f32)>) -> Vec<(u8, u8, u8)> {
    let mut out_colors = vec![];

    for shade in sample_coords {
        let color = hsv_to_rgb(hue as f64, shade.0 as f64, shade.1 as f64);
        out_colors.push(color);
    }

    out_colors
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum ColorShades {
    #[default]
    Tailwind,
    Custom(u8),
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Palette {
    pub shades_per_color: Mutable<ColorShades>,
    pub colors: MutableVec<PaletteColor>,
}
