use crate::views::color_panel::color_panel;
use dominator::{Dom};
use dwind::prelude::*;
use futures_signals::signal::{Mutable};

pub fn main_view() -> Dom {
    let color = PaletteColor {
        hue: Mutable::new(26.),
    };

    html!("div", {
        .dwclass!("flex flex-col gap-4 justify-center w-full h-full")
        .child(color_panel(&color))
    })
}

pub struct PaletteColor {
    pub hue: Mutable<f32>,
}
