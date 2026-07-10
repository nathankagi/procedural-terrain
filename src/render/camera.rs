use cgmath::{self, InnerSpace, SquareMatrix, Vector3};
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_position = camera.eye.to_homogeneous().into();
        self.view_proj = (OPENGL_TO_WGPU_MATRIX * camera.build_view_projection_matrix()).into();
    }
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

pub enum CameraController {
    Static(StaticCamera),
    Orbit(OrbitCamera),
}

impl CameraController {
    pub fn handle_key(&mut self, code: KeyCode, pressed: bool) -> bool {
        match self {
            CameraController::Static(c) => c.handle_key(code, pressed),
            CameraController::Orbit(c) => c.handle_key(code, pressed),
        }
    }

    pub fn handle_mouse_button(&mut self, button: MouseButton, pressed: bool) -> bool {
        match self {
            CameraController::Static(c) => c.handle_mouse_button(button, pressed),
            CameraController::Orbit(c) => c.handle_mouse_button(button, pressed),
        }
    }

    pub fn handle_mouse_motion(&mut self, camera: &mut Camera, dx: f64, dy: f64) {
        match self {
            CameraController::Static(c) => c.handle_mouse_motion(camera, dx, dy),
            CameraController::Orbit(c) => c.handle_mouse_motion(camera, dx, dy),
        }
    }

    pub fn handle_scroll(&mut self, camera: &mut Camera, delta: f32) {
        match self {
            CameraController::Static(c) => c.handle_scroll(camera, delta),
            CameraController::Orbit(c) => c.handle_scroll(camera, delta),
        }
    }

    pub fn update(&self, camera: &mut Camera, dt: f32) {
        match self {
            CameraController::Static(c) => c.update(camera, dt),
            CameraController::Orbit(c) => c.update(camera, dt),
        }
    }
}

pub struct StaticCamera;

impl StaticCamera {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_key(&mut self, _code: KeyCode, _pressed: bool) -> bool {
        false
    }

    pub fn handle_mouse_button(&mut self, _button: MouseButton, _pressed: bool) -> bool {
        false
    }

    pub fn handle_mouse_motion(&mut self, _camera: &mut Camera, _dx: f64, _dy: f64) {}

    pub fn handle_scroll(&mut self, _camera: &mut Camera, _delta: f32) {}

    pub fn update(&self, _camera: &mut Camera, _dt: f32) {}
}

pub struct OrbitCamera {
    is_dragging: bool,
    is_shift_held: bool,
    rotate_speed: f32,
    zoom_speed: f32,
    pan_speed: f32,
    min_distance: f32,
    max_distance: f32,
    min_pitch: f32,
    max_pitch: f32,
}

impl OrbitCamera {
    pub fn new() -> Self {
        Self {
            is_dragging: false,
            is_shift_held: false,
            rotate_speed: 0.005,
            zoom_speed: 0.5,
            pan_speed: 0.001,
            min_distance: 1.0,
            max_distance: 1000.0,
            min_pitch: 0.0,
            max_pitch: 89.0_f32.to_radians(),
        }
    }

    pub fn handle_key(&mut self, code: KeyCode, pressed: bool) -> bool {
        match code {
            KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                self.is_shift_held = pressed;
                true
            }
            _ => false,
        }
    }

    pub fn handle_mouse_button(&mut self, button: MouseButton, pressed: bool) -> bool {
        if button == MouseButton::Right {
            self.is_dragging = pressed;
            true
        } else {
            false
        }
    }

    pub fn handle_mouse_motion(&mut self, camera: &mut Camera, dx: f64, dy: f64) {
        if !self.is_dragging {
            return;
        }

        if self.is_shift_held {
            self.pan(camera, dx, dy);
        } else {
            self.rotate(camera, dx, dy);
        }
    }

    fn rotate(&self, camera: &mut Camera, dx: f64, dy: f64) {
        let offset = camera.eye - camera.target;
        let distance = offset.magnitude();
        let yaw = offset.x.atan2(offset.z) - dx as f32 * self.rotate_speed;
        let pitch = ((offset.y / distance).clamp(-1.0, 1.0).asin() + dy as f32 * self.rotate_speed)
            .clamp(self.min_pitch, self.max_pitch);

        camera.eye = camera.target
            + Vector3::new(
                pitch.cos() * yaw.sin(),
                pitch.sin(),
                pitch.cos() * yaw.cos(),
            ) * distance;
    }

    fn pan(&self, camera: &mut Camera, dx: f64, dy: f64) {
        let offset = camera.eye - camera.target;
        let distance = offset.magnitude();
        let yaw = offset.x.atan2(offset.z);

        let right = Vector3::new(yaw.cos(), 0.0, -yaw.sin());
        let forward = -offset.normalize();
        let up = right.cross(forward);

        let shift = (right * -dx as f32 + up * dy as f32) * self.pan_speed * distance;
        camera.eye += shift;
        camera.target += shift;
    }

    pub fn handle_scroll(&mut self, camera: &mut Camera, delta: f32) {
        let offset = camera.eye - camera.target;
        let distance = offset.magnitude();
        let new_distance =
            (distance - delta * self.zoom_speed).clamp(self.min_distance, self.max_distance);

        camera.eye = camera.target + offset.normalize() * new_distance;
    }

    pub fn update(&self, _camera: &mut Camera, _dt: f32) {}
}
