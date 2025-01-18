use crate::views::main_view::PalettePalViewModel;
use crate::views::tools::examples::dwui::dwui_example_container;
use crate::views::tools::wcag_contrast_tool::wcag_tool;
use dominator::Dom;
use futures_signals::signal::always;
use futures_signals::signal::SignalExt;
use futures_signals::signal_map::MutableBTreeMap;
use futures_signals::signal_vec::{SignalVec, SignalVecExt};
use palette_overview::palette_overview;
use serde::{Deserialize, Serialize};

pub mod curve_editor;
pub mod examples;
pub mod palette_overview;
pub mod wcag_contrast_tool;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
pub enum Tool {
    PaletteOverview,
    DwuiExample,
    WcagContrast,
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
    pub fn toggle_(&self, tool: Tool) {
        let mut show = self.show_tools.lock_mut();
        let v = show.get(&tool).unwrap_or(&false);
        show.insert(tool, !*v);
    }

    pub fn tools_children_signal(&self, vm: PalettePalViewModel) -> impl SignalVec<Item = Dom> {
        self.show_tools
            .entries_cloned()
            .filter(|v| v.1)
            .map_signal(move |(tool, _show)| match tool {
                Tool::DwuiExample => vm
                    .palette
                    .signal_cloned()
                    .map(dwui_example_container)
                    .boxed_local(),
                Tool::PaletteOverview => always(palette_overview(vm.clone())).boxed_local(),
                Tool::WcagContrast => vm
                    .palette
                    .signal_cloned()
                    .map(|palette| wcag_tool(&palette))
                    .boxed_local(),
            })
    }
}
