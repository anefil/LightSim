pub struct InnerState<'window> {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub surface: wgpu::Surface<'window>,
    pub surface_capabilities: wgpu::SurfaceCapabilities,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: winit::window::Window,
    pub config: wgpu::SurfaceConfiguration,
}

pub struct State<'window> {
    pub inner_state: InnerState<'window>,
    pub render_data: Option<Box<dyn Renderer>>,
}

impl<'window> State<'window> {
    pub async fn new(size: winit::dpi::PhysicalSize<u32>,window: winit::window::Window, instance_descriptor: wgpu::InstanceDescriptor, adapter_options: wgpu::RequestAdapterOptions<'window,'window>, device_descriptor: wgpu::DeviceDescriptor<'window>, render_data: Option<Box<dyn Renderer>>) -> Self {

        let instance = wgpu::Instance::new(instance_descriptor);
        
        let surface = unsafe { instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(&window).unwrap()).expect("Trouble creating surface") };
        // adapter_options.compatible_surface = Some(&surface);
        let adapter = instance.request_adapter(&adapter_options).await.expect("Adapter can not be retieved");

        let surface_capabilities = surface.get_capabilities(&adapter);
        // let (device, queue) = futures::executor::block_on(adapter.request_device(&device_descriptor, None)).expect("Could not request device");
        let (device, queue) = adapter.request_device(&device_descriptor, None).await.expect("Could not request device");

        let formats = surface_capabilities.formats[0].clone();

        State {
            inner_state: InnerState { 
                window,
                size,
                adapter,
                device,
                instance,
                queue,
                surface,
                surface_capabilities,
                config: wgpu::SurfaceConfiguration {
                    alpha_mode: wgpu::CompositeAlphaMode::Auto,
                    desired_maximum_frame_latency: 2,
                    format: formats,
                    height: 400,
                    width: 400,
                    present_mode: wgpu::PresentMode::AutoVsync,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: vec![]
                } 
            },
            render_data,
        }
    }

    pub fn config(&mut self, surface_config: wgpu::SurfaceConfiguration) {
        self.inner_state.config = surface_config.clone();
        self.inner_state.surface.configure(&self.inner_state.device, &self.inner_state.config);
    }

    pub fn resize(&mut self, desired_size: winit::dpi::PhysicalSize<u32>) {
        if desired_size.height != 0 && desired_size.width != 0 {
            self.inner_state.size = desired_size;
            self.inner_state.config.width = desired_size.width;
            self.inner_state.config.height = desired_size.height;
            self.inner_state.surface.configure(&self.inner_state.device, &self.inner_state.config);
        }
    }

    pub fn render(&mut self) {
        self.render_data.as_mut().unwrap().render(&mut self.inner_state);
    }
}

pub trait Renderer {
    fn render(&mut self, inner: &mut InnerState) -> Result<(),wgpu::SurfaceError> {
        Ok(())
    }
}