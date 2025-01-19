use std::fmt::{Display, Formatter};
use crate::views::main_view::PalettePalViewModel;
use crate::views::tools::examples::dwui::dwui_example_container;
use crate::views::tools::wcag_contrast_tool::wcag_tool;
use dominator::Dom;
use futures_signals::signal::{always, Signal};
use futures_signals::signal::SignalExt;
use futures_signals::signal_map::{MutableBTreeMap, SignalMapExt};
use futures_signals::signal_vec::{SignalVec, SignalVecExt};
use palette_overview::palette_overview;
use serde::{Deserialize, Serialize};
use crate::views::tools::curve_editor::sampling_curve_editor;

pub mod curve_editor;
pub mod examples;
pub mod palette_overview;
pub mod wcag_contrast_tool;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
pub enum Tool {
    PaletteOverview,
    DwuiExample,
    WcagContrast,
    CurveEditor
}

impl Display for Tool {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Tool::PaletteOverview => write!(f, "Color Wheel"),
            Tool::DwuiExample => write!(f, "DWUI Example"),
            Tool::WcagContrast => write!(f, "WCAG Contrast"),
            Tool::CurveEditor => write!(f, "Curve Editor"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolsViewState {
    show_tools: MutableBTreeMap<Tool, bool>,
}

impl Default for ToolsViewState {
    fn default() -> Self {
        let show_tools = MutableBTreeMap::new();
        show_tools.lock_mut().insert(Tool::PaletteOverview, true);
        show_tools.lock_mut().insert(Tool::DwuiExample, true);
        show_tools.lock_mut().insert(Tool::WcagContrast, true);

        Self { show_tools }
    }
}

impl ToolsViewState {
    pub fn toggle(&self, tool: Tool) {
        let mut show = self.show_tools.lock_mut();
        let v = show.get(&tool).unwrap_or(&false);
        show.insert(tool, !*v);
    }

    pub fn tool_state_signal(&self, tool: Tool) -> impl Signal<Item=bool> {
        self.show_tools.signal_map().key_cloned(tool).map(|v| v.unwrap_or(false))
    }

    pub fn tools_children_signal(&self, vm: PalettePalViewModel) -> impl SignalVec<Item = Dom> {
        self.show_tools
            .entries_cloned()
            .filter(|v| v.1)
            .map_signal(move |(tool, _show)| match tool {
                Tool::DwuiExample => vm
                    .palette
                    .signal_cloned()
                    .map(clone!(vm => move |palette| {
                        dwui_example_container(&vm, palette)
                    }))
                    .boxed_local(),
                Tool::PaletteOverview => always(palette_overview(vm.clone())).boxed_local(),
                Tool::WcagContrast => vm
                    .palette
                    .signal_cloned()
                    .map(clone!(vm => move |palette| wcag_tool(&vm, &palette)))
                    .boxed_local(),
                Tool::CurveEditor => {
                    always(sampling_curve_editor(&vm)).boxed_local()
                }
            })
    }

    /// Returns a function that toggles the visibility of a tool
    pub fn create_close_tool_handler(&self, tool: Tool) -> Box<dyn Fn() -> () + 'static> {
        let show_tools = self.show_tools.clone();

        Box::new(move || {
            show_tools.lock_mut().insert(tool, false);
        })
    }
}
