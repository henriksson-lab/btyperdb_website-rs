use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::HtmlIFrameElement;
use yew::{html, Callback, Component, Context, Html, NodeRef, Properties};

const IFRAME_STYLE: &str = "display: block; position: absolute; top: 0; left: 0; width: 100%; height: 100%; overflow: hidden; border: 0; opacity: 0; pointer-events: none; z-index: -1;";

/// Yew component to observe changes to the size of the parent element.
///
/// Code adapted from https://docs.rs/yew-component-size/latest/yew_component_size/  updated to new version of yew
#[derive(Debug)]
pub struct ComponentSizeObserver {
    iframe_ref: NodeRef,
    on_resize: Option<Closure<dyn Fn()>>,
}

/// ComponentSizeObserver properties
#[derive(Properties, Clone, PartialEq, Debug)]
pub struct Props {
    /// A callback that is fired when the component size changes for any reason.
    pub onsize: Callback<ComponentSize>,
}

/// A struct containing the width and height of the component
#[derive(Default, Clone, PartialEq, Debug, Serialize, Deserialize)]
//#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ComponentSize {
    /// Width of the component in pixels
    pub width: f64,

    /// Height of the component in pixels
    pub height: f64,
}

impl Component for ComponentSizeObserver {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            iframe_ref: Default::default(),
            on_resize: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn changed(
        &mut self,
        ctx: &Context<Self>,
        old_props: &Self::Properties,
    ) -> bool  {
        let props = ctx.props();
        if props != old_props {
            self.add_resize_listener(ctx);
            false
        } else {
            false
        }
    }
 
    
    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <iframe style={IFRAME_STYLE} ref={self.iframe_ref.clone()} />
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.add_resize_listener(ctx);
        }
    }
}




impl ComponentSizeObserver {


    fn add_resize_listener(&mut self, ctx: &Context<Self>) {
        let iframe = self.iframe_ref.cast::<HtmlIFrameElement>().unwrap();
        let window = iframe.content_window().unwrap();

        let iframe_ref = self.iframe_ref.clone();

        let size_callback = ctx.props().onsize.clone();
        let on_resize = Closure::wrap(Box::new(move || {
            //log::debug!("resizing");
            let iframe = iframe_ref.cast::<HtmlIFrameElement>().unwrap();
            let bcr = iframe.get_bounding_client_rect();
            size_callback.emit(ComponentSize {
                width: bcr.width(),
                height: bcr.height(),
            });
        }) as Box<dyn Fn()>);
        window.set_onresize(Some(on_resize.as_ref().unchecked_ref()));
        self.on_resize = Some(on_resize);
    }
}