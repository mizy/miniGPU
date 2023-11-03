use glam::Vec2;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

use crate::components::{
    orthographic_camera::OrthographicCamera,
    perspective_camera::{CameraTrait, PerspectiveCamera},
};

/// mouse left to rotate, right to pan

pub struct MapController {
    pub config: MapControllerConfig,
    pressed_key: Option<VirtualKeyCode>,
    mouse_left_pressed: bool,
    mouse_right_pressed: bool,
    mouse_now_pos: Vec2,
    before_pos: Vec2,
    mouse_wheel_delta: f32,
}

pub struct MapControllerConfig {
    pub rotate_speed: f32,
    pub pan_speed: f32,
}

impl Default for MapControllerConfig {
    fn default() -> Self {
        MapControllerConfig {
            rotate_speed: 0.01,
            pan_speed: 0.002,
        }
    }
}

impl MapController {
    pub fn new(config: MapControllerConfig) -> MapController {
        MapController {
            config,
            pressed_key: None,
            mouse_left_pressed: false,
            mouse_right_pressed: false,
            mouse_now_pos: Vec2::new(0., 0.),
            before_pos: Vec2::new(0., 0.),
            mouse_wheel_delta: 0.,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let new_pressed_key = match state {
                    ElementState::Pressed => Some(*keycode),
                    ElementState::Released => None,
                };
                self.pressed_key = new_pressed_key;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    winit::event::ElementState::Pressed => {
                        // now mouse position is the start position
                        if *button == winit::event::MouseButton::Left {
                            self.mouse_left_pressed = true;
                        } else if *button == winit::event::MouseButton::Right {
                            self.mouse_right_pressed = true;
                        }
                        self.before_pos = self.mouse_now_pos;
                    }
                    winit::event::ElementState::Released => {
                        if *button == winit::event::MouseButton::Left {
                            self.mouse_left_pressed = false;
                        } else if *button == winit::event::MouseButton::Right {
                            self.mouse_right_pressed = false;
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_now_pos = Vec2::new(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::LineDelta(_, y) => {
                    self.mouse_wheel_delta = *y;
                }
                winit::event::MouseScrollDelta::PixelDelta(p) => {
                    self.mouse_wheel_delta = p.y as f32;
                }
            },

            _ => {}
        }
    }

    pub fn update(&mut self, camera: &mut Box<dyn CameraTrait>) {
        let camera_type = camera.get_type();
        if camera_type == "perspective" {
            let camera = camera.as_any().downcast_mut::<PerspectiveCamera>().unwrap();
            self.update_perspective(camera);
        } else if camera_type == "orthographic" {
            // todo: there are also some bugs in orthographic camera controller
            let camera = camera
                .as_any()
                .downcast_mut::<OrthographicCamera>()
                .unwrap();
            self.update_orthographic(camera);
        }
    }

    pub fn update_orthographic(&mut self, camera: &mut OrthographicCamera) {
        if self.mouse_left_pressed {
            let dis = self.mouse_now_pos - self.before_pos;
            let camera_look_at = (camera.config.target - camera.config.position).normalize();
            let camera_right = camera_look_at.cross(-camera.config.up).normalize();
            let camera_forward = camera_look_at.cross(camera_right).normalize();
            let camera_move = camera_right * dis.x * self.config.pan_speed
                + camera_forward * dis.y * self.config.rotate_speed;
            camera.config.position += camera_move;
            camera.config.target += camera_move;
            self.before_pos = self.mouse_now_pos;
        } else if self.mouse_right_pressed {
            let dis = self.mouse_now_pos - self.before_pos;
            let camera_look_at = camera.config.target - camera.config.position;
            let camera_look_at_norm = camera_look_at.normalize();
            let radius = camera_look_at.length();
            let camera_right = camera_look_at_norm.cross(-camera.config.up).normalize();
            let radius_vec = camera_right * dis.x * self.config.rotate_speed - camera_look_at_norm;
            let camera_up =
                radius_vec.cross(camera_right).normalize() * dis.y * self.config.rotate_speed;
            camera.config.position =
                radius_vec.normalize() * radius + camera.config.target + camera_up;
            self.before_pos = self.mouse_now_pos;
        } else if self.mouse_wheel_delta != 0. {
            let camera_look_at = (camera.config.target - camera.config.position).normalize();
            let camera_move = camera_look_at * self.mouse_wheel_delta * self.config.pan_speed;
            camera.config.position += camera_move;
            self.mouse_wheel_delta = 0.;
        }
    }

    pub fn update_perspective(&mut self, camera: &mut PerspectiveCamera) {
        if self.mouse_left_pressed {
            let dis = self.mouse_now_pos - self.before_pos;
            let camera_look_at = (camera.config.target - camera.config.position).normalize();
            let camera_right = camera_look_at.cross(-camera.config.up).normalize();
            let camera_forward = camera_look_at.cross(camera_right).normalize();
            let camera_move = camera_right * dis.x * self.config.pan_speed
                + camera_forward * dis.y * self.config.rotate_speed;
            camera.config.position += camera_move;
            camera.config.target += camera_move;
            self.before_pos = self.mouse_now_pos;
        } else if self.mouse_right_pressed {
            let dis = self.mouse_now_pos - self.before_pos;
            let camera_look_at = camera.config.target - camera.config.position;
            let camera_look_at_norm = camera_look_at.normalize();
            let radius = camera_look_at.length();
            let camera_right = camera_look_at_norm.cross(-camera.config.up).normalize();
            let radius_vec = camera_right * dis.x * self.config.rotate_speed - camera_look_at_norm;
            let camera_up =
                radius_vec.cross(camera_right).normalize() * dis.y * self.config.rotate_speed;
            camera.config.position =
                radius_vec.normalize() * radius + camera.config.target + camera_up;
            self.before_pos = self.mouse_now_pos;
        } else if self.mouse_wheel_delta != 0. {
            let camera_look_at = (camera.config.target - camera.config.position).normalize();
            let camera_move = camera_look_at * self.mouse_wheel_delta * self.config.pan_speed;
            camera.config.position += camera_move;
            self.mouse_wheel_delta = 0.;
        }
    }
}

impl Default for MapController {
    fn default() -> Self {
        MapController::new(MapControllerConfig::default())
    }
}
