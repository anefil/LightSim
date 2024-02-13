use std::io::Read;

use wgpu::util::DeviceExt;

// pub struct OrthographicCamera {
//     position: [f32;3],
//     pointing_to: [f32;3],
//     scale: [f32;2],
// }

// impl Default for OrthographicCamera {
//     fn default() -> Self {
//         Self {
//             pointing_to: [0.0,0.0,0.0],
//             position: [-1.,-1.,1.],
//             scale: [1.,1.]
//         }
//     }
// }

const ARRAY_LEN: u32 = 2;

pub struct Vertex {
    position: [f32;2]
}

impl From<[f32;2]> for Vertex {
    fn from(value: [f32;2]) -> Self {
        Self {
            position: value
        }
    }
}

impl Vertex {
    pub fn transmute(&self) -> Vec<u8> {
        self.position.iter().flat_map(|f| unsafe { std::mem::transmute::<f32, [u8;4]>(*f) } ).collect::<Vec<_>>()
    }

    pub fn format() -> wgpu::VertexFormat {
        wgpu::VertexFormat::Float32x2
    }

    pub fn array_stride() -> u64 {
        std::mem::size_of::<f32>() as u64 * 2
    }
}

pub struct VertexHandler {
    vertex_array: Vec<Vertex>
}

impl VertexHandler {
    pub fn new(vertex: Vec<Vertex>) -> Self {
        VertexHandler { 
            vertex_array: vertex
         }
    }

    pub fn transmute(&self) -> Vec<u8> {
        self.vertex_array.iter().flat_map(|f| f.transmute()).collect::<Vec<_>>()
    }

    pub fn topology() -> wgpu::PrimitiveTopology {
        wgpu::PrimitiveTopology::TriangleStrip
    }
}

impl Default for VertexHandler {
    fn default() -> Self {
        VertexHandler::new(vec![
            [-1.,-1.].into(),
            [1.,-1.].into(),
            [-1.,1.].into(),
            [1.,1.].into(),
        ])
    }
}



pub struct ComputeShaderLightData {
    _SIZE: (u32,u32),
    time: f32,
    time_delta: f32,
    lightspeed: f32,
    wavelength: f32,
    size_w: f32,
    size_h: f32,
    ping_pong: bool,
    key_flags: u32
}


impl ComputeShaderLightData {
    fn transmute(&self) -> Vec<u8> {
        let mut vec = vec![];
        for i in &[self.time,self.time_delta,self.lightspeed,self.wavelength,self.size_w,self.size_h] {
            unsafe { std::mem::transmute::<f32, [u8;4]>(*i).iter().for_each(|f| vec.push(f.clone())); }
        }
        unsafe {
            std::mem::transmute::<u32, [u8;4]>(self.ping_pong as u32).iter().for_each(|f| vec.push(f.clone())); 
        };
        vec
    }
}

impl Default for ComputeShaderLightData {
    fn default() -> Self {
        ComputeShaderLightData { 
            _SIZE: (0,0),
            time: -0.09 * 1E-15, // nulled
            time_delta: 0.09 * 1E-15,
            lightspeed: 3. * 1E8, 
            wavelength: 500. * 1E-9,
            size_w: 50. * 1E-9,
            size_h: 50. * 1E-9,
            ping_pong: true, // flipped
            key_flags: 0
        }
    }
}

