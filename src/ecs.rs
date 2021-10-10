use std::{rc::{Rc}};

pub struct Entity {
    _id: usize,
    components: Vec<Component>
}

pub enum Component {
    Model(Rc<crate::model::Model>),
    Instance(crate::instance::Instance, wgpu::Buffer)
}

static mut ENTITY_COUNTER: usize = 0;

impl Entity{
    pub fn new(components: Vec<Component>) -> Entity {
        unsafe {
            ENTITY_COUNTER += 1; 
        
            Entity {
                _id: ENTITY_COUNTER,
                components,
            }
        }
    }

    pub fn get_components(&self) -> &Vec<Component> {
        &self.components
    }

    pub fn get_components_mut(&mut self) -> &mut Vec<Component> {
        &mut self.components
    }
}