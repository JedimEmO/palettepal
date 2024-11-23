use dominator::{events, Dom, DomBuilder};
use dwind::prelude::*;
use web_sys::SvgElement;

pub enum Icons {
    Copy,
    Paste,
    Download,
    Edit,
}

pub fn svg_button(
    icon: Icons,
    help: &str,
    mut on_click: impl FnMut(events::Click) -> () + 'static,
    apply: impl FnOnce(DomBuilder<SvgElement>) -> DomBuilder<SvgElement>,
) -> Dom {
    let apply = move |b: DomBuilder<SvgElement>| {
        let b = dwclass!(
            b,
            "fill-woodsmoke-100 hover:fill-picton-blue-500 w-8 h-8 cursor-pointer"
        );
        b.event(move |event: events::Click| {
            on_click(event);
        })
        .apply(apply)
    };

    let icon = match icon {
        Icons::Copy => copy_icon(apply),
        Icons::Paste => paste_icon(apply),
        Icons::Download => download_icon(apply),
        Icons::Edit => edit_icon(apply),
    };

    html!("div", {
        .attr("title", help)
        .child(icon)
    })
}

// icons from https://www.svgrepo.com/collection/neuicons-oval-line-icons

pub fn copy_icon(apply: impl FnOnce(DomBuilder<SvgElement>) -> DomBuilder<SvgElement>) -> Dom {
    svg!("svg", {
        .attr("viewBox", "0 0 24 24")
        .apply(apply)
        .child(svg!("path", {
            .attr("d", "M21,8H9A1,1,0,0,0,8,9V21a1,1,0,0,0,1,1H21a1,1,0,0,0,1-1V9A1,1,0,0,0,21,8ZM20,20H10V10H20ZM6,15a1,1,0,0,1-1,1H3a1,1,0,0,1-1-1V3A1,1,0,0,1,3,2H15a1,1,0,0,1,1,1V5a1,1,0,0,1-2,0V4H4V14H5A1,1,0,0,1,6,15Z")
        }))
    })
}

pub fn paste_icon(apply: impl FnOnce(DomBuilder<SvgElement>) -> DomBuilder<SvgElement>) -> Dom {
    svg!("svg", {
        .attr("viewBox", "0 0 24 24")
        .apply(apply)
        .child(svg!("path", {
            .attr("d", "M8,21a1,1,0,0,1-1,1H3a1,1,0,0,1-1-1V6A1,1,0,0,1,3,5H15a1,1,0,0,1,0,2H4V20H7A1,1,0,0,1,8,21ZM12,4a1,1,0,0,0,0-2H6A1,1,0,0,0,6,4Zm10,6V21a1,1,0,0,1-1,1H12a1,1,0,0,1-1-1V10a1,1,0,0,1,1-1h9A1,1,0,0,1,22,10Zm-2,1H13v9h7Zm-5,3.5h3a1,1,0,0,0,0-2H15a1,1,0,0,0,0,2Zm0,4h3a1,1,0,0,0,0-2H15a1,1,0,0,0,0,2Z")
        }))
    })
}

pub fn download_icon(apply: impl FnOnce(DomBuilder<SvgElement>) -> DomBuilder<SvgElement>) -> Dom {
    svg!("svg", {
        .attr("viewBox", "0 0 24 24")
        .apply(apply)
        .child(svg!("path", {
            .attr("d", "M4,20H20a1,1,0,0,1,0,2H4a1,1,0,0,1,0-2ZM12,2a1,1,0,0,0-1,1V14.586L8.707,12.293a1,1,0,1,0-1.414,1.414l4,4a1,1,0,0,0,.325.216.986.986,0,0,0,.764,0,1,1,0,0,0,.325-.216l4-4a1,1,0,0,0-1.414-1.414L13,14.586V3A1,1,0,0,0,12,2Z")
        }))
    })
}

pub fn edit_icon(apply: impl FnOnce(DomBuilder<SvgElement>) -> DomBuilder<SvgElement>) -> Dom {
    svg!("svg", {
        .attr("viewBox", "0 0 24 24")
        .apply(apply)
        .child(svg!("path", {
            .attr("d", "M21.707,4.475,19.525,2.293a1,1,0,0,0-1.414,0L9.384,11.021a.977.977,0,0,0-.241.39L8.052,14.684A1,1,0,0,0,9,16a.987.987,0,0,0,.316-.052l3.273-1.091a.977.977,0,0,0,.39-.241l8.728-8.727A1,1,0,0,0,21.707,4.475Zm-9.975,8.56-1.151.384.384-1.151,7.853-7.854.768.768ZM2,6A1,1,0,0,1,3,5h8a1,1,0,0,1,0,2H4V20H17V13a1,1,0,0,1,2,0v8a1,1,0,0,1-1,1H3a1,1,0,0,1-1-1Z")
        }))
    })
}
