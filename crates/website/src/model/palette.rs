use crate::model::palette_color::PaletteColor;
use crate::model::sampling_curve::SamplingCurve;
use futures_signals::signal_map::MutableBTreeMap;
use futures_signals::signal_vec::MutableVec;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const TAILWIND_NUMBERS: [u32; 11] = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950];

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Palette {
    pub colors: MutableVec<PaletteColor>,
    pub sampling_curves: MutableBTreeMap<Uuid, SamplingCurve>,
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
        }
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
                .iter().filter(|&v| (v.hue.get() - hue).abs() >= 2.).cloned()
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
}
