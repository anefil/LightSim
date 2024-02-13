use winit;

fn main() {   
    let ev_loop;
    let window;
    let size;
    
    #[cfg(target_arch="wasm32")]
    {

        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowBuilderExtWebSys;
        let canvas = web_sys::window().unwrap().document().unwrap().get_element_by_id("canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        ev_loop = winit::event_loop::EventLoopBuilder::new().build().expect("Trouble building event loop");
        size = { 
            let size = canvas.get_bounding_client_rect();
            winit::dpi::PhysicalSize {
                width: size.width() as u32,
                height: size.height() as u32
            }
        };
        window = winit::window::WindowBuilder::new().with_canvas(Some(canvas))
        .build(&ev_loop).expect("Trouble building window handler");
        wasm_bindgen_futures::spawn_local(rs_wgpu_cube::renderer::run(ev_loop,window, size));
    }

    #[cfg(not(target_arch="wasm32"))]
    {
        ev_loop = winit::event_loop::EventLoopBuilder::new().build().expect("Trouble building event loop");
        size = winit::dpi::PhysicalSize {
                width: 900,
                height: 900
            };
        window = winit::window::WindowBuilder::new().build(&ev_loop).expect("Trouble building window handler"); 
        futures::executor::block_on(rs_wgpu_cube::renderer::run(ev_loop,window,size));
    }
}