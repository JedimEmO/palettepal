use std::f32::consts::PI;
use crate::views::geometry::color_cake;
use dominator::Dom;
use dwind::prelude::*;
use dwui::prelude::*;
use dwui::{select, slider};
use futures_signals::map_ref;
use futures_signals::signal_vec::SignalVecExt;
use futures_signals::signal::{Mutable, Signal, SignalExt};
use futures_signals::signal_map::MutableBTreeMap;
use once_cell::sync::Lazy;
use uuid::Uuid;
use crate::mixins::panel::panel_mixin;
use crate::model::palette_color::PaletteColor;
use crate::model::sampling_curve::SamplingCurve;
use crate::views::svg_icons::{svg_button, Icons};

static COPIED_COLOR: Lazy<Mutable<Option<PaletteColor>>> = Lazy::new(|| Mutable::new(None));

pub fn color_panel(color: PaletteColor, sampling_curves: MutableBTreeMap<Uuid, SamplingCurve>) -> Dom {
    let hue: Mutable<f32> = color.hue.clone();
    let hue2: Mutable<f32> = color.hue.clone();

    let shades_signal = color.colors_u8_signal(&sampling_curves);
    let show_advanced = Mutable::new(false);

    let advanced_settings = map_ref! {
        let show_advanced = show_advanced.signal(),
        let sampling_rect = color.sampling_rect.signal_cloned() => {
            if !show_advanced {
                None
            } else {
                Some(html!("div", {
                    .dwclass!("flex @sm:flex-row @<sm:flex-col gap-2 m-t-8 justify-center")
                    .child(html!("div", {
                        .dwclass!("flex flex-col gap-1 w-72")
                        .child(text_input!({
                            .label("Hex hue".to_string())
                            .value(HueHexWrapper(hue2.clone()))
                        }))
                    }))
                    .child(html!("div", {
                        .dwclass!("flex flex-col gap-1 w-72")
                        .children([
                            slider!({
                                .label("X".to_string())
                                .value(sampling_rect.x.clone())
                                .min(-0.5)
                                .max(1.5)
                                .step(0.1)
                            }),
                            slider!({
                                .label("Y".to_string())
                                .value(sampling_rect.y.clone())
                                .min(-0.5)
                                .max(1.5)
                                .step(0.1)
                            }),
                            slider!({
                                .label("Width".to_string())
                                .value(sampling_rect.width.clone())
                                .min(-0.5)
                                .max(1.5)
                                .step(0.1)
                            }),
                            slider!({
                                .label("Height".to_string())
                                .value(sampling_rect.height.clone())
                                .min(-0.5)
                                .max(1.5)
                                .step(0.1)
                            }),
                            slider!({
                                .label("Rotation".to_string())
                                .value(sampling_rect.rotation.clone())
                                .min(0.)
                                .max(2. * PI)
                                .step(0.1)
                            }),
                        ])
                    }))
                }))
            }
        }
    };

    html!("div", {
        .apply(panel_mixin)
        .dwclass!("p-4 @>sm:w-md @<sm:w-sm rounded")
        .child(html!("div", {
            .dwclass!("grid")
            .child(html!("div", {
                .dwclass!("grid-col-1 grid-row-1")
                .dwclass!("flex w-full @sm:flex-row @<sm:flex-col @sm:align-items-start justify-center @<sm:align-items-center gap-4")
                .child(html!("div", {
                    .dwclass!("flex flex-col gap-2")
                    .children([
                        color_cake(hue.clone(), color.clone(), &sampling_curves, (512,512)),
                    ])
                }))
                .child(html!("div", {
                    .dwclass!("flex flex-col gap-1")
                    .children([
                        text_input!({
                            .label("Color name".to_string())
                            .value(color.name.clone())
                        }),
                        slider!({
                            .label("hue".to_string())
                            .max(360.)
                            .min(0.)
                            .step(1.)
                            .value(hue.clone())
                        }),
                        select!({
                            .label("Sampler".to_string())
                            .value(color.sampling_curve_id.clone())
                            .options_signal_vec(sampling_curves.entries_cloned().map(|(key, curve)| {
                                (key.to_string(), curve.name.get_cloned())
                            }).to_signal_cloned().to_signal_vec())
                        }),
                        select!({
                            .label("Color Space".to_string())
                            .value(color.color_space.clone())
                            .options(vec![
                                ("HSV".to_string(), "HSV".to_string()),
                                ("HSL".to_string(), "HSL".to_string()),
                            ])
                        }),
                        slider!({
                            .label("Color plane angle".to_string())
                            .max(360.0_f32)
                            .min(-360.0_f32)
                            .step(0.5)
                            .value(color.color_plane_angle.clone())
                        }),
                    ])
                }))
            }))
            .child(html!("div", {
                .dwclass!("grid-col-1 grid-row-1 flex justify-end pointer-events-none")
                .child(html!("div", {
                    .dwclass!("pointer-events-auto flex flex-col")
                    .children([
                        svg_button(Icons::Copy, "Copy color settings", clone!(color => move |_| {
                            COPIED_COLOR.set(Some(color.clone()));
                        }), |b| b),
                        svg_button(Icons::Paste, "Paste color settings", clone!(color => move |_| {
                            let Some(copied) = COPIED_COLOR.get_cloned() else {
                                return;
                            };

                            color.sampling_curve_id.set(copied.sampling_curve_id.get());
                            color.color_space.set(copied.color_space.get());
                            color.sampling_rect.set(serde_json::from_str(&serde_json::to_string(&copied.sampling_rect.get_cloned()).unwrap()).unwrap());
                        }), |b| {
                            dwclass_signal!(b, "fill-woodsmoke-500", COPIED_COLOR.signal_cloned().map(|v| v.is_none()))
                        }),
                        svg_button(Icons::Edit, "Advanced settings", move |_| {
                                show_advanced.set(!show_advanced.get())
                        },|b| b)
                    ])
                }))
            }))
            .child(horizontal_color_bar(shades_signal))
            .child_signal(advanced_settings)
        }))
})
}

fn horizontal_color_bar(shades_signal: impl Signal<Item=Vec<(u8, u8, u8)>> + 'static) -> Dom {
    html!("div", {
        .dwclass!("flex flex-row w-full justify-center m-t-4 p-l-16 p-r-16")
        .children_signal_vec(shades_signal.to_signal_vec().map(|shade| {
            let color = format!("rgb({}, {}, {})", shade.0, shade.1, shade.2);

            html!("div", {
                .dwclass!("@sm:aspect-video @<sm:aspect-square flex-1 max-h-12")
                .style("background-color", color)
            })
        }))
    })
}

struct HueHexWrapper(Mutable<f32>);

impl InputValueWrapper for HueHexWrapper {
    fn set(&self, value: String) -> ValidationResult {
        let Ok(hex) = hex_color::HexColor::parse(&value) else {
            return ValidationResult::Invalid { message: "Invalid hex color".to_string() };
        };

        let hsl = hsl::HSL::from_rgb(&[hex.r, hex.g, hex.b]);

        self.0.set(hsl.h as f32);

        ValidationResult::Valid
    }

    fn value_signal_cloned(&self) -> impl Signal<Item=String> + 'static {
        self.0.signal_cloned().map(|h| {
            let hsl = hsl::HSL { h: h as f64, s: 1., l: 0.5 };
            let rgb = hsl.to_rgb();
            format!("#{:02x}{:02x}{:02x}", rgb.0, rgb.1, rgb.2)
        })
    }
}