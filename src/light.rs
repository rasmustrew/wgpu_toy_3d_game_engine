use wgpu::util::DeviceExt;

use crate::{transform::Transform, renderer::Renderer};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Raw {
    pub position: [f32; 3],
    pub _padding: u32,
    pub color: [f32; 3],
    pub _padding2: u32,
}

pub struct Light {
    position: cgmath::Vector3<f32>,
    color: [f32; 3],
    buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Light {
    pub fn new(position: cgmath::Vector3<f32>, color: [f32; 3], renderer: &Renderer) -> Self {

        let raw = compute_raw(position, color);

        let buffer = renderer.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light VB"),
                contents: bytemuck::cast_slice(&[raw]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &renderer.light_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
            ],
            label: None,
        });

        Light {
            position,
            color, 
            buffer,
            bind_group
        }
    }

    pub fn to_raw(&self) -> Raw {
        compute_raw(self.position, self.color)
    }
}

fn compute_raw(position: cgmath::Vector3<f32>, color: [f32; 3]) -> Raw {
    Raw {
        position: position.into(),
        _padding: 0,
        color,
        _padding2: 0,
    }
}