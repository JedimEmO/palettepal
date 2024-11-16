use std::sync::Arc;
use dominator::{events, DomBuilder};
use wasm_bindgen::JsCast;
use web_sys::{Node};

pub fn click_outside_collapse_mixin<T: AsRef<Node> + Clone + 'static>(cb: impl Fn() -> () + 'static) -> impl FnMut(DomBuilder<T>) -> DomBuilder<T> {
    let cb = Arc::new(cb);
    move |b| {
        with_node!(b, element => {
            .global_event(clone!(cb => move |event: events::Click| {
                if let Some(target) = event.target().as_ref() {
                    if let Some(target) = target.dyn_ref::<Node>() {
                        if !element.as_ref().contains(Some(target)) {
                            cb();
                        }
                    }
                }
            }))
        })
    }
}