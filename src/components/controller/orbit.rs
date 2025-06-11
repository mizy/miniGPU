use crate::components::camera::{orthographic::OrthographicCamera, perspective::PerspectiveCamera};
use glam::Vec2;
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::Key,
};

/// OrbitController 用于相机轨道控制
/// 鼠标左键旋转，右键平移，滚轮缩放
pub struct OrbitController {
    pub config: OrbitControllerConfig,
    pressed_key: Option<Key>,
    mouse_left_pressed: bool,
    mouse_right_pressed: bool,
    mouse_now_pos: Vec2,
    before_pos: Vec2,
    mouse_wheel_delta: f32,
    zoom_velocity: f32,
}

pub struct OrbitControllerConfig {
    pub rotate_speed: f32,
    pub pan_speed: f32,
    pub zoom_speed: f32,
    pub width: f32,  // window width
    pub height: f32, // window height
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub scale_factor: f32,
    pub target_distance: f32,
    pub zoom_friction: f32,
    pub min_distance: f32,
    pub max_distance: f32,
}

impl Default for OrbitControllerConfig {
    fn default() -> Self {
        OrbitControllerConfig {
            rotate_speed: 0.02,
            pan_speed: 1.0,
            width: 800.0,
            height: 600.0,
            zoom_speed: 1.0,
            min_zoom: 0.05,
            max_zoom: 1000.0,
            scale_factor: 1.0,
            target_distance: 1.0,
            zoom_friction: 0.8,
            min_distance: 0.1,
            max_distance: 1000.0,
        }
    }
}

