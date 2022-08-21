


use std::sync::Arc;

use legion::{World, query, IntoQuery};
use wgpu::{util::DeviceExt, SurfaceConfiguration};
use winit::{
    window::Window,
};


use crate::{model::{Vertex, self, DrawModel, Model}, util::{create_render_pipeline}, texture::{self, Texture}, camera::{self, Camera, Projection}, transform::{self, Transform}, light::Light};

use crate::uniforms::Uniforms;


pub struct Renderer {
    surface: wgpu::Surface,
    surface_config: SurfaceConfiguration,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    render_pipeline:wgpu::RenderPipeline,
    depth_texture: texture::Texture,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    projection: Projection,
    uniforms: Uniforms,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    pub light_bind_group_layout: wgpu::BindGroupLayout,
    light_render_pipeline: wgpu::RenderPipeline,
    _debug_material: crate::model::Material,
}

impl Renderer {
    pub async fn new(window: &Window, camera: &Camera) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
        let surface = unsafe { instance.create_surface(window) };

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

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &surface_config);


        let light_bind_group_layout = Light::create_bind_group_layout(&device);
        let texture_bind_group_layout = Texture::create_bind_group_layout(&device);
        
        let depth_texture = texture::Texture::create_depth_texture(&device, &surface_config, "depth_texture");
        let projection = camera::Projection::new(surface_config.width, surface_config.height, cgmath::Deg(45.0), 0.1, 100.0);

        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj(camera, &projection);
        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let uniform_bind_group_layout = Uniforms::create_bind_group_layout(&device);
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }
            ],
            label: Some("uniform_bind_group"),
        });

        let clear_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };

        let render_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout, &light_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let render_pipeline = {
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
                &render_pipeline_layout,
                surface_config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), transform::Raw::desc()],
                vertex_shader,
                fragment_shader,
                "Render Pipeline",
            )
        };

        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout, &light_bind_group_layout],
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
                &device,
                &layout,
                surface_config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc()],
                shader_vertex,
                shader_fragment,
                "Light Pipeline"
            )
        };

        let debug_material = {
            let diffuse_bytes = include_bytes!("../resources/cobble-diffuse.png");
            let normal_bytes = include_bytes!("../resources/cobble-normal.png");
        
            let diffuse_texture = texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "resources/alt-diffuse.png", false).unwrap();
            let normal_texture = texture::Texture::from_bytes(&device, &queue, normal_bytes, "resources/alt-normal.png", true).unwrap();
            
            model::Material::new(&device, "alt-material", diffuse_texture, normal_texture, &texture_bind_group_layout)
        };

        Self {
            surface,
            surface_config,
            device,
            queue,
            size,
            clear_color,
            render_pipeline,
            depth_texture,
            uniforms,
            texture_bind_group_layout,
            camera_buffer: uniform_buffer,
            camera_bind_group: uniform_bind_group,
            light_bind_group_layout,
            light_render_pipeline,
            _debug_material: debug_material,
            projection,
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
            self.projection.resize(new_size.width, new_size.height);
        }
    }

    pub fn render(&mut self, world: &World) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        //command buffer
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut renderables = <(&Transform, &Arc<Model>)>::query();
        let mut lights = <&Light>::query();
        let light = lights.iter(world).next().unwrap();

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

            

            render_pass.set_pipeline(&self.light_render_pipeline); // NEW!
            // render_pass.draw_light_model(
            //     &self.obj_model,
            //     &self.uniform_bind_group,
            //     &self.light_bind_group,
            // );

            render_pass.set_pipeline(&self.render_pipeline);
            
            
            for (transform, model) in renderables.iter(world) {
                render_pass.set_vertex_buffer(1, transform.buffer.slice(..));
                render_pass.draw_model(model, &self.camera_bind_group,  &light.bind_group);
            }
        }

        // Finish giving commands, and submit command buffer to queue.
        let command_buffer = encoder.finish();
        self.queue.submit(std::iter::once(command_buffer));
        output.present();
        Ok(())
    }

    pub fn update(&mut self, camera: &Camera, world: &World) {
        self.uniforms.update_view_proj(camera, &self.projection);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.uniforms]));
        let mut transforms = <&Transform>::query();
        for transform in transforms.iter(world) {
            self.queue.write_buffer(&transform.buffer, 0, bytemuck::cast_slice(&[transform.to_raw()]));
        }

    }
}






