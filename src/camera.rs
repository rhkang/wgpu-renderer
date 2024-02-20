use bytemuck::{Pod, Zeroable};
use cgmath::{InnerSpace, SquareMatrix};
use winit::event::{ElementState, WindowEvent};

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: (0.0, -2.0, 1.5).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: 1.0,
            fovy: 60.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    pub fn set_initial_eye_pos(&mut self, pos: cgmath::Point3<f32>) {
        self.eye = pos;
    }
}

pub struct CameraController {
    move_speed: f32,
    angle_speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl Default for CameraController {
    fn default() -> Self {
        CameraController::new(0.1, 0.05)
    }
}

impl CameraController {
    fn new(speed: f32, angle_speed: f32) -> Self {
        Self {
            move_speed: speed,
            angle_speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn process_window_events(&mut self, event: &WindowEvent) -> bool {
        use winit::keyboard::{KeyCode, PhysicalKey};
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                let is_pressed = event.state == ElementState::Pressed;
                match event.physical_key {
                    PhysicalKey::Code(code) => match code {
                        KeyCode::ArrowUp => {
                            self.is_forward_pressed = is_pressed;
                            true
                        }
                        KeyCode::ArrowDown => {
                            self.is_backward_pressed = is_pressed;
                            true
                        }
                        KeyCode::KeyW => {
                            self.is_up_pressed = is_pressed;
                            true
                        }
                        KeyCode::KeyS => {
                            self.is_down_pressed = is_pressed;
                            true
                        }
                        KeyCode::KeyA => {
                            self.is_left_pressed = is_pressed;
                            true
                        }
                        KeyCode::KeyD => {
                            self.is_right_pressed = is_pressed;
                            true
                        }
                        _ => false,
                    },
                    PhysicalKey::Unidentified(_) => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        pub const MAX_RANGE: f32 = 100000000.0;

        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        if self.is_forward_pressed && forward_mag > self.move_speed {
            camera.eye += forward_norm * self.move_speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.move_speed;
        }

        let right = forward_norm.cross(camera.up).normalize();

        let local_up = forward_norm
            .cross(cgmath::Vector3::unit_x() * -1.0)
            .normalize();

        if self.is_up_pressed {
            camera.target = camera.eye + (forward_norm + local_up * self.angle_speed).normalize();
        }
        if self.is_down_pressed {
            camera.target = camera.eye + (forward_norm - local_up * self.angle_speed).normalize();
        }
        if self.is_right_pressed {
            camera.target = camera.eye + (forward_norm + right * self.angle_speed).normalize();
        }
        if self.is_left_pressed {
            camera.target = camera.eye + (forward_norm - right * self.angle_speed).normalize();
        }

        let forward_final = (camera.target - camera.eye).normalize();
        camera.target = camera.eye + forward_final * MAX_RANGE;
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct CameraUniform {
    transformation: [[f32; 4]; 4],
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            transformation: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_transformation(&mut self, camera: &Camera) {
        self.transformation = camera.build_view_projection_matrix().into();
    }
}
