use std::collections::HashMap;
use crate::model::sampling::{algebraic_simple, get_equidistant_points_in_range};
use futures_signals::map_ref;
use futures_signals::signal::{always, BoxSignal, Mutable, Signal, SignalExt};
use futures_signals::signal_vec::MutableVec;
use hsv::hsv_to_rgb;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use dwind_build::colors::Color;
use glam::{Mat3, Vec2};

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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SamplingRect {
    pub x: Mutable<f32>,
    pub y: Mutable<f32>,
    pub width: Mutable<f32>,
    pub height: Mutable<f32>,
    pub rotation: Mutable<f32>,
}

impl Default for SamplingRect {
    fn default() -> Self {
        Self {
            x: Mutable::new(0.),
            y: Mutable::new(0.),
            width: Mutable::new(1.),
            height: Mutable::new(1.),
            rotation: Mutable::new(0.),
        }
    }
}

impl SamplingRect {
    pub fn translate_mat(&self) -> Mat3 {
        Mat3::from_translation(glam::Vec2::new(self.x.get(), self.y.get()))
    }

    pub fn scale_mat(&self) -> Mat3 {
        Mat3::from_scale(glam::Vec2::new(self.width.get(), self.height.get()))
    }

    pub fn rotate_mat(&self) -> Mat3 {
        Mat3::from_rotation_z(self.rotation.get())
    }

    pub fn matrices_signal(&self) -> impl Signal<Item=(Mat3, Mat3, Mat3)> {
        map_ref! {
            let x = self.x.signal(),
            let y = self.y.signal(),
            let width = self.width.signal(),
            let height = self.height.signal(),
            let rotation= self.rotation.signal() => {
                (Mat3::from_translation(glam::Vec2::new(*x, *y)),
                 Mat3::from_scale(glam::Vec2::new(*width, *height)),
                 Mat3::from_rotation_z(*rotation))
            }
        }
    }

    pub fn matrices(&self) -> (Mat3, Mat3, Mat3) {
        (Mat3::from_translation(glam::Vec2::new(self.x.get(), self.y.get())),
         Mat3::from_scale(glam::Vec2::new(self.width.get(), self.height.get())),
         Mat3::from_rotation_z(self.rotation.get()))
    }
}

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

        map_ref! {
            let sampler = self.sampler.signal_cloned(),
            let shades = shades_per_color => {
                match shades {
                    ColorShades::Tailwind => {
                        match sampler {
                            ColorSampler::Sigmoid {amplification } => {
                                let matrices_signal = sampling_rect.signal_cloned().map(|m| m.matrices_signal()).flatten();

                                sigmoid_sample_signal(amplification.clone(), matrices_signal).boxed()
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

impl Into<dwind_build::colors::Color> for PaletteColor {
    fn into(self) -> Color {
        const TAILWIND_NUMBERS: [u32; 11] = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950];

        let colors = match self.sampler.get_cloned() {
            ColorSampler::Sigmoid { amplification } => {
                let matrices = self.sampling_rect.get_cloned().matrices();
                let points = sigmoid_sample(&matrices, &amplification.get());

                self.colors_u8(&points)
            }
            ColorSampler::Diagonal => {
                vec![]
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

fn colors_u8(hue: f32, sample_coords: &Vec<(f32, f32)>) -> Vec<(u8, u8, u8)> {
    let mut out_colors = vec![];

    for shade in sample_coords {
        let color = hsv_to_rgb(hue as f64, shade.0 as f64, shade.1 as f64);
        out_colors.push(color);
    }

    out_colors
}

fn sigmoid_sample_signal(amplification: Mutable<f32>, sampling_rect_matrices_signal: impl Signal<Item=(Mat3, Mat3, Mat3)> + 'static) -> impl Signal<Item=Vec<(f32, f32)>> + 'static {
    map_ref! {
        let sampling_rect_matrices = sampling_rect_matrices_signal,
        let amplification = amplification.signal() => {
            sigmoid_sample(sampling_rect_matrices, amplification)
        }
    }
}

fn sigmoid_sample(sampling_rect_matrices: &(Mat3, Mat3, Mat3), amplification: &f32) -> Vec<(f32, f32)> {
    let mut points = vec![];
    let x_coords = get_equidistant_points_in_range(0., 1., 11);

    for x in x_coords {
        let x_samp = (x - 0.5) * amplification;
        let y = 0.5 - algebraic_simple(x_samp as f64)*1./2.;

        let point = Vec2::new(x.clamp(0., 1.), y.clamp(0., 1.) as f32);
        let mat = sampling_rect_matrices.0 * sampling_rect_matrices.1 * sampling_rect_matrices.2;
        let point = mat.transform_point2(point);

        points.push((point.x.clamp(0., 1.), point.y.clamp(0., 1.) as f32));
    }

    if *amplification < 0. {
        points.reverse();
    }

    points
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

impl Palette {
    pub fn add_new_color(&self) {
        let new_color = PaletteColor::new((self.colors.lock_mut().len() as  f32 * 26.) % 360.);
        self.colors.lock_mut().push_cloned(new_color);
    }
}