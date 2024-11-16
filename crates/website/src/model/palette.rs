use std::collections::HashMap;
use crate::model::sampling::{algebraic_simple, get_equidistant_points_in_range};
use futures_signals::map_ref;
use futures_signals::signal::{always, Mutable, Signal, SignalExt};
use futures_signals::signal_vec::MutableVec;
use hsv::hsv_to_rgb;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use dwind_build::colors::Color;
use glam::{Mat3, Vec2};

const TAILWIND_NUMBERS: [u32; 11] = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950];

const DWIND_CURVE: [(f32, f32); 11] = [
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

const DWIND_CURVE2: [(f32, f32); 11] = [
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ColorSampler {
    Sigmoid {
        amplification: Mutable<f32>,
    },
    Diagonal,
    DwindCurve,
    DwindCurve2,
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
            Self::DwindCurve => "DwindCurve".to_string(),
            Self::DwindCurve2 => "DwindCurve2".to_string(),
        }
    }
}

impl FromStr for ColorSampler {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Sigmoid" => Ok(Self::Sigmoid { amplification: Mutable::new(4.) }),
            "Diagonal" => Ok(Self::Diagonal),
            "DwindCurve" => Ok(Self::DwindCurve),
            "DwindCurve2" => Ok(Self::DwindCurve2),
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
        }.flatten()
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

fn colors_u8(hue: f32, sample_coords: &Vec<(f32, f32)>) -> Vec<(u8, u8, u8)> {
    let mut out_colors = vec![];

    for shade in sample_coords {
        let color = hsv_to_rgb((hue as f64).clamp(0., 360.), shade.0 as f64, shade.1 as f64);
        out_colors.push(color);
    }

    out_colors
}

fn static_sample_signal(sampling_rect_matrices_signal: impl Signal<Item=(Mat3, Mat3, Mat3)> + 'static, points_signal: impl Signal<Item=Vec<(f32, f32)>> + 'static) -> impl Signal<Item=Vec<(f32, f32)>> + 'static {
    map_ref! {
        let points = points_signal,
        let sampling_rect_matrices = sampling_rect_matrices_signal => {
            static_sample(sampling_rect_matrices, points)
        }
    }
}

fn static_sample(sampling_rect_matrices: &(Mat3, Mat3, Mat3), input_points: &Vec<(f32, f32)>) -> Vec<(f32, f32)> {
    let mut points = vec![];

    for (x, y) in input_points {
        let point = Vec2::new(x.clamp(0., 1.), y.clamp(0., 1.));
        let trans_to_center = Mat3::from_translation(glam::Vec2::new(0.5, 0.5));
        let trans_back = Mat3::from_translation(glam::Vec2::new(-0.5, -0.5));
        // let mat = trans_to_center * sampling_rect_matrices.0 * sampling_rect_matrices.1 * sampling_rect_matrices.2 * trans_back;
        let mat = sampling_rect_matrices.0 * sampling_rect_matrices.1 * sampling_rect_matrices.2;
        let point = mat.transform_point2(point);

        points.push((point.x.clamp(0., 1.), point.y.clamp(0., 1.)));
    }

    points
}

fn sigmoid_sample_signal(amplification: Mutable<f32>, sampling_rect_matrices_signal: impl Signal<Item=(Mat3, Mat3, Mat3)> + 'static, sampling_x_coords_signal: impl Signal<Item=Vec<f32>> + 'static) -> impl Signal<Item=Vec<(f32, f32)>> + 'static {
    map_ref! {
        let sampling_rect_matrices = sampling_rect_matrices_signal,
        let amplification = amplification.signal(),
        let x_coords = sampling_x_coords_signal => {
            sigmoid_sample(sampling_rect_matrices, amplification, x_coords.clone())
        }
    }
}

fn sigmoid_sample(sampling_rect_matrices: &(Mat3, Mat3, Mat3), amplification: &f32, x_coords: Vec<f32>) -> Vec<(f32, f32)> {
    let mut points = vec![];

    for x in x_coords {
        let x_samp = (x - 0.5) * amplification;
        let y = 0.5 - algebraic_simple(x_samp as f64) * 1. / 2.;

        let point = Vec2::new(x.clamp(0., 1.), y.clamp(0., 1.) as f32);
        let trans_to_center = Mat3::from_translation(glam::Vec2::new(0.5, 0.5));
        let trans_back = Mat3::from_translation(glam::Vec2::new(-0.5, -0.5));
        let mat = trans_to_center * sampling_rect_matrices.0 * sampling_rect_matrices.1 * sampling_rect_matrices.2 * trans_back;
        let mat = sampling_rect_matrices.0 * sampling_rect_matrices.1 * sampling_rect_matrices.2;
        let point = mat.transform_point2(point);

        points.push((point.x.clamp(0., 1.), point.y.clamp(0., 1.)));
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
    Custom(Vec<f32>),
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Palette {
    pub shades_per_color: Mutable<ColorShades>,
    pub colors: MutableVec<PaletteColor>,
}

impl Palette {
    pub fn add_new_color(&self) {
        let new_color = PaletteColor::new((self.colors.lock_mut().len() as f32 * 26.).rem_euclid(360.));
        self.colors.lock_mut().push_cloned(new_color);
    }

    pub fn to_jasc_pal(&self) -> String {
        let mut palette = jascpal::Palette::new();
        let shades = self.shades_per_color.get_cloned();

        for color in self.colors.lock_mut().iter() {
            let curve = color.samples(shades.clone());
            let swatch = color.colors_u8(&curve);

            for (r, g, b) in swatch {
                palette.colors_mut().push(jascpal::Color::new(r, g, b));
            }
        }

        palette.to_string()
    }
}