use crate::mixins::panel::widget_panel_mixin;
use crate::views::main_view::PalettePalViewModel;
use crate::views::tools::Tool;
use dominator::{events, Dom};
use dwind::prelude::*;
use dwind_build::colors::Color;
use dwui::prelude::*;
use futures_signals::signal::{always, Mutable, SignalExt};
use web_sys::{HtmlElement, HtmlTextAreaElement};

pub fn color_import_tool(vm: &PalettePalViewModel) -> Dom {
    let palette = vm.palette.get_cloned();

    html!("div", {
        .dwclass!("p-2")
        .apply(widget_panel_mixin(always("Import Color".to_string()), Some(palette.tools_view_state.create_close_tool_handler(Tool::ColorImport))))
        .child(color_import_tool_body(&vm))
    })
}

fn color_import_tool_body(vm: &PalettePalViewModel) -> Dom {
    let json_text = Mutable::new("".to_string());
    let color_signal = json_text.signal_ref(|text| {
        serde_json::from_str::<Color>(&text)
            .map(Some)
            .unwrap_or(None)
    });

    let palette = vm.palette.clone();

    html!("div", {
        .dwclass!("flex flex-col")
        .text("Import DWIND swatch json")
        .child(html!("textarea" => HtmlTextAreaElement, {
            .attr("cols", "40")
            .attr("rows", "20")
            .with_node!(element => {
                .event(clone!(element, json_text => move |_: events::Change| {
                    json_text.set(element.value());
                }))
                .event(clone!(element, json_text => move |_: events::Input| {
                    json_text.set(element.value());
                }))
            })
        }))
        .child(button!({
            .content(Some(html!("span", { .text("Import")})))
            .disabled_signal(color_signal.map(|v| v.is_none()))
            .on_click(move |_| {
                let color = serde_json::from_str::<Color>(&json_text.get_cloned()).map(Some).unwrap_or(None).unwrap();
                palette.lock_mut().import_dwind_color(color);
            })
        }))
    })
}
