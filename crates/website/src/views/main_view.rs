use crate::model::palette::{ColorShades, Palette, PaletteColor};
use crate::views::color_panel::color_panel;
use dominator::Dom;
use dwind::prelude::*;
use futures_signals::signal::Mutable;
use futures_signals::signal_vec::{MutableVec, SignalVecExt};

pub fn main_view() -> Dom {
    let palette = Palette {
        shades_per_color: Mutable::new(ColorShades::Tailwind),
        colors: MutableVec::new_with_values(vec![
            PaletteColor::new(26.),
            PaletteColor::new(26.),
            PaletteColor::new(26. + (2. * 360. / 5.)),
            PaletteColor::new(26. + (3. * 360. / 5.)),
        ]),
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
