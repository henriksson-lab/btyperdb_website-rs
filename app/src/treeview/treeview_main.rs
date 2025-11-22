
use wasm_bindgen::JsCast;
use web_sys::window;
use web_sys::{DomRect, EventTarget, HtmlCanvasElement, WebGlRenderingContext as GL};
use yew::{html, Callback, Component, Context, Html, MouseEvent, NodeRef, WheelEvent};
use yew::Properties;

use crate::appstate::AsyncData;
use crate::core_model::MsgCore;
use crate::treeview::Camera2D;
use crate::resize::ComponentSize;
use crate::treeview::treelayout::TreeLayout;



////////////////////////////////////////////////////////////
/// RGB color, 0...1
//type Color3f = (f32,f32,f32);

////////////////////////////////////////////////////////////
/// Vectors, 3d and 4d
type Vec3 = (f32,f32,f32);





////////////////////////////////////////////////////////////
/// Message sent to the event system for updating the page
#[derive(Debug)]
pub enum MsgReduction {
    MouseMove(f32,f32, bool),
    MouseClick,
    MouseWheel(f32),
    Propagate(MsgCore),
}


////////////////////////////////////////////////////////////
/// Properties for ReductionView
#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_propagate: Callback<MsgCore>,
    pub last_component_size: ComponentSize,
    pub treedata: AsyncData<TreeLayout>,
}


////////////////////////////////////////////////////////////
/// random note: Wrap gl in Rc (Arc for multi-threaded) so it can be injected into the render-loop closure.
pub struct ReductionView {
    node_ref: NodeRef,
    last_pos: (f32,f32),
    camera: Camera2D,
    last_treedata: AsyncData<TreeLayout>,
}

impl Component for ReductionView {
    type Message = MsgReduction;
    type Properties = Props;

