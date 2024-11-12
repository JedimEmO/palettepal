use crate::model::palette::{ColorSampler, ColorShades, Palette, PaletteColor};
use crate::views::color_panel::color_panel;
use dominator::Dom;
use dwind::prelude::*;
use futures_signals::signal::{always, Mutable, SignalExt};
use futures_signals::signal_vec::{MutableVec, SignalVecExt};
use crate::views::palette_controls::palette_controls;
use crate::widgets::menu_overlay::menu_overlay;

pub fn main_view() -> Dom {
    let palette = Mutable::new(Palette {
        shades_per_color: Mutable::new(ColorShades::Tailwind),
        colors: MutableVec::new_with_values(vec![
            PaletteColor {
                name: "default-color".to_string().into(),
                hue: Default::default(),
                sampling_rect: Default::default(),
                sampler: Mutable::new(ColorSampler::DwindCurve),
            }
        ]),
    });

    menu_overlay(
        always(palette_controls(palette.clone())),
        always(html!("div", {
            .dwclass!("flex justify-center w-full h-screen align-items-start")
            .dwclass!("linear-gradient-180 gradient-from-woodsmoke-800 gradient-to-woodsmoke-950")
            .child(palette_view(palette))
        })),
    )
}

pub fn palette_view(palette: Mutable<Palette>) -> Dom {
    html!("div", {
        .dwclass!("flex flex-col gap-4 justify-center m-t-16")
        .child_signal(palette.signal_ref(|palette| {
            Some(html!("div", {
                .dwclass!("flex flex-col gap-4 ")
                .children_signal_vec(palette.colors.signal_vec_cloned().map(clone!(palette => move |color| {
                    color_panel(color, palette.shades_per_color.read_only())
                })))
            }))
        }))
    })
}
