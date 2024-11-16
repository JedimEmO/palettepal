use anyhow::anyhow;
use dominator::{Dom, DomBuilder};
use futures_signals::signal::SignalExt;
use futures_signals_component_macro::component;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{HtmlCanvasElement, HtmlElement, WebGl2RenderingContext};

#[component(render_fn = shader_canvas_render)]
struct ShaderCanvas<
    TCtor: FnOnce(WebGl2RenderingContext, DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement> = fn (WebGl2RenderingContext, DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement>
> {
    #[signal]
    #[default(100)]
    canvas_width: i32,
    #[signal]
    #[default(100)]
    canvas_height: i32,
    #[default(|_, b| b)]
    ctor: TCtor,
}

pub(crate) use shader_canvas;

pub fn shader_canvas_render(props: impl ShaderCanvasPropsTrait + 'static) -> Dom {
    let ShaderCanvasProps {
        canvas_width: width,
        canvas_height: height,
        ctor,
        apply,
    } = props.take();

    let width = width.broadcast();
    let height = height.broadcast();

    html!("canvas", {
        .attr_signal("width", width.signal().map(|v| format!("{v}px")))
        .attr_signal("height", height.signal().map(|v| format!("{v}px")))
        .with_node!(canvas => {
            .apply(|b| {
                let canvas = canvas.unchecked_into::<HtmlCanvasElement>();
                let context =  canvas
                    .get_context("webgl2")
                    .map_err(|_| anyhow!("failed to get context")).expect_throw("failed to get context")
                    .ok_or(anyhow!("failed to get context")).expect_throw("failed to get context")
                    .dyn_into::<WebGl2RenderingContext>()
                    .map_err(|_| {
                        anyhow!("failed to convert to webgl2 context")
                    }).expect_throw("failed to get context");

                ctor(context, b)
            })
        })
        .apply_if(apply.is_some(), move |b| b.apply(apply.unwrap()))
    })
}