impl OrbitController {
    pub fn new(config: OrbitControllerConfig) -> OrbitController {
        OrbitController {
            config,
            pressed_key: None,
            mouse_left_pressed: false,
            mouse_right_pressed: false,
            mouse_now_pos: Vec2::new(0.0, 0.0),
            before_pos: Vec2::new(0.0, 0.0),
            mouse_wheel_delta: 0.0,
            zoom_velocity: 0.0,
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
            WindowEvent::MouseInput { state, button, .. } => match state {
                winit::event::ElementState::Pressed => {
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
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_now_pos = Vec2::new(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::LineDelta(_, y) => {
                    self.mouse_wheel_delta = *y * 0.0005;
                }
                winit::event::MouseScrollDelta::PixelDelta(p) => {
                    self.mouse_wheel_delta = p.y as f32 * 0.0005;
                }
            },
            _ => {}
        }
    }

    /// 更新透视相机
    pub fn update_perspective_camera(&mut self, camera: &mut PerspectiveCamera) {
        let mut config = camera.to_config();

        // 计算相机向量
        let camera_look_at = config.target - config.position;
        let camera_look_at_norm = camera_look_at.normalize();
        let camera_up = glam::Vec3::Y; // 使用世界坐标系的上方向
        let camera_right = camera_look_at_norm.cross(camera_up).normalize();
        let camera_up_corrected = camera_right.cross(camera_look_at_norm).normalize();

        // 处理平移
        if self.mouse_right_pressed {
            let dis = self.mouse_now_pos - self.before_pos;

            // 计算每个像素对应的世界坐标系中的距离
            let distance_to_target = config.position.distance(config.target);
            let view_height = 2.0 * (config.fov / 2.0).tan() * distance_to_target;
            let view_width = view_height * config.aspect;

            let pan_speed_x = view_width / self.config.width / self.config.scale_factor;
            let pan_speed_y = view_height / self.config.height / self.config.scale_factor;

            // 计算相机移动向量（注意Y轴翻转）
            let camera_move = (camera_right * dis.x * pan_speed_x
                - camera_up_corrected * dis.y * pan_speed_y)
                * self.config.pan_speed;

            config.position += camera_move;
            config.target += camera_move;

            self.before_pos = self.mouse_now_pos;
        }
        // 处理旋转
        else if self.mouse_left_pressed {
            config.position =
                self.rotate_camera_around_target(config.target, config.position, camera_up);
            self.before_pos = self.mouse_now_pos;
        }

        // 处理缩放
        let current_distance = config.position.distance(config.target);
        self.handle_zoom_input(current_distance);

        if let Some(zoom_factor) = self.calculate_zoom_factor() {
            let new_distance = (current_distance * zoom_factor)
                .clamp(self.config.min_distance, self.config.max_distance);

            let direction = (config.position - config.target).normalize();
            config.position = config.target + direction * new_distance;
        }

        // 更新相机
        camera.update_from_config(config);
    }

    /// 更新正交相机
    pub fn update_orthographic_camera(&mut self, camera: &mut OrthographicCamera) {
        let mut config = camera.to_config();

        // 计算相机向量
        let camera_look_at = config.target - config.position;
        let camera_look_at_norm = camera_look_at.normalize();
        let camera_up = glam::Vec3::Y;
        let camera_right = camera_look_at_norm.cross(camera_up).normalize();
        let camera_up_corrected = camera_right.cross(camera_look_at_norm).normalize();

        // 处理平移
        if self.mouse_right_pressed {
            let dis = self.mouse_now_pos - self.before_pos;

            // 正交相机的平移计算
            let view_width = config.width / config.zoom;
            let view_height = view_width / config.aspect;

            let pan_speed_x = view_width / self.config.width / self.config.scale_factor;
            let pan_speed_y = view_height / self.config.height / self.config.scale_factor;

            let camera_move = (camera_right * dis.x * pan_speed_x
                - camera_up_corrected * dis.y * pan_speed_y)
                * self.config.pan_speed;

            config.position += camera_move;
            config.target += camera_move;

            self.before_pos = self.mouse_now_pos;
        }
        // 处理旋转
        else if self.mouse_left_pressed {
            config.position =
                self.rotate_camera_around_target(config.target, config.position, camera_up);
            self.before_pos = self.mouse_now_pos;
        }

        // 处理缩放（正交相机通过调整zoom实现）
        let current_distance = config.position.distance(config.target);
        self.handle_zoom_input(current_distance);

        if let Some(zoom_factor) = self.calculate_zoom_factor() {
            let mut new_zoom = config.zoom * zoom_factor;
            new_zoom = new_zoom.clamp(self.config.min_zoom, self.config.max_zoom);
            config.zoom = new_zoom;
        }

        // 更新相机
        camera.update_from_config(config);
    }

    /// 旋转相机围绕目标点
    fn rotate_camera_around_target(
        &mut self,
        target: glam::Vec3,
        current_position: glam::Vec3,
        world_up: glam::Vec3,
    ) -> glam::Vec3 {
        let camera_look_at = target - current_position;
        let distance = camera_look_at.length();

        // 计算当前的球面坐标
        let current_dir = camera_look_at.normalize();

        // 计算鼠标移动
        let dis = self.mouse_now_pos - self.before_pos;

        // 水平旋转（绕世界Y轴）
        let horizontal_angle = dis.x * self.config.rotate_speed;
        let horizontal_rotation = glam::Quat::from_axis_angle(world_up, -horizontal_angle);

        // 垂直旋转（绕相机右向量）
        let camera_right = current_dir.cross(world_up).normalize();
        let vertical_angle = dis.y * self.config.rotate_speed;
        let vertical_rotation = glam::Quat::from_axis_angle(camera_right, -vertical_angle);

        // 组合旋转
        let combined_rotation = horizontal_rotation * vertical_rotation;
        let new_direction = combined_rotation * (-current_dir);

        // 限制垂直角度（避免翻转）
        let new_direction = self.clamp_vertical_angle(new_direction, world_up);

        target + new_direction * distance
    }

    /// 限制垂直角度，避免相机翻转
    fn clamp_vertical_angle(&self, direction: glam::Vec3, world_up: glam::Vec3) -> glam::Vec3 {
        let max_angle = 85.0_f32.to_radians(); // 最大仰角/俯角
        let dot_product = direction.dot(world_up);
        let current_angle = dot_product.acos();

        if current_angle < max_angle || current_angle > (std::f32::consts::PI - max_angle) {
            // 需要限制角度
            let target_angle = if current_angle < max_angle {
                max_angle
            } else {
                std::f32::consts::PI - max_angle
            };

            let horizontal_dir = glam::Vec3::new(direction.x, 0.0, direction.z).normalize();
            let cos_target = target_angle.cos();
            let sin_target = target_angle.sin();

            glam::Vec3::new(
                horizontal_dir.x * sin_target,
                cos_target,
                horizontal_dir.z * sin_target,
            )
            .normalize()
        } else {
            direction
        }
    }

    /// 处理缩放输入
    fn handle_zoom_input(&mut self, current_distance: f32) {
        if self.mouse_wheel_delta != 0.0 {
            self.config.target_distance = current_distance;
            self.zoom_velocity += self.mouse_wheel_delta * self.config.zoom_speed;
            self.mouse_wheel_delta = 0.0;
        }
    }

    /// 计算缩放因子并应用摩擦力
    fn calculate_zoom_factor(&mut self) -> Option<f32> {
        let zoom_velocity_epsilon = 1e-6;
        if self.zoom_velocity.abs() > zoom_velocity_epsilon {
            let zoom_factor = (1.0 - self.zoom_velocity).clamp(0.9, 1.1);
            self.zoom_velocity *= self.config.zoom_friction;

            if self.zoom_velocity.abs() < zoom_velocity_epsilon {
                self.zoom_velocity = 0.0;
            }

            Some(zoom_factor)
        } else {
            None
        }
    }

    /// 更新窗口尺寸
    pub fn update_window_size(&mut self, width: f32, height: f32) {
        self.config.width = width;
        self.config.height = height;
    }

    /// 重置控制器状态
    pub fn reset(&mut self) {
        self.mouse_left_pressed = false;
        self.mouse_right_pressed = false;
        self.mouse_wheel_delta = 0.0;
        self.zoom_velocity = 0.0;
        self.pressed_key = None;
    }
}

impl Default for OrbitController {
    fn default() -> Self {
        OrbitController::new(OrbitControllerConfig::default())
    }
}
