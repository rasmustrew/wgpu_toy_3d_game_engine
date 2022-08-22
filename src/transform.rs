use wgpu::util::DeviceExt;

use crate::{model, renderer::Renderer};

pub struct Transform {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
    pub buffer: wgpu::Buffer,
}

impl Transform {
    pub fn to_raw(&self) -> Raw {
        compute_raw(self.position, self.rotation)
    }

    pub fn new(position: cgmath::Vector3<f32>, rotation: cgmath::Quaternion<f32>, renderer: &Renderer) ->  Self {

        let raw = compute_raw(position, rotation);
        let buffer = renderer.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Transform Buffer"),
                contents: bytemuck::cast_slice(&[raw]),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        Self {
            position,
            rotation,
            buffer,
        }
    }

    pub fn rotate_by(&mut self, rotation: cgmath::Quaternion<f32>) {
        self.rotation = rotation * self.rotation;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[allow(dead_code)]
pub struct Raw {
    pub model: [[f32; 4]; 4],
    pub normal: [[f32; 3]; 3],
}

fn compute_raw(position: cgmath::Vector3<f32>, rotation: cgmath::Quaternion<f32>) -> Raw {
    let model =
        cgmath::Matrix4::from_translation(position) * cgmath::Matrix4::from(rotation);
    Raw {
        model: model.into(),
        normal: cgmath::Matrix3::from(rotation).into(),
    }
}

impl model::Vertex for Raw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                //Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                //Rotation
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}