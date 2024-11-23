#[macro_use]
extern crate dominator;

#[macro_use]
extern crate dwind_macros;

#[macro_use]
extern crate dwui;

#[macro_use]
extern crate log;

pub mod mixins;
pub mod model;
pub mod views;
pub mod widgets;

use crate::views::main_view::main_view;
use dominator::{append_dom, body, stylesheet};
use dwind::colors::DWIND_COLORS;
use dwui::theme::colors::ColorsCssVariables;
use log::Level;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
async fn main() {
    dwind::stylesheet();
    wasm_log::init(wasm_log::Config::new(Level::Info));
    dwui::theme::apply_style_sheet(Some(ColorsCssVariables::new(
        &DWIND_COLORS["picton-blue"],
        &DWIND_COLORS["woodsmoke"],
        &DWIND_COLORS["woodsmoke"],
        &DWIND_COLORS["red"],
    )));
    stylesheet!(["body"], {
        .style("overflow-y", "scroll")
        .style("background-color", &dwind::colors::DWIND_COLORS["woodsmoke"][&900])
    });

    append_dom(&body(), main_view());
}
