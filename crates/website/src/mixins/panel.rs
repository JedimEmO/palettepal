use dominator::DomBuilder;
use dwind::prelude::*;
use web_sys::Element;

pub fn panel_mixin<T: AsRef<Element>>(b: DomBuilder<T>) -> DomBuilder<T> {
    let b = dwclass!(b, "border border-woodsmoke-800 hover:border-woodsmoke-500 rounded");
    dwclass!(b, "transition-all bg-woodsmoke-900")
}
