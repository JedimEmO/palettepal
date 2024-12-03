use futures_signals::map_ref;
use futures_signals::signal::{Mutable, Signal};
use glam::{Mat3, Vec2};
use hsv::hsv_to_rgb;
use js_sys::Math::sqrt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub fn get_equidistant_points_in_range(start: f32, end: f32, count: usize) -> Vec<f32> {
    let mut points = vec![];

    for idx in 0..count {
        let t = idx as f32 / (count as f32 - 1.);
        let x = start + t * (end - start);

        points.push(x);
    }

    points
}

pub fn algebraic_simple(x: f64) -> f64 {
    x / sqrt(1. + x.powi(2))
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
            x: Mutable::new(0.13),
            y: Mutable::new(0.13),
            width: Mutable::new(0.75),
            height: Mutable::new(0.75),
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

    pub fn matrices_signal(&self) -> impl Signal<Item = (Mat3, Mat3, Mat3)> {
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
        (
            Mat3::from_translation(glam::Vec2::new(self.x.get(), self.y.get())),
            Mat3::from_scale(glam::Vec2::new(self.width.get(), self.height.get())),
            Mat3::from_rotation_z(self.rotation.get()),
        )
    }
}

pub fn hsv_colors_u8(hue: f32, angle: f32, sample_coords: &Vec<Vec2>) -> Vec<(u8, u8, u8)> {
    let mut out_colors = vec![];

    for shade in sample_coords {
        // Shift the hue based on the color plane angle
        let color_hue = (hue - angle * shade.y).rem_euclid(360.);
        let color = hsv_to_rgb(color_hue as f64, shade.x as f64, shade.y as f64);
        out_colors.push(color);
    }

    out_colors
}

pub fn hsl_colors_u8(hue: f32, angle: f32, sample_coords: &Vec<Vec2>) -> Vec<(u8, u8, u8)> {
    let mut out_colors = vec![];

    for shade in sample_coords {
        // Shift the hue based on the color plane angle
        let color_hue = hue - angle * shade.y;
        let color = hsl::HSL {
            h: (color_hue as f64).rem_euclid(360.),
            s: shade.x as f64,
            l: shade.y as f64,
        }
        .to_rgb();
        out_colors.push(color);
    }

    out_colors
}

pub fn static_sample_signal(
    sampling_rect_matrices_signal: impl Signal<Item = (Mat3, Mat3, Mat3)> + 'static,
    points_signal: impl Signal<Item = Vec<Vec2>> + 'static,
) -> impl Signal<Item = Vec<Vec2>> + 'static {
    map_ref! {
        let points = points_signal,
        let sampling_rect_matrices = sampling_rect_matrices_signal => {
            static_sample(sampling_rect_matrices, points)
        }
    }
}

pub fn static_sample(
    sampling_rect_matrices: &(Mat3, Mat3, Mat3),
    input_points: &Vec<Vec2>,
) -> Vec<Vec2> {
    let mut points = vec![];

    for point in input_points {
        let point = Vec2::new(point.x.clamp(0., 1.), point.y.clamp(0., 1.));
        let mat = sampling_rect_matrices.0 * sampling_rect_matrices.1 * sampling_rect_matrices.2;
        let point = mat.transform_point2(point);

        points.push(Vec2::new(point.x.clamp(0., 1.), point.y.clamp(0., 1.)));
    }

    points
}
