use crate::model::palette_color::ColorSpace::HSV;
use crate::model::palette_color::{ColorSpace, PaletteColor};
use crate::model::sampling_curve::SamplingCurve;
use crate::views::tools::ToolsViewState;
use dwind_build::colors::Color;
use futures_signals::signal::SignalExt;
use futures_signals::signal_map::MutableBTreeMap;
use futures_signals::signal_vec::{MutableVec, SignalVec, SignalVecExt};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;
use wasm_bindgen::UnwrapThrowExt;

pub const TAILWIND_NUMBERS: [u32; 11] = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950];

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Palette {
    pub colors: MutableVec<PaletteColor>,
    pub sampling_curves: MutableBTreeMap<Uuid, SamplingCurve>,
    #[serde(default)]
    pub tools_view_state: ToolsViewState,
}

impl Palette {
    pub fn new() -> Self {
        let colors = MutableVec::new_with_values(vec![PaletteColor::new(180.)]);

        let sampling_curves = MutableBTreeMap::new();
        sampling_curves
            .lock_mut()
            .insert_cloned(Uuid::nil(), SamplingCurve::tailwind_happy());

        sampling_curves
            .lock_mut()
            .insert_cloned(Uuid::from_u128(1), SamplingCurve::tailwind_diagonal());

        sampling_curves
            .lock_mut()
            .insert_cloned(Uuid::from_u128(2), SamplingCurve::pixelart_5());

        Self {
            colors,
            sampling_curves,
            tools_view_state: Default::default(),
        }
    }

    pub fn import_dwind_color(&self, color: Color) {
        let curve_id = Uuid::new_v4();
        let mut curve = SamplingCurve::new();
        curve.curve.lock_mut().clear();

        curve.name.set(format!("{}_curve", color.name));

        let mut hue = 0.;

        for (shade, color) in color.shades {
            let hex =
                hex_color::HexColor::parse(&color).expect_throw("Failed to parse color {shade}");
            let hsv = rgb_hsv::rgb_to_hsv((
                hex.r as f32 / 256.,
                hex.g as f32 / 256.,
                hex.b as f32 / 256.,
            ));
            hue = hsv.0 * 360.;
            curve.curve.lock_mut().push(Vec2::new(hsv.1, hsv.2))
        }

        curve.sort();

        self.sampling_curves
            .lock_mut()
            .insert_cloned(curve_id, curve);

        let mut palette_color = PaletteColor::new(hue);
        palette_color.name.set(color.name);
        palette_color.sampling_curve_id.set(curve_id);
        palette_color.color_space.set(ColorSpace::HSV);

        self.colors.lock_mut().push_cloned(palette_color);
    }

    pub fn add_new_curve(&self) -> Uuid {
        let id = Uuid::new_v4();
        let new_curve = SamplingCurve::new();
        self.sampling_curves.lock_mut().insert_cloned(id, new_curve);

        id
    }

    pub fn add_new_color(&self) {
        let new_color =
            PaletteColor::new((self.colors.lock_mut().len() as f32 * 26.).rem_euclid(360.));
        self.colors.lock_mut().push_cloned(new_color);
    }

    pub fn add_new_color_hue(&self, hue: f32) {
        let new_color = PaletteColor::new(hue.rem_euclid(360.));
        self.colors.lock_mut().push_cloned(new_color);
    }

    pub fn remove_hue(&self, hue: f32) {
        let mut colors = self.colors.lock_mut();

        if colors.len() < 1 {
            return;
        }

        colors.replace_cloned(
            colors
                .iter()
                .filter(|&v| (v.hue.get() - hue).abs() >= 2.)
                .cloned()
                .collect::<Vec<_>>(),
        );
    }

    pub fn to_jasc_pal(&self) -> String {
        let mut palette = jascpal::Palette::new();

        for color in self.colors.lock_mut().iter() {
            let curve = color.samples(&self.sampling_curves);
            let swatch = color.colors_u8(&curve);

            for (r, g, b) in swatch {
                palette.colors_mut().push(jascpal::Color::new(r, g, b));
            }
        }

        palette.to_string()
    }

    pub fn palette_colors_signal(&self) -> impl SignalVec<Item = (u8, u8, u8)> {
        let curves = self.sampling_curves.clone();

        self.colors.signal_vec_cloned().map(clone!(curves => move |color| {
            color.colors_u8_signal(&curves).throttle(|| gloo_timers::future::sleep(Duration::from_millis(500))).to_signal_vec()
        })).flatten()
    }
}