    ////////////////////////////////////////////////////////////
    /// Create this component
    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            node_ref: NodeRef::default(),
            last_pos: (0.0,0.0),
            camera: Camera2D::new(),
            last_treedata: AsyncData::NotLoaded
        }
    }


    ////////////////////////////////////////////////////////////
    /// Handle an update message
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {            


            ////////////////////////////////////////////////////////////
            // Message: Propagate message to component above
            MsgReduction::Propagate(msg) => {
                ctx.props().on_propagate.emit(msg);
                false
            },

            ////////////////////////////////////////////////////////////
            // Message: Mouse has moved
            MsgReduction::MouseMove(x,y, press_left) => {
                let last_pos = self.last_pos;
                self.last_pos = (x,y);
                //  log::debug!(".. {:?}", last_pos);

                //Handle panning
                if press_left {
                    let dx = x - last_pos.0;
                    let dy = y - last_pos.1;
                    //log::debug!("dd {:?}", (dx,dy));
                    self.camera.x -= (dx as f32) / self.camera.zoom_x;
                    self.camera.y -= (dy as f32) / self.camera.zoom_y;
                    return true;
                }

                false
            },

            ////////////////////////////////////////////////////////////
            // Message: Mouse wheel rotated
            MsgReduction::MouseWheel(dy) => {
                let (cx,cy) = self.last_pos;
                let (wx, wy) = self.camera.cam2world(cx, cy);
                let scale = (10.0f32).powf(dy / 100.0);
                self.camera.zoom_around(wx,wy, scale);
                true
            },

            ////////////////////////////////////////////////////////////
            // Message: Mouse has clicked
            MsgReduction::MouseClick => {
                false
            },

        }
    }




    ////////////////////////////////////////////////////////////
    /// Render this component
    fn view(&self, ctx: &Context<Self>) -> Html {

        log::debug!("render reduction main");

        let cb_mousemoved = ctx.link().callback(move |e: MouseEvent | { 
            e.prevent_default();
            let (x_cam, y_cam) = mouseevent_get_cx(&e);
            let press_left = e.buttons() & 1 > 0;

            MsgReduction::MouseMove(x_cam,y_cam, press_left)
            //there is mouse movement! https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/movementX 
        });
        
        let cb_mousewheel = ctx.link().callback(move |e: WheelEvent | { 
            e.prevent_default();
            MsgReduction::MouseWheel(e.delta_y() as f32)
        });

        let cb_mouseclicked = ctx.link().callback(move |_e: MouseEvent | { 
            MsgReduction::MouseClick
        });
        

        //Compute current canvas size. Not automatic via CSS
        let window = window().expect("no window");//.document().expect("no document on window");
        let _window_h = window.inner_height().expect("failed to get height").as_f64().unwrap();
        let window_w = window.inner_width().expect("failed to get width").as_f64().unwrap();
        let canvas_w = (window_w*0.99) as usize;
        let canvas_h = 500 as usize; //(window_h*0.59) as usize;


        //Compose the view
        html! {
            <div style="display: flex; height: 500px; position: relative;">

                <div style="position: absolute; left:0; top:0; display: flex; ">
                    <canvas 
                        ref={self.node_ref.clone()} 
                        style="border:1px solid #000000;"
                        onmousemove={cb_mousemoved} 
                        onclick={cb_mouseclicked} 
                        onwheel={cb_mousewheel} 
                        width={format!{"{}", canvas_w}}
                        height={format!{"{}", canvas_h}}
                    />
                </div>

            </div>
        }
    }





    ////////////////////////////////////////////////////////////
    /// Called after DOM has been created
    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {

        let async_treedata = &ctx.props().treedata;

        if let AsyncData::Loaded(treedata) = &async_treedata {

            //Fit camera whenever we get a new umap to show
            if &self.last_treedata != async_treedata {
//                log::debug!(" fit_reduction ");
                self.camera.fit_reduction(&treedata.get_bounding_rect());
//                self.camera.zoom_x *= 2.0;
                self.last_treedata = async_treedata.clone();
            }


            log::debug!("camera {:?}", self.camera);

            // Only start the render loop if it's the first render
            // There's no loop cancellation taking place, so if multiple renders happen,
            // there would be multiple loops running. That doesn't *really* matter here because
            // there's no props update and no SSR is taking place, but it is something to keep in
            // consideration

            // TODO should we only render if data changed?
            /*
            if !first_render {
                return;
            }
            */
            

            // Once rendered, store references for the canvas and GL context. These can be used for
            // resizing the rendering area when the window or canvas element are resized, as well as
            // for making GL calls.
            let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();

            let gl: GL = canvas
                .get_context("webgl")
                .unwrap()
                .unwrap()
                .dyn_into()
                .unwrap();

            let vert_code = String::from(include_str!("./umap.vert"));
            let frag_code = include_str!("./umap.frag");

            //Get position data
            let num_lines = treedata.gl_num_lines;
            let num_points = (num_lines*2) as usize;
            log::debug!("num_lines {}", num_lines);
            
            let mut vec_vertex:Vec<f32> = Vec::new();
            let vec_vertex_size = 6; //Size of vec3+vec3    /// overkill!!
            vec_vertex.reserve(num_points * vec_vertex_size);  

            //If we offset all colors to separate part of new array, we can do a memcpy instead
            for i in 0..num_points {
                let input_base = i*2;

                vec_vertex.push(*treedata.vec_vertex.get(input_base+0).unwrap());
                vec_vertex.push(*treedata.vec_vertex.get(input_base+1).unwrap());
                vec_vertex.push(0.0); // only used for 3d reductions

                vec_vertex.push(0.0); ///////////////////////////////////////////////// color index. remove, put in separate buffer
                vec_vertex.push(0.0); ///////////////////////////////////////////////// color index. remove, put in separate buffer    filler for now
                vec_vertex.push(0.0); ///////////////////////////////////////////////// color index. remove, put in separate buffer
            }

            //Connect vertex array to GL
            let vertex_buffer = gl.create_buffer().unwrap();
            let js_vertex = js_sys::Float32Array::from(vec_vertex.as_slice());
            //let verts = js_sys::Int32Array::from(vertices_int.as_slice());
            gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vertex_buffer));
            gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &js_vertex, GL::STATIC_DRAW);

            //Compile vertex shader
            let vert_shader = gl.create_shader(GL::VERTEX_SHADER).unwrap();
            gl.shader_source(&vert_shader, vert_code.as_str());
            gl.compile_shader(&vert_shader);

            
            /*let msg= gl.get_shader_info_log(&vert_shader);
            if let Some(msg)=msg {
                log::debug!("error {}", msg);
            }*/

            //Compile fragment shader
            let frag_shader = gl.create_shader(GL::FRAGMENT_SHADER).unwrap();
            gl.shader_source(&frag_shader, frag_code);
            gl.compile_shader(&frag_shader);

            //Attach shaders
            let shader_program = gl.create_program().unwrap();
            gl.attach_shader(&shader_program, &vert_shader);
            gl.attach_shader(&shader_program, &frag_shader);
            gl.link_program(&shader_program);
            gl.use_program(Some(&shader_program));

            //Size of a float in bytes
            let sizeof_float = 4;

            //Attach the position vector as an attribute for the GL context.
            let a_position = gl.get_attrib_location(&shader_program, "a_position") as u32;
            //log::debug!("a_position {}",a_position);
            gl.enable_vertex_attrib_array(a_position);
            gl.vertex_attrib_pointer_with_i32(
                a_position, 
                3, 
                GL::FLOAT, 
                false, 
                sizeof_float*6, 
                0
            );

            //Attach color vector as an attribute
            let a_color = gl.get_attrib_location(&shader_program, "a_color") as u32;
            //log::debug!("a_color {}",a_color);
            gl.enable_vertex_attrib_array(a_color);
            gl.vertex_attrib_pointer_with_i32(
                a_color, 
                3,
                GL::FLOAT, 
                false, 
                sizeof_float * 6, 
                sizeof_float * 3
            );

            //Attach camera attributes
            let u_camera_x = gl.get_uniform_location(&shader_program, "u_camera_x");
            let u_camera_y = gl.get_uniform_location(&shader_program, "u_camera_y");
            let u_camera_zoom_x = gl.get_uniform_location(&shader_program, "u_camera_zoom_x");
            let u_camera_zoom_y = gl.get_uniform_location(&shader_program, "u_camera_zoom_y");
            gl.uniform1f(u_camera_x.as_ref(), self.camera.x as f32);
            gl.uniform1f(u_camera_y.as_ref(), self.camera.y as f32);
            gl.uniform1f(u_camera_zoom_x.as_ref(), self.camera.zoom_x as f32);
            gl.uniform1f(u_camera_zoom_y.as_ref(), self.camera.zoom_y as f32);

            //log::debug!("canvas {} {}   {:?}", canvas.width(), canvas.height(), self.camera);

            let u_display_w = gl.get_uniform_location(&shader_program, "u_display_w");
            let u_display_h = gl.get_uniform_location(&shader_program, "u_display_h");
            gl.uniform1f(u_display_w.as_ref(), canvas.width() as f32);
            gl.uniform1f(u_display_h.as_ref(), canvas.height() as f32);

            // clear canvas
            gl.clear_color(1.0, 1.0, 1.0, 1.0);
            gl.clear(GL::COLOR_BUFFER_BIT);
            
            // to make round points, need to draw square https://stackoverflow.com/questions/7237086/opengl-es-2-0-equivalent-for-es-1-0-circles-using-gl-point-smooth
            gl.draw_arrays(GL::LINES, 0, num_points as i32);
        } 
    }
}