pub struct Data {
    pub _STEP: u64,
    pub vertex: VertexHandler,
    pub compute_data: ComputeShaderLightData,
    pub compute_bind_group: Option<wgpu::BindGroup>,
    pub render_bind_group: Option<wgpu::BindGroup>,
    pub render_pipeline: Option<wgpu::RenderPipeline>,
    pub compute_pipeline: Option<wgpu::ComputePipeline>,
    pub buffer: Option<wgpu::Buffer>,
    pub cumulative_storage_buffer: Option<wgpu::Buffer>,
    pub compute_buffer: Option<wgpu::Buffer>,
    pub first_re_render_texture: Option<wgpu::Texture>,
    pub first_re_render_texture_view: Option<wgpu::TextureView>,
    pub first_re_render_texture_sampler: Option<wgpu::Sampler>,
    pub second_re_render_texture: Option<wgpu::Texture>,
    pub second_re_render_texture_view: Option<wgpu::TextureView>,
    pub second_re_render_texture_sampler: Option<wgpu::Sampler>,
    pub first_im_render_texture: Option<wgpu::Texture>,
    pub first_im_render_texture_view: Option<wgpu::TextureView>,
    pub first_im_render_texture_sampler: Option<wgpu::Sampler>,
    pub second_im_render_texture: Option<wgpu::Texture>,
    pub second_im_render_texture_view: Option<wgpu::TextureView>,
    pub second_im_render_texture_sampler: Option<wgpu::Sampler>,
    pub params_render_texture: Option<wgpu::Texture>,
    pub params_render_texture_view: Option<wgpu::TextureView>,
    pub params_render_texture_sampler: Option<wgpu::Sampler>,
}

pub fn render_function(state: &mut crate::state_machine::InnerState, data: &mut Box<dyn std::any::Any>) -> Result<(), wgpu::SurfaceError> {
    let data = data.downcast_mut::<Data>().unwrap();

    data._STEP += 1;

    data.compute_data.time += data.compute_data.time_delta;
    data.compute_data.ping_pong = !data.compute_data.ping_pong;
    state.queue.write_buffer(data.compute_buffer.as_ref().unwrap(), 0, &data.compute_data.transmute());
    // if data._STEP % 200 == 0 {
    //     // let mut result = false;
    //     {
    //     data.cumulative_storage_buffer.as_ref().unwrap().slice(..).map_async(wgpu::MapMode::Read, |f| {f.is_ok();});
    //     state.device.poll(wgpu::Maintain::Wait);
    //     let a = data.cumulative_storage_buffer.as_ref().unwrap().slice(..).get_mapped_range();
    //     let value = unsafe {
    //         std::mem::transmute::<[u8;16], [f32;4]>(a.as_ref().try_into().unwrap())
    //     };
    //     println!("{:?}", value);
    //     }
    //     data.cumulative_storage_buffer.as_ref().unwrap().unmap();
    // }
    if data.compute_data.ping_pong {
        state.queue.write_buffer(data.cumulative_storage_buffer.as_ref().unwrap(), 8, &[0,0,0,0,0,0,0,0]);
    } else {
        state.queue.write_buffer(data.cumulative_storage_buffer.as_ref().unwrap(), 0, &[0,0,0,0,0,0,0,0]);
    }
    let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Compute Encoder"),
    });
    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("Compute Pass"), timestamp_writes: None });
        compute_pass.set_pipeline(data.compute_pipeline.as_ref().unwrap());
        compute_pass.set_bind_group(0, data.compute_bind_group.as_ref().unwrap(), &[]);

        compute_pass.insert_debug_marker("Compute eee idk");
        compute_pass.dispatch_workgroups(data.compute_data._SIZE.0, data.compute_data._SIZE.1, 1);
    }
    state.queue.submit(Some(encoder.finish()));

    if data._STEP % 5 == 0 {
    let output = state.surface.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
        array_layer_count: None,
        label: Some("Output texture"),
        format: Some(state.surface_capabilities.formats[0].clone()),
        dimension: None,
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0
    });

    let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(data.render_pipeline.as_ref().unwrap());
        render_pass.set_bind_group(0, data.render_bind_group.as_ref().unwrap(),&[]);

        let buffer_slice = data.buffer.as_ref().unwrap();
        // render_pass.set_bind_group(0, data.bind_group.as_ref().unwrap(), &[]);
        render_pass.set_vertex_buffer(0, buffer_slice.slice(..));


        render_pass.draw(0..4, 0..1);
        
    }
    state.queue.submit(Some(encoder.finish()));
    output.present();
    }
    Ok(())
}

