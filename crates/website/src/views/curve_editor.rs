use dominator::{events, Dom, EventOptions};
use futures_signals::signal::{option, Mutable, ReadOnlyMutable, SignalExt};
use uuid::Uuid;
use crate::views::main_view::PalettePalViewModel;
use dwind::prelude::*;
use futures_signals::signal_vec::SignalVecExt;
use crate::mixins::panel::panel_mixin;
use dwind::prelude::*;
use dwui::prelude::InputValueWrapper;
use futures_signals::signal_map::SignalMapExt;
use glam::Vec2;
use crate::mixins::observe_size::observe_size_mixin;
use crate::model::palette::Palette;
use dominator::events::MouseButton;
use crate::model::sampling_curve::Modifiers;

pub fn sampling_curve_editor(vm: PalettePalViewModel) -> Dom {
    let PalettePalViewModel { palette, .. } = vm;
    let palette = palette.get_cloned();

    // nil is always present
    let selected_curve = Mutable::new(Uuid::nil());

    let all_curves = palette.sampling_curves.entries_cloned().map(|(k, v)| {
        (k, v.name.get_cloned())
    }).to_signal_cloned().to_signal_vec();

    html!("div", {
        .apply(panel_mixin)
        .dwclass!("flex flex-row divide-x")
        // Curve selector
        .child(html!("div", {
            .dwclass!("w-40 flex flex-col")
            .children_signal_vec(all_curves.map(clone!(selected_curve => move |(key, name)| {
                html!("div", {
                    .apply(panel_mixin)
                    .dwclass_signal!("gradient-from-picton-blue-500", selected_curve.signal().map(move |v| v == key))
                    .dwclass!("cursor-pointer text-center hover:text-picton-blue-600")
                    .text(&name)
                    .event(clone!(selected_curve => move |_: events::Click| {
                        selected_curve.set(key)
                    }))
                })
            })))
            .child(html!("div", {
                .apply(panel_mixin)
                .dwclass!("cursor-pointer text-center hover:text-picton-blue-600")
                .text(&"+")
                .event(clone!(selected_curve, palette => move |_: events::Click| {
                    let new_id = palette.add_new_curve();
                    selected_curve.set(new_id);
                }))
            }))
        }))
        // Curve editor
        .child(curve_editor(palette, selected_curve.read_only()))
    })
}

fn curve_editor(palette: Palette, current_curve_id: ReadOnlyMutable<Uuid>) -> Dom {
    let rect_size = Mutable::new((0., 0.));

    let rect_sample_space_curve = current_curve_id.signal().map(clone!(palette => move |id| {
        palette.sampling_curves.signal_map_cloned().key_cloned(id).map(|curve| {
            option(curve.map(|c| c.curve.signal_cloned()))
        }).flatten()
    })).flatten();

    let dragging_idx: Mutable<Option<usize>> = Mutable::new(None);

    html!("div", {
        .dwclass!("bg-picton-blue-500 aspect-square max-w-80")
        .apply(observe_size_mixin(rect_size.clone()))
        .child(svg!("svg", {
            .attr("viewBox", "0 0 512, 512")
            .attr("width", "100%")
            .attr("height", "100%")
            .event(clone!(current_curve_id, palette, rect_size => move |event: events::DoubleClick| {
                let x = event.offset_x();
                let y = event.offset_y();

                let x = x as f32 / rect_size.get().0 as f32;
                let y = 1. - y as f32 / rect_size.get().1 as f32;
                let curves = palette.sampling_curves.lock_mut();

                curves.get(&current_curve_id.get()).map(|curve| {
                    curve.add_new_point(Vec2::new(x, y));
                });
            }))
            .event(clone!(current_curve_id, dragging_idx, palette, rect_size => move |event: events::MouseMove| {
                let curves = palette.sampling_curves.lock_mut();
                let curve = curves.get(&current_curve_id.get());

                let x = event.offset_x();
                let y = event.offset_y();
                let x = x as f32 / rect_size.get().0 as f32;
                let y = 1. - y as f32 / rect_size.get().1 as f32;

                if event.shift_key() {
                    curve.map(|c| {
                        c.try_y_align_at_x(Vec2::new(x, y));
                    });

                    return;
                }

                let idx = dragging_idx.get();

                if idx.is_none() {
                    return;
                }

                let idx = idx.unwrap();

                curve.map(clone!(dragging_idx => move |curve| {
                    let idx = curve.replace_point(idx, Vec2::new(x, y), Modifiers {alt: event.alt_key(), ctrl: event.ctrl_key(), shift: event.shift_key(),});
                    dragging_idx.set(Some(idx));
                }));
            }))
            .children_signal_vec(rect_sample_space_curve.map(move |curve| {
                let Some(curve) = curve else {
                    return vec![svg!("circle")];
                };

                curve.into_iter().enumerate().map(clone!(current_curve_id, dragging_idx, palette => move |(idx, point)| {
                    svg!("circle", {
                        .dwclass!("cursor-pointer")
                        .attr("r", "10px")
                        .attr("cx", &(point.x * 512.).to_string())
                        .attr("cy", &(512. - point.y * 512.).to_string())
                        .event(clone!(current_curve_id, dragging_idx, palette => move |event: events::MouseDown| {
                            if event.button() == MouseButton::Left {
                                dragging_idx.set(Some(idx));
                            } else if event.button() == MouseButton::Right {
                                event.stop_propagation();
                                palette.sampling_curves.lock_ref().get(&current_curve_id.get()).map(|c| {
                                    c.curve.lock_mut().remove(idx);
                                });
                            }
                        }))
                        .event_with_options(&EventOptions {bubbles: true, preventable: true}, |event: events::ContextMenu| {
                            event.prevent_default();
                            event.stop_propagation();
                        })
                        .global_event(clone!(dragging_idx => move |_: events::MouseUp| {
                            dragging_idx.set(None);
                        }))
                    })
                })).collect()
            }).to_signal_vec())
        }))
    })
}