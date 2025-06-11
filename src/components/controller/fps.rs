use crate::components::camera::{
    orthographic::{OrthographicCamera, OrthographicCameraConfig},
    perspective::{PerspectiveCamera, PerspectiveCameraConfig},
};
use glam::{Vec2, Vec3};
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{Key, NamedKey},
};

/// FPS 控制器用于第一人称视角控制
/// WASD 移动，鼠标控制视角，空格跳跃，Shift 冲刺
pub struct FPSController {
    pub config: FPSControllerConfig,

    // 键盘输入状态
    keys_pressed: std::collections::HashSet<Key>,

    // 鼠标状态
    mouse_delta: Vec2,
    mouse_sensitivity: f32,
    mouse_locked: bool,

    // 相机旋转状态
    yaw: f32,   // 水平旋转角度（弧度）
    pitch: f32, // 垂直旋转角度（弧度）

    // 移动状态
    velocity: Vec3,
    is_on_ground: bool,

    // 时间相关
    last_update_time: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct FPSControllerConfig {
    // 移动设置
    pub move_speed: f32,
    pub sprint_speed: f32,
    pub crouch_speed: f32,
    pub jump_force: f32,
    pub gravity: f32,

    // 鼠标设置
    pub mouse_sensitivity: f32,
    pub invert_y: bool,

    // 角度限制
    pub max_pitch: f32, // 最大俯仰角（弧度）
    pub min_pitch: f32, // 最小俯仰角（弧度）

    // 物理设置
    pub friction: f32,
    pub air_friction: f32,
    pub ground_check_distance: f32,

    // 相机设置
    pub eye_height: f32,        // 眼睛高度
    pub crouch_eye_height: f32, // 蹲下时的眼睛高度
    pub head_bob_enabled: bool, // 是否启用头部摆动
    pub head_bob_intensity: f32,
    pub head_bob_frequency: f32,

    // 平滑设置
    pub position_smoothing: f32,
    pub rotation_smoothing: f32,
}

impl Default for FPSControllerConfig {
    fn default() -> Self {
        Self {
            move_speed: 5.0,
            sprint_speed: 8.0,
            crouch_speed: 2.0,
            jump_force: 6.0,
            gravity: 15.0,

            mouse_sensitivity: 0.002,
            invert_y: false,

            max_pitch: std::f32::consts::FRAC_PI_2 - 0.1, // 89度
            min_pitch: -std::f32::consts::FRAC_PI_2 + 0.1, // -89度

            friction: 8.0,
            air_friction: 2.0,
            ground_check_distance: 0.1,

            eye_height: 1.8,
            crouch_eye_height: 1.2,
            head_bob_enabled: true,
            head_bob_intensity: 0.02,
            head_bob_frequency: 2.0,

            position_smoothing: 0.1,
            rotation_smoothing: 0.05,
        }
    }
}

impl FPSController {
    pub fn new(config: FPSControllerConfig) -> Self {
        Self {
            config,
            keys_pressed: std::collections::HashSet::new(),
            mouse_delta: Vec2::ZERO,
            mouse_sensitivity: 1.0,
            mouse_locked: false,
            yaw: 0.0,
            pitch: 0.0,
            velocity: Vec3::ZERO,
            is_on_ground: true,
            last_update_time: std::time::Instant::now(),
        }
    }

