use dominator::{events, DomBuilder};
use dwind::prelude::*;
use futures_signals::signal::{Signal};
use web_sys::{Element, HtmlElement};

pub fn panel_mixin<T: AsRef<Element>>(b: DomBuilder<T>) -> DomBuilder<T> {
    let b = dwclass!(
        b,
        "border border-woodsmoke-800 hover:border-woodsmoke-500 rounded"
    );
    dwclass!(b, "transition-all bg-woodsmoke-900")
}

pub fn widget_panel_mixin(
    label: impl Signal<Item = String> + 'static,
    on_close: Option<Box<dyn Fn() -> () + 'static>>
) -> impl FnOnce(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement> {
    move |b| {
        panel_mixin(b.apply(move |b| {
            b.child(html!("div", {
                .dwclass!("flex-1 h-10 font-extrabold text-l border-b border-woodsmoke-600 flex flex-row align-items-center justify-between")
                .style("margin-bottom", "4px")
                .child(html!("div", {
                    .text_signal(label)
                }))
                .apply_if(on_close.is_some(), move |b| {
                    b.child(html!("div", {
                        .dwclass!("cursor-pointer hover:text-woodsmoke-400")
                        .text("✖")
                        .event(move |_: events::Click| {
                            on_close.as_ref().unwrap()();
                        })
                    }))
                })
            }))
        }))
    }
}