////////////////////////////////////////////////////////////
/// Convert from vector to HTML color code
pub fn rgbvec2string(c: Vec3) -> String {
    let red=(c.0*255.0) as u8;
    let green=(c.1*255.0) as u8;
    let blue=(c.2*255.0) as u8;
    format!("#{:02X}{:02X}{:02X}", red, green, blue)
}



////////////////////////////////////////////////////////////
/// Get current camera position from a mouse event
fn mouseevent_get_cx(e: &MouseEvent) -> (f32,f32) {
    let target: Option<EventTarget> = e.target();
    let canvas: HtmlCanvasElement = target.and_then(|t| t.dyn_into::<HtmlCanvasElement>().ok()).expect("wrong type");

    let rect:DomRect = canvas.get_bounding_client_rect();
    let x = e.client_x() - (rect.left() as i32);
    let y = e.client_y() - (rect.top() as i32);

    let w = rect.width() as f32;
    let h = rect.height() as f32;

    let x_cam = (x as f32 - w/2.0)/(w/2.0);
    let y_cam = (y as f32 - h/2.0)/(h/2.0);

//    log::debug!("getcx  {} {}", x_cam, y_cam);

    (x_cam, y_cam)
}



////////////////////////////////////////////////////////////
/// Read color RGB vector from html string to 0..255
pub fn parse_rgb_i64(s: &String) -> (i64, i64, i64) {

    let s = s.as_str();
    let s_r = s.get(1..3).expect("Could not get R");
    let s_g = s.get(3..5).expect("Could not get G");
    let s_b = s.get(5..7).expect("Could not get B");
    //log::debug!("got r: {} {} {}",s_r, s_g, s_b);

    let r = i64::from_str_radix(s_r, 16).expect("parse error");
    let g = i64::from_str_radix(s_g, 16).expect("parse error");
    let b = i64::from_str_radix(s_b, 16).expect("parse error");

    (r,g,b)
}


////////////////////////////////////////////////////////////
/// Read color RGB vector from html string to 0..1
pub fn parse_rgb_f64(s: &String) -> (f32, f32, f32) {
    let (r,g,b) = parse_rgb_i64(s);
    (
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
    )
}
