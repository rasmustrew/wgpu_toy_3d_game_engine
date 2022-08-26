use wgpu::{Device};


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Raw {
    pub position: [f32; 3],
    pub _padding: u32,
    pub color: [f32; 3],
    pub _padding2: u32,
}

pub struct Light {
    pub position: cgmath::Vector3<f32>,
    color: [f32; 3],
}

impl Light {
    pub fn new(position: cgmath::Vector3<f32>, color: [f32; 3]) -> Self {

        Self {
            position,
            color, 
        }
    }

    pub fn to_raw(&self) -> Raw {
        Raw::new(self.position, self.color)
    }

    

    
}

impl Raw {
    fn new(position: cgmath::Vector3<f32>, color: [f32; 3]) -> Self {
        Self {
            position: position.into(),
            _padding: 0,
            color,
            _padding2: 0,
        }
    }

    pub fn create_bind_group_layout(device: &Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: None,
        
        })
    }

}

