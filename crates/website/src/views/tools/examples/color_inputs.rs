use crate::model::palette::Palette;
use crate::model::palette_color::PaletteColor;
use dominator::Dom;
use dwui::prelude::*;
use futures_signals::signal::{always, LocalBoxSignal, Mutable, Signal, SignalExt};
use futures_signals::signal_vec::SignalVec;
use std::rc::Rc;

pub struct ColorAdapter {
    palette: Rc<Palette>,
    color: Mutable<Option<PaletteColor>>,
}
impl InputValueWrapper for ColorAdapter {
    fn set(&self, value: String) -> ValidationResult {
        let Some(color) = self
            .palette
            .colors
            .lock_ref()
            .iter()
            .find(|v| v.name.get_cloned() == value)
            .cloned()
        else {
            return ValidationResult::Invalid {
                message: "invalid color".to_string(),
            };
        };

        self.color.set(Some(color.clone()));

        ValidationResult::Valid
    }

    fn value_signal_cloned(&self) -> LocalBoxSignal<'static, String> {
        self.color
            .signal_cloned()
            .map(|v| {
                v.map(|v| v.name.signal_cloned().boxed())
                    .unwrap_or(always("".to_string()).boxed())
            })
            .flatten()
            .boxed_local()
    }
}

/// This function creates a color input dropdown for selecting a color from a given palette.
/// It takes the name of the input, a reference to the palette, a mutable reference to the selected color,
/// and a signal vector of options (color name and value pairs).
/// The selected color is wrapped in a `ColorAdapter` to handle the input value and validation.
pub fn color_input(
    name: &str,
    palette: &Rc<Palette>,
    out_color: Mutable<Option<PaletteColor>>,
    options: impl SignalVec<Item = (String, String)> + 'static,
) -> Dom {
    select!({
        .label(name.to_string())
        .value(ColorAdapter {palette: palette.clone(), color: out_color })
        .options_signal_vec(options)
    })
}