    /// 处理窗口事件
    pub fn process_events(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    state, logical_key, ..
                },
                ..
            } => match state {
                ElementState::Pressed => {
                    self.keys_pressed.insert(logical_key.clone());
                }
                ElementState::Released => {
                    self.keys_pressed.remove(logical_key);
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                if self.mouse_locked {
                    // 计算鼠标移动增量
                    // 注意：这里假设你有一种方式获取鼠标的相对移动
                    // 在实际应用中，你可能需要使用 DeviceEvent::MouseMotion
                }
            }
            _ => {}
        }
    }

    /// 处理设备事件（用于获取鼠标相对移动）
    pub fn process_device_events(&mut self, event: &winit::event::DeviceEvent) {
        match event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                if self.mouse_locked {
                    self.mouse_delta.x += delta.0 as f32;
                    self.mouse_delta.y += delta.1 as f32;
                }
            }
            _ => {}
        }
    }

    /// 锁定/解锁鼠标
    pub fn set_mouse_locked(&mut self, locked: bool) {
        self.mouse_locked = locked;
        if !locked {
            self.mouse_delta = Vec2::ZERO;
        }
    }

    /// 更新透视相机
    pub fn update_perspective_camera(&mut self, camera: &mut PerspectiveCamera, delta_time: f32) {
        let mut config = camera.to_config();

        // 更新旋转
        self.update_rotation();

        // 更新移动
        self.update_movement(&mut config, delta_time);

        // 计算相机方向
        let forward = self.get_forward_vector();
        let right = self.get_right_vector();
        let up = Vec3::Y;

        // 应用头部摆动
        let bob_offset = if self.config.head_bob_enabled {
            self.calculate_head_bob(delta_time)
        } else {
            Vec3::ZERO
        };

        // 更新相机位置和目标
        let eye_height = if self.is_crouching() {
            self.config.crouch_eye_height
        } else {
            self.config.eye_height
        };

        config.position = Vec3::new(
            config.position.x,
            config.position.y + eye_height,
            config.position.z,
        ) + bob_offset;

        config.target = config.position + forward;

        // 更新相机
        camera.update_from_config(config);

        // 清除鼠标增量
        self.mouse_delta = Vec2::ZERO;
    }

    /// 更新正交相机（FPS 模式下较少使用，但为了完整性提供）
    pub fn update_orthographic_camera(&mut self, camera: &mut OrthographicCamera, delta_time: f32) {
        let mut config = camera.to_config();

        // 更新旋转
        self.update_rotation();

        // 更新移动
        self.update_movement(&mut config, delta_time);

        // 计算相机方向
        let forward = self.get_forward_vector();

        // 更新相机位置和目标
        let eye_height = if self.is_crouching() {
            self.config.crouch_eye_height
        } else {
            self.config.eye_height
        };

        config.position = Vec3::new(
            config.position.x,
            config.position.y + eye_height,
            config.position.z,
        );

        config.target = config.position + forward;

        // 更新相机
        camera.update_from_config(config);

        // 清除鼠标增量
        self.mouse_delta = Vec2::ZERO;
    }

    /// 更新旋转
    fn update_rotation(&mut self) {
        // 应用鼠标输入到旋转
        self.yaw -= self.mouse_delta.x * self.config.mouse_sensitivity;
        self.pitch -= self.mouse_delta.y
            * self.config.mouse_sensitivity
            * if self.config.invert_y { -1.0 } else { 1.0 };

        // 限制俯仰角
        self.pitch = self
            .pitch
            .clamp(self.config.min_pitch, self.config.max_pitch);

        // 规范化偏航角
        self.yaw = self.yaw % (2.0 * std::f32::consts::PI);
    }

    /// 更新移动
    fn update_movement<T>(&mut self, config: &mut T, delta_time: f32)
    where
        T: HasPosition,
    {
        let forward = self.get_forward_vector_flat(); // 水平前进方向
        let right = self.get_right_vector();

        // 获取输入方向
        let mut input_direction = Vec3::ZERO;

        if self.is_key_pressed(&Key::Character("w".into())) {
            input_direction += forward;
        }
        if self.is_key_pressed(&Key::Character("s".into())) {
            input_direction -= forward;
        }
        if self.is_key_pressed(&Key::Character("a".into())) {
            input_direction -= right;
        }
        if self.is_key_pressed(&Key::Character("d".into())) {
            input_direction += right;
        }

        // 规范化输入方向
        if input_direction.length() > 0.0 {
            input_direction = input_direction.normalize();
        }

        // 计算目标速度
        let current_speed = if self.is_sprinting() {
            self.config.sprint_speed
        } else if self.is_crouching() {
            self.config.crouch_speed
        } else {
            self.config.move_speed
        };

        let target_velocity = input_direction * current_speed;

        // 应用摩擦力
        let friction = if self.is_on_ground {
            self.config.friction
        } else {
            self.config.air_friction
        };

        // 更新水平速度
        self.velocity.x = lerp(self.velocity.x, target_velocity.x, friction * delta_time);
        self.velocity.z = lerp(self.velocity.z, target_velocity.z, friction * delta_time);

        // 处理跳跃
        if self.is_key_pressed(&Key::Named(NamedKey::Space)) && self.is_on_ground {
            self.velocity.y = self.config.jump_force;
            self.is_on_ground = false;
        }

        // 应用重力
        if !self.is_on_ground {
            self.velocity.y -= self.config.gravity * delta_time;
        }

        // 更新位置
        let old_position = config.get_position();
        let new_position = old_position + self.velocity * delta_time;
        config.set_position(new_position);

        // 简单的地面检测（你可能需要更复杂的碰撞检测）
        if new_position.y <= 0.0 {
            config.set_position(Vec3::new(new_position.x, 0.0, new_position.z));
            self.velocity.y = 0.0;
            self.is_on_ground = true;
        }
    }

    /// 计算头部摆动
    fn calculate_head_bob(&self, delta_time: f32) -> Vec3 {
        if self.velocity.length() < 0.1 {
            return Vec3::ZERO;
        }

        let time = self.last_update_time.elapsed().as_secs_f32();
        let bob_y = (time * self.config.head_bob_frequency).sin() * self.config.head_bob_intensity;
        let bob_x = (time * self.config.head_bob_frequency * 0.5).sin()
            * self.config.head_bob_intensity
            * 0.5;

        Vec3::new(bob_x, bob_y, 0.0)
    }

    /// 获取前进方向向量
    fn get_forward_vector(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
    }

    /// 获取水平前进方向向量
    fn get_forward_vector_flat(&self) -> Vec3 {
        Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin()).normalize()
    }

    /// 获取右方向向量
    fn get_right_vector(&self) -> Vec3 {
        Vec3::new(
            (self.yaw + std::f32::consts::FRAC_PI_2).cos(),
            0.0,
            (self.yaw + std::f32::consts::FRAC_PI_2).sin(),
        )
    }

    /// 检查键是否被按下
    fn is_key_pressed(&self, key: &Key) -> bool {
        self.keys_pressed.contains(key)
    }

    /// 检查是否在冲刺
    fn is_sprinting(&self) -> bool {
        self.is_key_pressed(&Key::Named(NamedKey::Shift))
    }

    /// 检查是否在蹲下
    fn is_crouching(&self) -> bool {
        self.is_key_pressed(&Key::Named(NamedKey::Control))
    }

    /// 设置相机角度
    pub fn set_rotation(&mut self, yaw: f32, pitch: f32) {
        self.yaw = yaw;
        self.pitch = pitch.clamp(self.config.min_pitch, self.config.max_pitch);
    }

    /// 获取当前角度
    pub fn get_rotation(&self) -> (f32, f32) {
        (self.yaw, self.pitch)
    }

    /// 设置位置
    pub fn set_position(&mut self, position: Vec3) {
        // 这个方法需要与相机配合使用
    }

    /// 重置控制器状态
    pub fn reset(&mut self) {
        self.keys_pressed.clear();
        self.mouse_delta = Vec2::ZERO;
        self.velocity = Vec3::ZERO;
        self.yaw = 0.0;
        self.pitch = 0.0;
        self.is_on_ground = true;
    }

    /// 更新配置
    pub fn update_config(&mut self, config: FPSControllerConfig) {
        self.config = config;
    }
}

/// 用于支持不同相机类型的 trait
trait HasPosition {
    fn get_position(&self) -> Vec3;
    fn set_position(&mut self, position: Vec3);
}

impl HasPosition for PerspectiveCameraConfig {
    fn get_position(&self) -> Vec3 {
        self.position
    }

    fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }
}

impl HasPosition for OrthographicCameraConfig {
    fn get_position(&self) -> Vec3 {
        self.position
    }

    fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }
}

/// 线性插值辅助函数
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

impl Default for FPSController {
    fn default() -> Self {
        Self::new(FPSControllerConfig::default())
    }
}
