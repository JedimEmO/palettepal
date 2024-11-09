use std::str::FromStr;
use crate::views::color_panel::color_panel;
use dominator::{Dom};
use dwind::prelude::*;
use futures_signals::signal::{Mutable};
use futures_signals::signal_vec::{MutableVec, SignalVecExt};
use serde::{Deserialize, Serialize};

pub fn main_view() -> Dom {
    let palette = Palette {
        shades_per_color: Mutable::new(ColorShades::Tailwind),
        colors: MutableVec::new_with_values(vec![
            PaletteColor::new(26. ),
            PaletteColor::new(26. + (360. / 5.)),
            PaletteColor::new(26. + (2. * 360. / 5.)),
            PaletteColor::new(26. + (3. * 360. / 5.)),
        ])
    };

    html!("div", {
        .dwclass!("w-screen h-screen flex align-items-center justify-center")
        .child(palette_view(palette))
    })
}

pub fn palette_view(palette: Palette) -> Dom {
    html!("div", {
        .dwclass!("flex flex-col gap-4 justify-center")
        .children_signal_vec(palette.colors.signal_vec_cloned().map(clone!(palette => move |color| {
            color_panel(color, palette.shades_per_color.read_only())
        })))
    })
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum ColorSampler {
    Sigmoid,
    #[default]
    Diagonal
}

impl ToString for ColorSampler {
    fn to_string(&self) -> String {
        match self {
            Self::Sigmoid => "Sigmoid".to_string(),
            Self::Diagonal => "Diagonal".to_string(),
        }
    }
}

impl FromStr for ColorSampler {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Sigmoid" => Ok(Self::Sigmoid),
            "Diagonal" => Ok(Self::Diagonal),
            _ => Err(())
        }
    }
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
}


#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum ColorShades {
    #[default]
    Tailwind,
    Custom(u8)
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Palette {
    pub shades_per_color: Mutable<ColorShades>,
    pub colors: MutableVec<PaletteColor>
}