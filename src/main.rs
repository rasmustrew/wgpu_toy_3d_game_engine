#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]
#![allow(clippy::cast_precision_loss)]
use std::{rc::Rc};

use cgmath::{Deg, InnerSpace, Quaternion, Rotation3, Zero};
use ecs::{World, Light};
use winit::{
    event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    window::Window,
};
use wgpu::{util::DeviceExt};

mod util;
mod texture;
mod camera;
mod uniforms;
mod model;
mod transform;
mod ecs;
mod renderer;
use model::{Model};
use crate::camera::Camera;
use crate::transform::Transform;
use crate::ecs::Component;

const NUM_INSTANCES_PER_ROW: u16 = 10;
const SPACE_BETWEEN: f32 = 3.0;

struct State {
    mouse_pressed: bool,
    camera: Camera,
    camera_controller: camera::Controller,
    _models: Vec<Rc<Model>>,
    world: World,
    renderer: renderer::Renderer,
}

impl State {
    // Creating some of the wgpu types requires async code
    #[allow(clippy::too_many_lines)]
    async fn new(window: &Window) -> Self {
        let mut world = World::new();
        let camera = camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let camera_controller = camera::Controller::new(4.0, 0.4);
        let renderer = renderer::Renderer::new(window, &camera).await;

        let light_transform = Transform {
            position: cgmath::Vector3 { x: 2.0, y: 2.0, z: 2.0 },
            rotation: cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0),
        };
        let light_transform_buffer = renderer.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light Transform Buffer"),
                contents: bytemuck::cast_slice(&[light_transform.to_raw()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let light = Light {
            color: [1.0, 1.0, 1.0],
        };
        let light_buffer = renderer.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light VB"),
                contents: bytemuck::cast_slice(&[light]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let light_bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &renderer.light_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: light_transform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });
        let light_transform = Component::Transform(light_transform, light_transform_buffer);
        let light = Component::Light(light, light_buffer, light_bind_group);
        world.create_entity(vec![light_transform, light]);



        let res_dir = std::path::Path::new(env!("OUT_DIR")).join("resources");
        let cube_model = model::Model::load(
            &renderer.device,
            &renderer.queue,
            &renderer.texture_bind_group_layout,
            res_dir.join("cube.obj"),
        ).unwrap(); 

        let floor_model = model::Model::load(
            &renderer.device,
            &renderer.queue,
            &renderer.texture_bind_group_layout,
            res_dir.join("floor.obj"),
        ).unwrap();
        
        // Floor
        let floor_model = Rc::new(floor_model);
        let component_model = Component::Model(floor_model.clone());
        let transform = Transform {
            position: cgmath::Vector3 { x: 0.0, y: 0.0, z: 0.0 }, 
            rotation: cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0),
        };
        let instance_data = transform.to_raw();
        let instance_buffer = renderer.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&[instance_data]),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );
        let component_instances = Component::Transform(transform, instance_buffer);
        world.create_entity(vec![component_model, component_instances]); 


        // Cubes
        let cube_model = Rc::new(cube_model);
        (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let x = SPACE_BETWEEN * (f32::from(x) - f32::from(NUM_INSTANCES_PER_ROW) / 2.0);
                let z = SPACE_BETWEEN * (f32::from(z) - f32::from(NUM_INSTANCES_PER_ROW) / 2.0);

                let position = cgmath::Vector3 { x, y: 0.0, z };

                let rotation = if position.is_zero() {
                    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                };

                Transform {
                    position, rotation,
                }
            })
        }).for_each(|instance: Transform| {
            let component_model = Component::Model(cube_model.clone());
            let instance_data = instance.to_raw();
            let instance_buffer = renderer.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&[instance_data]),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                }
            );
            let component_instances = Component::Transform(instance, instance_buffer);
            world.create_entity(vec![component_model, component_instances]);
        });

        Self {
            mouse_pressed: false,
            _models: vec![cube_model, floor_model],
            world,
            camera,
            camera_controller,
            renderer,
        }
    }

    // Handle events, return true if want to capture that event so it does not get handled further
    fn input(&mut self, event: &DeviceEvent) -> bool {
        match event {
            DeviceEvent::Key(
                KeyboardInput {
                    virtual_keycode: Some(key),
                    state,
                    ..
                }
            ) => self.camera_controller.process_keyboard(*key, *state),
            DeviceEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            DeviceEvent::Button {
                button: 1, // Left Mouse Button
                state,
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            DeviceEvent::MouseMotion { delta } => {
                if self.mouse_pressed {
                    self.camera_controller.process_mouse(delta.0, delta.1);
                }
                true
            }
            _ => false,
        }
    }

    fn update(&mut self, dt: std::time::Duration) {
        // Camera
        self.camera_controller.update_camera(&mut self.camera, dt);
        // Models
        let rotate_by = Quaternion::from_angle_z(Deg(1.0));
        self.world.do_on_components(&mut |component| {
            match component {
                Component::Model(_) | Component::Light(_, _, _) => (),
                Component::Transform(instance, _) => {
                    instance.rotation = rotate_by * instance.rotation;
                },
            }
        });
        self.renderer.update(&self.camera, &self.world);
    }
}




fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = pollster::block_on(State::new(&window));
    let mut last_render_time = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::DeviceEvent {
            ref event,
            .. // We're not using device_id currently
        } => {
            state.input(event);
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    state.renderer.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.renderer.resize(**new_inner_size);
                }
                _ => {}
            }
        }
        Event::RedrawRequested(_) => {
            let now = std::time::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                state.update(dt);
            match state.renderer.render(&state.world) {
                Ok(_) => {}
                // Recreate the swap_chain if lost
                Err(wgpu::SurfaceError::Lost) => state.renderer.resize(state.renderer.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
        _ => {}
    });
}
