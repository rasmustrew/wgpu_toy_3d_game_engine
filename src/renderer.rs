use std::sync::Arc;
use legion::{World, IntoQuery};
use wgpu::{SurfaceConfiguration, util::DeviceExt};
use winit::window::Window;
use crate::{model::{Vertex, self, Draw, Model, DrawLight}, texture::{self, Texture}, camera::{self, Camera}, transform::{self, Transform}, light::{Light, self}};


pub struct Renderer {
    surface: wgpu::Surface,
    pub surface_config: SurfaceConfiguration,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    render_pipeline:wgpu::RenderPipeline,
    depth_texture: texture::Texture,
    light_render_pipeline: wgpu::RenderPipeline,
    _debug_light_model: Model,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    num_lights: usize,
}

impl Renderer {
    pub async fn new(window: &Window, _init_light: &Light) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            dx12_shader_compiler: Default::default(),
        });
        let surface = unsafe { instance.create_surface(window) }.unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps.formats.iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![]
        };
        surface.configure(&device, &surface_config);
        
        let depth_texture = texture::Texture::create_depth_texture(&device, &surface_config, "depth_texture");
        let clear_color = wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 };

        let light_bind_group_layout = light::Raw::create_bind_group_layout(&device);
        let texture_bind_group_layout = Texture::create_bind_group_layout(&device);
        let camera_bind_group_layout = camera::Raw::create_bind_group_layout(&device);
        

        let light_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                size: 1,
                label: Some("Light VB"),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light::Raw::create_bind_group_layout(&device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: light_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });
        
        let render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
            let vertex_shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader Vertex"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/wgpu_0.13/vertex_shader.wgsl").into()),
            };
            let fragment_shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader Fragment"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/wgpu_0.13/fragment_shader.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &layout,
                (surface_config.format, Some(texture::Texture::DEPTH_FORMAT)),
                &[model::ModelVertex::desc(), transform::Raw::desc()],
                vertex_shader,
                fragment_shader,
                "Render Pipeline",
            )
        };

        let light_render_pipeline = create_light_render_pipeline(&device, &camera_bind_group_layout, &light_bind_group_layout, &surface_config);


        let res_dir = std::path::Path::new(env!("OUT_DIR")).join("resources");

        let _debug_light_model = model::Model::load(
            &device,
            &queue,
            &Texture::create_bind_group_layout(&device),
            res_dir.join("cube.obj"),
        ).unwrap();


        Self {
            surface,
            surface_config,
            device,
            queue,
            size,
            clear_color,
            render_pipeline,
            depth_texture,
            light_render_pipeline,
            light_buffer,
            light_bind_group,
            num_lights: 0,
            _debug_light_model,
        }
    }

    // If the window has been resized, we need to recreate the surface with the new size. 
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
            self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.surface_config, "depth_texture");
        }
    }

    pub fn update_lights(&mut self, lights: &Vec<light::Raw>) {
        let new_buffer= self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light VB"),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(lights),
            }
        );
        let new_bind_group =  self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light::Raw::create_bind_group_layout(&self.device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: new_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });
        self.num_lights = lights.len();
        self.light_buffer = new_buffer;
        self.light_bind_group = new_bind_group;

    }

    pub fn render(&mut self, world: &World, camera: &Camera) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        //command buffer
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(self.clear_color),
                            store: true,
                        }
                    })
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            
            let num_lights = <&Light>::query().iter(world).count() as u32;
            
            render_pass.set_pipeline(&self.light_render_pipeline);
            render_pass.draw_light_model_instanced(
                &self._debug_light_model,
                0..num_lights,
                &camera.bind_group,
                &self.light_bind_group,
            );
            
            
            let mut renderables = <(&Transform, &Arc<Model>)>::query();
            render_pass.set_pipeline(&self.render_pipeline);
            for (transform, model) in renderables.iter(world) {
                render_pass.set_vertex_buffer(1, transform.buffer.slice(..));
                render_pass.draw_model(model, &camera.bind_group,  &self.light_bind_group);
            }
        }

        // Finish giving commands, and submit command buffer to queue.
        let command_buffer = encoder.finish();
        self.queue.submit(std::iter::once(command_buffer));
        output.present();
        Ok(())
    }

    pub fn update(&mut self, camera: &Camera, world: &World) {
        let camera_raw = camera.to_raw();
        self.queue.write_buffer(&camera.buffer, 0, bytemuck::cast_slice(&[camera_raw]));

        let lights_raw: Vec<light::Raw> = <&Light>::query().iter(world).map(|light| {
            light.to_raw()
        }).collect();
        if lights_raw.len() == self.num_lights {
            self.queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&lights_raw));
            
        } else {
            self.update_lights(&lights_raw);
        }
        

        let mut transforms = <&Transform>::query();
        for transform in transforms.iter(world) {
            self.queue.write_buffer(&transform.buffer, 0, bytemuck::cast_slice(&[transform.to_raw()]));
        }

    }
}

fn create_light_render_pipeline(device: &wgpu::Device, camera_bind_group_layout: &wgpu::BindGroupLayout, light_bind_group_layout: &wgpu::BindGroupLayout, surface_config: &SurfaceConfiguration) -> wgpu::RenderPipeline {
    let light_render_pipeline = {
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Light Pipeline Layout"),
            bind_group_layouts: &[camera_bind_group_layout, light_bind_group_layout],
            push_constant_ranges: &[],
        });
        let shader_vertex = wgpu::ShaderModuleDescriptor {
            label: Some("Light Shader Vertex"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/wgpu_0.13/vertex_shader_light_box.wgsl").into()),
        };
        let shader_fragment = wgpu::ShaderModuleDescriptor {
            label: Some("Light Shader Fragment"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/wgpu_0.13/fragment_shader_light_box.wgsl").into()),
        };
        create_render_pipeline(
            device,
            &layout,
            (surface_config.format, Some(texture::Texture::DEPTH_FORMAT)),
            &[model::ModelVertex::desc()],
            shader_vertex,
            shader_fragment,
            "Light Pipeline"
        )
    };
    light_render_pipeline
}


pub fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    format: (wgpu::TextureFormat, Option<wgpu::TextureFormat>),
    vertex_layouts: &[wgpu::VertexBufferLayout],
    vertex_shader: wgpu::ShaderModuleDescriptor,
    fragment_shader: wgpu::ShaderModuleDescriptor,
    label: &str,
) -> wgpu::RenderPipeline {
    let vertex_shader = device.create_shader_module(vertex_shader);
    let fragment_shader = device.create_shader_module(fragment_shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &vertex_shader,
            entry_point: "main",
            buffers: vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &fragment_shader,
            entry_point: "main",
            targets: &[Some(wgpu::ColorTargetState {
                format: format.0,
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
            // Requires Features::DEPTH_CLAMPING
            unclipped_depth: false,
        },
        depth_stencil: format.1.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}





