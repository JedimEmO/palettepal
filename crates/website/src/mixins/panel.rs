use dominator::DomBuilder;
use dwind::prelude::*;
use web_sys::Element;

pub fn panel_mixin<T: AsRef<Element>>(b: DomBuilder<T>) -> DomBuilder<T> {
    let b = dwclass!(b, "border border-woodsmoke-600 hover:border-woodsmoke-400");
    dwclass!(b, "transition-all linear-gradient-135 gradient-from-woodsmoke-700 hover:gradient-from-woodsmoke-700 gradient-to-woodsmoke-950 hover:gradient-to-woodsmoke-900")
}
