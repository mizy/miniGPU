use glam::Vec2;
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::Key,
};

use crate::components::{
    orthographic_camera::OrthographicCamera,
    perspective_camera::{CameraTrait, PerspectiveCamera},
};

/// mouse left to rotate, right to pan

pub struct MapController {
    pub config: MapControllerConfig,
    pressed_key: Option<Key>,
    mouse_left_pressed: bool,
    mouse_right_pressed: bool,
    mouse_now_pos: Vec2,
    before_pos: Vec2,
    mouse_wheel_delta: f32,
}

pub struct MapControllerConfig {
    pub rotate_speed: f32,
    pub pan_speed: f32,
    pub width: f32,  // window width
    pub height: f32, //// window height
}

impl Default for MapControllerConfig {
    fn default() -> Self {
        MapControllerConfig {
            rotate_speed: 0.02,
            pan_speed: 1.,
            width: 800.,
            height: 600.,
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
                event: KeyEvent {
                    state, logical_key, ..
                },
                ..
            } => {
                let new_pressed_key = match state {
                    ElementState::Pressed => Some(logical_key),
                    ElementState::Released => None,
                };
                self.pressed_key = new_pressed_key.cloned();
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
            let camera = camera
                .as_any()
                .downcast_mut::<OrthographicCamera>()
                .unwrap();
            self.update_orthographic(camera);
        }
    }

    pub fn update_orthographic(&mut self, camera: &mut OrthographicCamera) {
        if self.mouse_right_pressed {
            let dis = self.mouse_now_pos - self.before_pos;
            let camera_look_at = (camera.config.target - camera.config.position).normalize();
            let camera_right = camera_look_at.cross(-camera.config.up).normalize();
            let camera_up = camera.config.up.normalize();
            // 计算每个像素对应的世界坐标系中的距离
            let view_width = camera.config.width;
            let view_height = camera.config.width / camera.config.aspect;
            let pan_speed_x = view_width / self.config.width;
            let pan_speed_y = view_height / self.config.height;

            let camera_move = (camera_right * dis.x * pan_speed_x
                + camera_up * dis.y * pan_speed_y)
                * self.config.pan_speed;

            camera.config.position += camera_move;
            camera.config.target += camera_move;

            self.before_pos = self.mouse_now_pos;
        } else if self.mouse_left_pressed {
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
            let camera_up = camera.config.up.normalize();

            // 计算每个像素对应的世界坐标系中的距离
            let view_height = 2.0
                * (camera.config.fov.to_radians() / 2.0).tan()
                * camera.config.position.distance(camera.config.target);
            let view_width = view_height * self.config.width / self.config.height;
            let pan_speed_x = view_width / self.config.width;
            let pan_speed_y = view_height / self.config.height;

            // 计算相机移动向量
            let camera_move = (camera_right * dis.x * pan_speed_x
                + camera_up * dis.y * pan_speed_y)
                * self.config.pan_speed;

            // 更新相机位置和目标
            camera.config.position += camera_move;
            camera.config.target += camera_move;

            // 更新鼠标位置
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
