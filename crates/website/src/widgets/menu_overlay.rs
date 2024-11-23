use dominator::Dom;
use dwind::prelude::*;
use futures_signals::signal::{Signal, SignalExt};

pub fn menu_overlay(
    top: impl Signal<Item = Dom> + 'static,
    inner: impl Signal<Item = Dom> + 'static,
) -> Dom {
    html!("div", {
        .dwclass!("grid")
        .child(html!("div", {
            .dwclass!("@sm:absolute @sm:top-0 @sm:bottom-0 @sm:left-0 @sm:right-0 @sm:overflow-y-auto")
            .dwclass!("grid-col-1 @sm:grid-row-1 @<sm:grid-row-2")
            .child_signal(inner.map(Some))
        }))

        .child(html!("div", {
            .dwclass!("grid-col-1 @sm:grid-row-1 pointer-events-none")
            .dwclass!("@sm:absolute top-0 bottom-0 left-0 right-0 overflow-y-auto")
            .child_signal(top.map(Some))
        }))
    })
}
