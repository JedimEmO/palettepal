use crate::model::palette::{ColorShades, Palette, PaletteColor};
use crate::views::color_panel::color_panel;
use dominator::Dom;
use dwind::prelude::*;
use futures_signals::signal::{Mutable, SignalExt};
use futures_signals::signal_vec::{MutableVec, SignalVecExt};
use crate::views::palette_controls::palette_controls;

pub fn main_view() -> Dom {
    let palette = Mutable::new(Palette {
        shades_per_color: Mutable::new(ColorShades::Tailwind),
        colors: MutableVec::new_with_values(vec![
        ]),
    });

    html!("div", {
        .dwclass!("flex justify-center")
        .child(palette_view(palette))
    })
}

pub fn palette_view(palette: Mutable<Palette>) -> Dom {
    html!("div", {
        .dwclass!("flex flex-col gap-4 justify-center m-t-8")
        .child(palette_controls(palette.clone()))
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
