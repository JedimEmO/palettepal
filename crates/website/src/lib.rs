#[macro_use]
extern crate dominator;

#[macro_use]
extern crate dwind_macros;

pub mod views;
pub mod widgets;

use crate::views::main_view::main_view;
use dominator::{append_dom, body, stylesheet};
use log::Level;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
async fn main() {
    dwind::stylesheet();
    wasm_log::init(wasm_log::Config::new(Level::Info));

    stylesheet!(["body"], {
        .style("background-color", &dwind::colors::DWIND_COLORS["woodsmoke"][&900])
    });

    append_dom(&body(), main_view());
}
