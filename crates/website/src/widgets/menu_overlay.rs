use dominator::Dom;
use futures_signals::signal::{Signal, SignalExt};
use dwind::prelude::*;

pub fn menu_overlay(top: impl Signal<Item=Dom> + 'static, inner: impl Signal<Item=Dom> + 'static) -> Dom {
    html!("div", {
        .dwclass!("grid")
        .child(html!("div", {
            .dwclass!("absolute top-0 bottom-0 left-0 right-0 overflow-y-auto")
            .dwclass!("grid-col-1 grid-row-1")
            .child_signal(inner.map(Some))
        }))

        .child(html!("div", {
            .dwclass!("grid-col-1 grid-row-1 pointer-events-none")
            .dwclass!("absolute top-0 bottom-0 left-0 right-0 overflow-y-auto")
            .child_signal(top.map(Some))
        }))
    })
}