#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]
#![allow(clippy::cast_precision_loss)]

mod texture;
mod camera;
mod model;
mod transform;
mod light;
mod renderer;


use std::sync::Arc;

use cgmath::{Deg, InnerSpace, Quaternion, Rotation3, Zero};
use legion::World;
use legion::IntoQuery;
use light::Light;
use texture::Texture;
use winit::{
    event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    window::Window,
};
use camera::Camera;
use transform::Transform;
use model::{Model};

const NUM_INSTANCES_PER_ROW: u16 = 10;
const SPACE_BETWEEN: f32 = 3.0;

struct State {
    mouse_pressed: bool,
    camera: Camera,
    camera_controller: camera::Controller,
    models: Vec<Arc<Model>>,
    world: World,
    renderer: renderer::Renderer,
}

impl State {
    async fn new(window: &Window) -> Self {
        let world = World::default();
        
        let renderer = renderer::Renderer::new(window).await;
        let camera = camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0), cgmath::Deg(45.0), 0.1, 100.0, &renderer);
        let camera_controller = camera::Controller::new(4.0, 0.4);


        let res_dir = std::path::Path::new(env!("OUT_DIR")).join("resources");
        
        let floor_model = model::Model::load(
            &renderer.device,
            &renderer.queue,
            &Texture::create_bind_group_layout(&renderer.device),
            res_dir.join("floor.obj"),
        ).unwrap();
        let floor_model = Arc::new(floor_model);

        let cube_model = model::Model::load(
            &renderer.device,
            &renderer.queue,
            &Texture::create_bind_group_layout(&renderer.device),
            res_dir.join("cube.obj"),
        ).unwrap();
        let cube_model = Arc::new(cube_model);
        

        Self {
            mouse_pressed: false,
            models: vec![floor_model, cube_model],
            world,
            camera,
            camera_controller,
            renderer,
        }
    }

    fn populate_world(&mut self) {

        let position = cgmath::Vector3 { x: 5.0, y: 5.0, z: 5.0 };
        let color = [1.0, 1.0, 1.0];
        let light = Light::new(position, color, &self.renderer); 
        let light_debug_model = self.models[1].clone();
        self.world.push((light, light_debug_model));

        let position = cgmath::Vector3 { x: 0.0, y: 0.0, z: 0.0 };
        let rotation = cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0);
        let transform = Transform::new(position, rotation, &self.renderer);
        let floor_model = self.models[0].clone();
        self.world.push((transform, floor_model));

        
        (0..NUM_INSTANCES_PER_ROW).for_each(|z| {
            (0..NUM_INSTANCES_PER_ROW).for_each(|x| {
                let x = SPACE_BETWEEN * (f32::from(x) - f32::from(NUM_INSTANCES_PER_ROW) / 2.0);
                let z = SPACE_BETWEEN * (f32::from(z) - f32::from(NUM_INSTANCES_PER_ROW) / 2.0);

                let position = cgmath::Vector3 { x, y: 0.0, z };

                let rotation = if position.is_zero() {
                    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                };

                let cube_model = self.models[1].clone();
                let transform = Transform::new(position, rotation, &self.renderer);
                self.world.push((transform, cube_model));
                
            });
        });
        dbg!("done populating world");
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

        let mut transforms = <&mut Transform>::query();
        for transform in transforms.iter_mut(&mut self.world) {
            transform.rotate_by(rotate_by);
        }

        let mut lights = <&mut Light>::query();
        for light in lights.iter_mut(&mut self.world) {
            light.position = cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0)) * light.position;
        }
        

        self.renderer.update(&self.camera, &self.world);

    }
}




fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = pollster::block_on(State::new(&window));
    state.populate_world();
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
                    state.camera.resize(*physical_size);
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
            match state.renderer.render(&state.world, &state.camera) {
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
