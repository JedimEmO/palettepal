use futures_signals::signal::{Mutable};
use futures_signals::signal_vec::MutableVec;
use serde::{Deserialize, Serialize};
use crate::model::palette_color::PaletteColor;

pub const TAILWIND_NUMBERS: [u32; 11] = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950];

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