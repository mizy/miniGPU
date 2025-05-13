use glam::Vec2;
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::Key,
    window,
};

use crate::components::{
    orthographic_camera::OrthographicCamera,
    perspective_camera::{CameraTrait, PerspectiveCamera},
};

/// mouse left to rotate, right to pan, because i want to make it like a map

pub struct MapController {
    pub config: MapControllerConfig,
    pressed_key: Option<Key>,
    mouse_left_pressed: bool,
    mouse_right_pressed: bool,
    mouse_now_pos: Vec2,
    before_pos: Vec2,
    mouse_wheel_delta: f32,
    target_distance: f32,
    zoom_velocity: f32,
    zoom_friction: f32,
    min_distance: f32,
    max_distance: f32,
}

pub struct MapControllerConfig {
    pub rotate_speed: f32,
    pub pan_speed: f32,
    pub zoom_speed: f32,
    pub width: f32,  // window width
    pub height: f32, //// window height
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub scale_factor: f32,
}

impl Default for MapControllerConfig {
    fn default() -> Self {
        MapControllerConfig {
            rotate_speed: 0.02,
            pan_speed: 1.,
            width: 800.,
            height: 600.,
            zoom_speed: 1.,
            min_zoom: 0.05,
            max_zoom: 1000.,
            scale_factor: 1.0,
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
            target_distance: 1.,
            zoom_velocity: 0.,
            zoom_friction: 0.9,
            min_distance: 0.1,
            max_distance: 1000.,
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
                    self.mouse_wheel_delta = *y / 0.0005; // 0.0005 is a magic number to make zoom start from 1
                }
                winit::event::MouseScrollDelta::PixelDelta(p) => {
                    self.mouse_wheel_delta = p.y as f32 / 0.0005;
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

    // orthographic camera use left mouse to rotate, right mouse to pan, it's like a normal CAD user's habit
    pub fn update_orthographic(&mut self, camera: &mut OrthographicCamera) {
        if self.mouse_right_pressed {
            let dis = self.mouse_now_pos - self.before_pos;
            let view_width = camera.config.width / camera.config.zoom; // 考虑缩放级别
            let view_height = view_width / camera.config.aspect;

            let camera_look_at = (camera.config.target - camera.config.position).normalize();
            let camera_right = camera_look_at.cross(-camera.config.up).normalize();
            let camera_up = camera.config.up.normalize();

            let pan_speed_x = view_width / self.config.width / self.config.scale_factor;
            let pan_speed_y = view_height / self.config.height / self.config.scale_factor;

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
        }
        if self.mouse_wheel_delta != 0. {
            // 计算缩放速度
            let zoom_speed = 0.1 * self.config.zoom_speed;

            // 累加缩放速度
            self.zoom_velocity += -self.mouse_wheel_delta * zoom_speed;

            // 重置滚轮增量
            self.mouse_wheel_delta = 0.;
        }

        // 应用缩放 - 直接修改相机宽度/高度而不是移动相机
        let zoom_velocity_epsilon = 1e-6;
        if self.zoom_velocity.abs() > zoom_velocity_epsilon {
            // 计算缩放因子
            let zoom_factor = (1.0 - self.zoom_velocity).max(0.9).min(1.1);
            let mut current_zoom = camera.config.zoom * zoom_factor;

            // 限制缩放范围
            current_zoom = current_zoom.clamp(self.config.min_zoom, self.config.max_zoom);

            camera.config.zoom = current_zoom;

            self.zoom_velocity *= self.zoom_friction;

            if self.zoom_velocity.abs() < zoom_velocity_epsilon {
                self.zoom_velocity = 0.0;
            }
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
            let pan_speed_x = view_width / self.config.width / self.config.scale_factor;
            let pan_speed_y = view_height / self.config.height / self.config.scale_factor;

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
        }
        if self.mouse_wheel_delta != 0. {
            // 获取当前距离
            let current_distance = camera.config.position.distance(camera.config.target);
            self.target_distance = current_distance;

            // 根据距离调整缩放速度
            let zoom_speed = current_distance * self.config.zoom_speed;

            // 累加缩放速度
            self.zoom_velocity += -self.mouse_wheel_delta * zoom_speed;

            // 重置滚轮增量
            self.mouse_wheel_delta = 0.;
        }

        // 应用缩放
        let zoom_velocity_epsilon = 1e-6;
        if self.zoom_velocity.abs() > zoom_velocity_epsilon {
            let current_distance = camera.config.position.distance(camera.config.target);

            let zoom_factor: f32 = (1.0 - self.zoom_velocity).max(0.9).min(1.1);
            let new_distance =
                (current_distance * zoom_factor).clamp(self.min_distance, self.max_distance);

            let direction = (camera.config.position - camera.config.target).normalize();
            camera.config.position = camera.config.target + direction * new_distance;

            self.zoom_velocity *= self.zoom_friction;

            if self.zoom_velocity.abs() < zoom_velocity_epsilon {
                self.zoom_velocity = 0.0;
            }
        }
    }
}

impl Default for MapController {
    fn default() -> Self {
        MapController::new(MapControllerConfig::default())
    }
}
