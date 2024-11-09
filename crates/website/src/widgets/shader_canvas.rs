use anyhow::anyhow;
use dominator::{Dom, DomBuilder};
use futures_signals::map_ref;
use futures_signals_component_macro::component;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
use futures_signals::signal::SignalExt;

#[component(render_fn = shader_canvas_render)]
struct ShaderCanvas<
    TCtor: FnOnce(WebGl2RenderingContext, DomBuilder<HtmlCanvasElement>) -> DomBuilder<HtmlCanvasElement> = fn (WebGl2RenderingContext, DomBuilder<HtmlCanvasElement>) -> DomBuilder<HtmlCanvasElement>
> {
    #[signal]
    #[default(100)]
    width: u32,
    #[signal]
    #[default(100)]
    height: u32,
    #[default(|_, b| b)]
    ctor: TCtor,
}

pub(crate) use shader_canvas;

pub fn shader_canvas_render(props: impl ShaderCanvasPropsTrait + 'static) -> Dom {
    let ShaderCanvasProps { width, height, ctor } = props.take();
    let width = width.broadcast();
    let height = height.broadcast();

    let dim_signal = map_ref! {
        let w = width.signal(),
        let h = height.signal() => {
            (*w, *h)
        }
    };

    html!("canvas" => HtmlCanvasElement, {
        .attr_signal("width", width.signal().map(|v| format!("{v}px")))
        .attr_signal("height", height.signal().map(|v| format!("{v}px")))
        .with_node!(canvas => {
            .apply(|b| {
                let context =  canvas
                    .get_context("webgl2")
                    .map_err(|_| anyhow!("failed to get context")).expect_throw("failed to get context")
                    .ok_or(anyhow!("failed to get context")).expect_throw("failed to get context")
                    .dyn_into::<WebGl2RenderingContext>().map_err(|_| {
                    anyhow!("failed to convert to webgl2 context")
                }).expect_throw("failed to get context");


                ctor(context, b)
            })
        })
    })
}