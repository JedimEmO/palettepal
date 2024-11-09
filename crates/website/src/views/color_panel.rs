use dominator::Dom;
use dwind::prelude::*;
use dwui::prelude::*;
use dwui::slider;
use futures_signals::signal::{Mutable};
use crate::views::geometry::{color_cake, color_plane};
use crate::views::main_view::PaletteColor;

pub fn color_panel(color: &PaletteColor) -> Dom {
    let hue: Mutable<f32> = color.hue.clone();

    html!("div", {
        .dwclass!("p-4 bg-woodsmoke-800 w-md")
        .dwclass!("flex flex-row gap-4")
        .child(color_plane(hue.read_only()))
        .child(color_cake(hue.read_only()))
        .child(html!("div", {
            .child(slider!({
                .label("hue".to_string())
                .max(360.)
                .min(0.)
                .step(0.1)
                .value(hue.clone())
            }))
        }))
    })
}

