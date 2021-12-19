use std::{rc::{Rc}};

use wgpu::BindGroup;

use crate::{model::Model, transform::Transform};

pub struct World {
    entity_counter: u64,
    pub entities: Vec<Entity>,
}

pub struct Entity {
    _id: u64,
    pub components: Vec<Component>
}

pub enum Component {
    Model(Rc<crate::model::Model>),
    Transform(crate::transform::Transform, wgpu::Buffer),
    Light(Light, wgpu::Buffer, wgpu::BindGroup)
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Light {
    pub color: [f32; 3],
}


impl World {
    pub fn new() -> Self {
        Self {
            entity_counter: 0,
            entities: vec![],
        }
    }

    pub fn create_entity (&mut self, components: Vec<Component>) {
        self.entity_counter += 1;
        let entity = Entity {
            _id: self.entity_counter,
            components,
        };
        self.entities.push(entity);
    }

    pub fn do_on_components<F>(&mut self, f: &mut F) where

    F: FnMut(&mut Component) -> () {
        self.entities.iter_mut().for_each(|entity|{
            entity.components.iter_mut().for_each(|component| {
                f(component)
            }); 
        });
    }

    pub fn do_with_components<F>(&self, f: F) where
    F: Fn(&Component) -> () {
        self.entities.iter().for_each(|entity|{
            entity.components.iter().for_each(|component| {
                f(component)
            }); 
        });
    }

    pub fn find_light(&self) -> (&Light, &wgpu::Buffer, &BindGroup) {
        let light = self.entities.iter().find_map(|entity| -> Option<(&Light, &wgpu::Buffer, &BindGroup)> {
            let light = entity.components.iter().find_map(|component| -> Option<(&Light, &wgpu::Buffer, &BindGroup)> {
                if let Component::Light(light, buffer, bind_group) = component {
                    Some((light, buffer, bind_group))
                } else {
                    None
                }
            });
            light
        });
        light.unwrap()
    }

    pub fn find_renderables(&self) -> Vec<(&Rc<Model>, &Transform, &wgpu::Buffer)>{
        let renderables = self.entities.iter().filter_map(|entity| -> Option<(&Rc<Model>, &Transform, &wgpu::Buffer)> {
            let model = entity.components.iter().find_map(|component| {
                if let Component::Model(model) = component {
                    Some(model)
                } else {
                    None
                }
            });
                
            let instance = entity.components.iter().find_map(|component| {
                if let Component::Transform(instance, buffer) = component {
                    Some((instance, buffer))
                } else {
                    None
                }
                });
            
            if let Some((transform, buffer)) = instance {
                if let Some(model) = model {
                    return Some((model, transform, buffer))
                }
            }
            None
        }).collect::<Vec<_>>();
        renderables
    }

    
}