pub async fn run(ev_loop: winit::event_loop::EventLoop<()>, window: winit::window::Window, size: winit::dpi::PhysicalSize<u32>) {

    let instance_descriptor = wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        flags: wgpu::InstanceFlags::VALIDATION | wgpu::InstanceFlags::DEBUG,
        dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
    };
    let adapter_options = wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false
    };
    let device_descriptor = wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES | wgpu::Features::TEXTURE_BINDING_ARRAY | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY | wgpu::Features::MAPPABLE_PRIMARY_BUFFERS,
        required_limits: wgpu::Limits {
            max_storage_textures_per_shader_stage: 5,
            ..wgpu::Limits::downlevel_defaults()
        }
    };

    let vertex_shader;
    let fragment_shader;
    let compute_shader;
    let fragment_shader_vec;
    let vertex_shader_vec;
    let compute_shader_vec;

    #[cfg(not(target_arch="wasm32"))] {
        vertex_shader_vec = std::fs::read("shaders/vertex.wgsl").expect("No shader at the location");
        fragment_shader_vec = std::fs::read("shaders/fragment.wgsl").expect("No shader at the location");
        compute_shader_vec = std::fs::read("shaders/compute.wgsl").expect("No shader at the location");
        vertex_shader = std::str::from_utf8(&vertex_shader_vec).unwrap();
        fragment_shader = std::str::from_utf8(&fragment_shader_vec).unwrap();
        compute_shader = std::str::from_utf8(&compute_shader_vec).unwrap();
    }

    #[cfg(target_arch="wasm32")]
    {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        vertex_shader_vec = document.get_element_by_id("s_vertex").unwrap().inner_html();
        fragment_shader_vec = document.get_element_by_id("s_fragment").unwrap().inner_html();
        compute_shader_vec = document.get_element_by_id("s_compute").unwrap().inner_html();
        vertex_shader = &vertex_shader_vec[..];
        fragment_shader = &fragment_shader_vec[..];
        compute_shader = &compute_shader_vec[..];
    }

    let mut state_machine = crate::state_machine::State::new(
        size,
        window,
        instance_descriptor,
        adapter_options,
        device_descriptor,
        Box::new(crate::renderer::render_function),
        Box::new(crate::renderer::Data {
            _STEP: 0,
            vertex: crate::renderer::VertexHandler::default(),
            compute_data: ComputeShaderLightData::default(),
            // camera: crate::renderer::OrthographicCamera::default(),,
            render_pipeline: None,
            compute_pipeline: None,
            compute_bind_group: None,
            render_bind_group: None,
            buffer: None,
            compute_buffer: None,
            cumulative_storage_buffer: None,
             first_re_render_texture: None,
             first_re_render_texture_sampler: None,
             first_re_render_texture_view: None,
            second_re_render_texture: None,
            second_re_render_texture_sampler: None,
            second_re_render_texture_view: None,
             first_im_render_texture: None,
             first_im_render_texture_sampler: None,
             first_im_render_texture_view: None,
            second_im_render_texture: None,
            second_im_render_texture_sampler: None,
            second_im_render_texture_view: None,
            params_render_texture: None,
            params_render_texture_sampler: None,
            params_render_texture_view: None
        })
    ).await;
    
    let size = state_machine.inner_state.size.clone();
    let surface_capabilities = &state_machine.inner_state.surface_capabilities;
    let surface_config =  wgpu::SurfaceConfiguration {
        alpha_mode: surface_capabilities.alpha_modes[0],
        desired_maximum_frame_latency: 2,
        format: surface_capabilities.formats.iter().copied().filter(|f| f.is_srgb()).next().unwrap_or(surface_capabilities.formats[0]),
        view_formats: vec![],
        height: size.height,
        width: size.width,
        // present_mode: surface_capabilities.present_modes[0],
        present_mode: wgpu::PresentMode::Immediate,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    };

    state_machine.config(surface_config);

    let data = state_machine.data.downcast_mut::<Data>().unwrap();

    let vertex_shader_module = state_machine.inner_state.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Vertex Shader Module Descriptor"),
        source: wgpu::ShaderSource::Wgsl(vertex_shader.into()),
    });
    let fragment_shader_module = state_machine.inner_state.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Fragment Shader Module Descriptor"),
        source: wgpu::ShaderSource::Wgsl(fragment_shader.into()),
    });
    let compute_shader_module = state_machine.inner_state.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Compute Shader Module Descriptor"),
        source: wgpu::ShaderSource::Wgsl(compute_shader.into()),
    });
    
    let vertex_data = data.vertex.transmute();

    let buffer = state_machine.inner_state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Buffer for vertex"),
        usage: wgpu::BufferUsages::VERTEX,
        contents: vertex_data.as_slice()
    });

    data.compute_data._SIZE = (size.width,size.height);

    data.first_re_render_texture = Some(state_machine.inner_state.device.create_texture(
        &wgpu::TextureDescriptor {
            size: wgpu::Extent3d { 
                width: size.width, height: size.height, depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("first compute texture"),
            view_formats: &[]
        }
    ));

    data.first_re_render_texture_view = Some(data.first_re_render_texture.as_ref().unwrap().create_view(&wgpu::TextureViewDescriptor {
        ..Default::default()
    }));

    data.first_re_render_texture_sampler = Some(state_machine.inner_state.device.create_sampler(&wgpu::SamplerDescriptor {
        ..Default::default()
    }));

    data.second_re_render_texture = Some(state_machine.inner_state.device.create_texture(
        &wgpu::TextureDescriptor {
            size: wgpu::Extent3d { 
                width: size.width, height: size.height, depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("second compute texture"),
            view_formats: &[]
        }
    ));

    data.second_re_render_texture_view = Some(data.second_re_render_texture.as_ref().unwrap().create_view(&wgpu::TextureViewDescriptor {
        ..Default::default()
    }));

    data.second_re_render_texture_sampler = Some(state_machine.inner_state.device.create_sampler(&wgpu::SamplerDescriptor {
        ..Default::default()
    }));

    data.first_im_render_texture = Some(state_machine.inner_state.device.create_texture(
        &wgpu::TextureDescriptor {
            size: wgpu::Extent3d { 
                width: size.width, height: size.height, depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("first compute texture"),
            view_formats: &[]
        }
    ));

    data.first_im_render_texture_view = Some(data.first_im_render_texture.as_ref().unwrap().create_view(&wgpu::TextureViewDescriptor {
        ..Default::default()
    }));

    data.first_im_render_texture_sampler = Some(state_machine.inner_state.device.create_sampler(&wgpu::SamplerDescriptor {
        ..Default::default()
    }));

    data.second_im_render_texture = Some(state_machine.inner_state.device.create_texture(
        &wgpu::TextureDescriptor {
            size: wgpu::Extent3d { 
                width: size.width, height: size.height, depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("second compute texture"),
            view_formats: &[]
        }
    ));

    data.second_im_render_texture_view = Some(data.second_im_render_texture.as_ref().unwrap().create_view(&wgpu::TextureViewDescriptor {
        ..Default::default()
    }));

    data.second_im_render_texture_sampler = Some(state_machine.inner_state.device.create_sampler(&wgpu::SamplerDescriptor {
        ..Default::default()
    }));


    data.params_render_texture = Some(state_machine.inner_state.device.create_texture(
        &wgpu::TextureDescriptor {
            size: wgpu::Extent3d { 
                width: size.width, height: size.height, depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::STORAGE_BINDING,
            label: Some("param compute texture"),
            view_formats: &[]
        }
    ));

    data.params_render_texture_view = Some(data.params_render_texture.as_ref().unwrap().create_view(&wgpu::TextureViewDescriptor {
        ..Default::default()
    }));

    data.params_render_texture_sampler = Some(state_machine.inner_state.device.create_sampler(&wgpu::SamplerDescriptor {
        ..Default::default()
    }));
    
    let compute_buffer = state_machine.inner_state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { 
        label: Some("Compute buffer"), 
        contents: &data.compute_data.transmute(), 
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC
    });

    let cumulative_storage_buffer = state_machine.inner_state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Cumulative compute buffer"),
        contents: unsafe { &std::mem::transmute::<[f32;4],[u8;16]>([0.,0.,1.,1.]) },
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ
    });


    let compute_bind_group_layout = state_machine.inner_state.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Compute bindings layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture { 
                    access: wgpu::StorageTextureAccess::ReadWrite, 
                    format: wgpu::TextureFormat::Rgba32Float,
                    view_dimension: wgpu::TextureViewDimension::D2
                },
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture { 
                    access: wgpu::StorageTextureAccess::ReadWrite, 
                    format: wgpu::TextureFormat::Rgba32Float,
                    view_dimension: wgpu::TextureViewDimension::D2
                },
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture { 
                    access: wgpu::StorageTextureAccess::ReadWrite, 
                    format: wgpu::TextureFormat::Rgba32Float,
                    view_dimension: wgpu::TextureViewDimension::D2
                },
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture { 
                    access: wgpu::StorageTextureAccess::ReadWrite, 
                    format: wgpu::TextureFormat::Rgba32Float,
                    view_dimension: wgpu::TextureViewDimension::D2
                },
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture { 
                    access: wgpu::StorageTextureAccess::ReadWrite, 
                    format: wgpu::TextureFormat::Rgba32Float,
                    view_dimension: wgpu::TextureViewDimension::D2
                },
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer { 
                    ty: wgpu::BufferBindingType::Uniform, 
                    has_dynamic_offset: false, 
                    min_binding_size: None 
                },
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer { 
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false, 
                    min_binding_size: None 
                },
                count: None
            }
        ]
    });

    let compute_bind_group = state_machine.inner_state.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Compute bindings"),
        layout: &compute_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(data.first_re_render_texture_view.as_ref().unwrap())
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(data.second_re_render_texture_view.as_ref().unwrap())
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(data.first_im_render_texture_view.as_ref().unwrap())
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(data.second_im_render_texture_view.as_ref().unwrap())
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(data.params_render_texture_view.as_ref().unwrap())
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &compute_buffer,
                    offset: 0,
                    size: None
                })
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { 
                    buffer: &cumulative_storage_buffer, 
                    offset: 0, 
                    size: None 
                })
            }
        ]
    });
    data.compute_bind_group = Some(compute_bind_group);

    let render_bind_group_layout = state_machine.inner_state.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
        label: Some("Render bindings layout"), 
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture { 
                    sample_type: wgpu::TextureSampleType::Float { filterable: false }, 
                    view_dimension: wgpu::TextureViewDimension::D2, 
                    multisampled: false 
                },
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture { 
                    sample_type: wgpu::TextureSampleType::Float { filterable: false }, 
                    view_dimension: wgpu::TextureViewDimension::D2, 
                    multisampled: false 
                },
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture { 
                    sample_type: wgpu::TextureSampleType::Float { filterable: false }, 
                    view_dimension: wgpu::TextureViewDimension::D2, 
                    multisampled: false 
                },
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture { 
                    sample_type: wgpu::TextureSampleType::Float { filterable: false }, 
                    view_dimension: wgpu::TextureViewDimension::D2, 
                    multisampled: false 
                },
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 7,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 8,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture { 
                    sample_type: wgpu::TextureSampleType::Float { filterable: false }, 
                    view_dimension: wgpu::TextureViewDimension::D2, 
                    multisampled: false 
                },
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 9,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 10,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer { 
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false, 
                    min_binding_size: None
                },
                count: None
            }
        ]
    });

    let render_bind_group = state_machine.inner_state.device.create_bind_group(&wgpu::BindGroupDescriptor { 
        label: Some("Render bindings"),
        layout: &render_bind_group_layout, 
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(data.first_re_render_texture_view.as_ref().unwrap())
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(data.first_re_render_texture_sampler.as_ref().unwrap())
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(data.second_re_render_texture_view.as_ref().unwrap())
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(data.second_re_render_texture_sampler.as_ref().unwrap())
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(data.first_im_render_texture_view.as_ref().unwrap())
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Sampler(data.first_im_render_texture_sampler.as_ref().unwrap())
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: wgpu::BindingResource::TextureView(data.second_im_render_texture_view.as_ref().unwrap())
            },
            wgpu::BindGroupEntry {
                binding: 7,
                resource: wgpu::BindingResource::Sampler(data.second_im_render_texture_sampler.as_ref().unwrap())
            },
            wgpu::BindGroupEntry {
                binding: 8,
                resource: wgpu::BindingResource::TextureView(data.params_render_texture_view.as_ref().unwrap())
            },
            wgpu::BindGroupEntry {
                binding: 9,
                resource: wgpu::BindingResource::Sampler(data.params_render_texture_sampler.as_ref().unwrap())
            },
            wgpu::BindGroupEntry {
                binding: 10,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { 
                    buffer: &compute_buffer, 
                    offset: 0, 
                    size: None 
                })
            }
        ]
    });
    data.render_bind_group = Some(render_bind_group);

    
    let compute_pipeline_layout = state_machine.inner_state.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("(compute) Pipeline Layout descriptor"),
        push_constant_ranges: &[],
        bind_group_layouts: &[
            &compute_bind_group_layout
        ]
    });

    
    let compute_pipeline = state_machine.inner_state.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        entry_point: "main",
        module: &compute_shader_module,
        label: Some("Compute shader pipeline"),
        layout: Some(&compute_pipeline_layout)
    });
    

    let render_pipeline_layout = state_machine.inner_state.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("(render) Pipeline Layout descriptor"),
        push_constant_ranges: &[],
        bind_group_layouts: &[
            &render_bind_group_layout
        ]
    });
    
    let render_pipeline = state_machine.inner_state.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        vertex: wgpu::VertexState {
            buffers: &[wgpu::VertexBufferLayout {
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    format: Vertex::format(),
                    offset: 0,
                    shader_location: 0
                }],
                array_stride: Vertex::array_stride(),
            }],
            entry_point: "main",
            module: &vertex_shader_module,
        } ,
        fragment: Some(wgpu::FragmentState {
            targets: &[Some(wgpu::ColorTargetState {
                format: state_machine.inner_state.surface_capabilities.formats[0].clone(), //todo,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL
            })],
            entry_point: "main",
            module: &fragment_shader_module,
        }),
        depth_stencil: None,
        label: Some("Render Pipeline Descriptor"),
        layout: Some(&render_pipeline_layout),
        multisample: wgpu::MultisampleState {
            ..Default::default()
        },
        multiview: None,
        primitive: wgpu::PrimitiveState {
            unclipped_depth: false,
            conservative: false,
            cull_mode: None, // todo,
            front_face: wgpu::FrontFace::Ccw,
            polygon_mode: wgpu::PolygonMode::Fill,
            strip_index_format: None,
            topology: VertexHandler::topology()
        }
    });


    data.render_pipeline = Some(render_pipeline);
    data.compute_pipeline = Some(compute_pipeline);
    

    data.buffer = Some(buffer);
    data.compute_buffer = Some(compute_buffer);
    data.cumulative_storage_buffer = Some(cumulative_storage_buffer);
    // state_machine.data.downcast_mut::<Data>().unwrap().bind_group = Some(bind_group);

    // wgpu::ShaderStages
    
    let _result = ev_loop.run(move |event, target| {
        let _ = &state_machine;
        target.set_control_flow(winit::event_loop::ControlFlow::Poll);
        

        match event {
            winit::event::Event::WindowEvent { window_id: _, event } => {
                match event {
                    winit::event::WindowEvent::CloseRequested => {
                        target.exit();
                    },
                    winit::event::WindowEvent::Resized(v) => {
                        #[cfg(target_arch="wasm32")] {
                            unsafe {
                                web_sys::console::log(&wasm_bindgen::JsValue::from(format!("{}, {}", v.width, v.height)).into());
                            }
                        }
                        #[cfg(not(target_arch="wasm32"))] {
                            state_machine.resize(v);
                        }
                    },
                    winit::event::WindowEvent::RedrawRequested => {
                        state_machine.render();
                    }

                    _ => {}
                }
            },
            winit::event::Event::AboutToWait => {
                state_machine.inner_state.window.request_redraw();
            }

            _ => {}
        }
    });
}
