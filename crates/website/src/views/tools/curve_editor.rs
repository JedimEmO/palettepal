use crate::mixins::observe_size::observe_size_mixin;
use crate::mixins::panel::{panel_mixin, widget_panel_mixin};
use crate::model::palette::Palette;
use crate::model::sampling_curve::{Modifiers, SamplingCurve};
use crate::views::main_view::PalettePalViewModel;
use dominator::events::MouseButton;
use dominator::{events, Dom, EventOptions};
use dwind::prelude::*;
use dwui::prelude::*;
use futures_signals::signal::{always, Mutable, ReadOnlyMutable, Signal, SignalExt};
use futures_signals::signal_map::SignalMapExt;
use futures_signals::signal_vec::SignalVecExt;
use glam::Vec2;
use uuid::Uuid;
use crate::views::tools::Tool;

pub fn sampling_curve_editor(vm: &PalettePalViewModel) -> Dom {
    let PalettePalViewModel { palette, .. } = vm;
    let palette = palette.get_cloned();

    // nil is always present
    let selected_curve = Mutable::new(Uuid::nil());

    let all_curves = palette
        .sampling_curves
        .entries_cloned()
        .map(|(k, v)| (k, v.name))
        .to_signal_cloned()
        .to_signal_vec();

    let body = html!("div", {
        .dwclass!("flex flex-row divide-x p-4")
        // Curve selector
        .child(html!("div", {
            .dwclass!("w-60 flex flex-col flex-initial")
            .children_signal_vec(all_curves.map(clone!(selected_curve => move |(key, name)| {
                html!("div", {
                    .apply(panel_mixin)
                    .dwclass_signal!("gradient-from-picton-blue-500", selected_curve.signal().map(move |v| v == key))
                    .dwclass!("cursor-pointer text-center hover:text-picton-blue-600")
                    .text_signal(name.signal_cloned())
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
        .child_signal(curve_editor(palette.clone(), selected_curve.read_only()))
    });

    html!("div", {
        .dwclass!("p-2")
        .apply(widget_panel_mixin(always("Curve Editor".to_string()), Some(palette.tools_view_state.create_close_tool_handler(Tool::CurveEditor))))
        .child(body)
    })
}

fn curve_editor(
    palette: Palette,
    current_curve_id: ReadOnlyMutable<Uuid>,
) -> impl Signal<Item = Option<Dom>> + 'static {
    let curve_signal = current_curve_id
        .signal()
        .map(clone!(palette => move |id| {
            palette.sampling_curves.signal_map_cloned().key_cloned(id)
        }))
        .flatten();

    curve_signal.map(move |curve| {
        let Some(curve) = curve else {
            return None;
        };

        Some(curve_editor_inner(curve, true))
    })
}

pub fn curve_editor_inner(curve: SamplingCurve, meta_info: bool) -> Dom {
    let rect_sample_space_curve = curve.curve.signal_cloned();

    let rect_size = Mutable::new((0., 0.));
    let dragging_idx: Mutable<Option<usize>> = Mutable::new(None);

    html!("div", {
        .dwclass!("flex gap-4 w-full")
        .apply_if(meta_info, |b| b.child(html!("div", {
            .dwclass!("flex-initial w-60 flex flex-col gap-4")
            .children([
                text_input!({
                    .label("Curve name".to_string())
                    .value(curve.name.clone())
                })
            ])
            .child_signal(curve.curve.signal_ref(|curve_data| {
                let len = curve_data.len();

                Some(html!("div", {
                    .dwclass!("rounded-full p-2 bg-woodsmoke-300 text-woodsmoke-800 text-center")
                    .text(if len == 11 { "tailwind compatible" } else { "not tailwind compatible" })
                }))
            }))
        })))
        .child(html!("div", {
            .dwclass!("bg-picton-blue-500 aspect-square max-w-80")
            .apply(observe_size_mixin(rect_size.clone()))
            .child(svg!("svg", {
                .attr("viewBox", "0 0 512, 512")
                .attr("width", "100%")
                .attr("height", "100%")
                .event(clone!(curve, rect_size => move |event: events::DoubleClick| {
                    let x = event.offset_x();
                    let y = event.offset_y();

                    let x = x as f32 / rect_size.get().0 as f32;
                    let y = 1. - y as f32 / rect_size.get().1 as f32;

                    curve.add_new_point(Vec2::new(x, y));
                }))
                .event(clone!(curve, dragging_idx, rect_size => move |event: events::MouseMove| {
                    let x = event.offset_x();
                    let y = event.offset_y();
                    let x = x as f32 / rect_size.get().0 as f32;
                    let y = 1. - y as f32 / rect_size.get().1 as f32;

                    if event.shift_key() {
                        curve.try_y_align_at_x(Vec2::new(x, y));

                        return;
                    }

                    let idx = dragging_idx.get();

                    if idx.is_none() {
                        return;
                    }

                    let idx = idx.unwrap();

                    let idx = curve.replace_point(idx, Vec2::new(x, y), Modifiers {alt: event.alt_key(), ctrl: event.ctrl_key(), shift: event.shift_key(),});
                    dragging_idx.set(Some(idx));
                }))
                .children_signal_vec(rect_sample_space_curve.map(clone!(curve => move |curve_data| {
                    curve_data.into_iter().enumerate().map(clone!(curve, dragging_idx => move |(idx, point)| {
                        svg!("circle", {
                            .dwclass!("cursor-pointer")
                            .attr("r", "10px")
                            .attr("cx", &(point.x * 512.).to_string())
                            .attr("cy", &(512. - point.y * 512.).to_string())
                            .event(clone!(curve, dragging_idx => move |event: events::MouseDown| {
                                if event.button() == MouseButton::Left {
                                    dragging_idx.set(Some(idx));
                                } else if event.button() == MouseButton::Right {
                                    event.stop_propagation();
                                    curve.curve.lock_mut().remove(idx);
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
                })).to_signal_vec())
            }))
        }))
    })
}
