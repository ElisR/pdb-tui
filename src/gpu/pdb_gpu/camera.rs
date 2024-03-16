use crate::gpu::pdb_gpu::input::{UnifiedEvent, UnifiedKeyCode, UnifiedKeyKind};

pub struct Camera {
    pub eye: nalgebra::Point3<f32>,
    pub target: nalgebra::Point3<f32>,
    pub up: nalgebra::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> nalgebra::Matrix4<f32> {
        let view =
            nalgebra::Isometry3::look_at_rh(&self.eye, &self.target, &self.up).to_homogeneous();
        let proj = nalgebra::Perspective3::new(self.aspect, self.fovy, self.znear, self.zfar)
            .to_homogeneous();
        proj * view
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_position: [f32; 4],
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: nalgebra::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        // We're using Vector4 because ofthe camera_uniform 16 byte spacing requirement
        self.view_position = camera.eye.to_homogeneous().into();
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: nalgebra::Matrix4::identity().into(),
        }
    }
}

pub struct CameraController {
    pub speed: f32,
    pub is_up_pressed: bool,
    pub is_down_pressed: bool,
    pub is_forward_pressed: bool,
    pub is_backward_pressed: bool,
    pub is_left_pressed: bool,
    pub is_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_up_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: UnifiedEvent) -> bool {
        let is_pressed = event.kind == UnifiedKeyKind::Press;
        match event.keycode {
            UnifiedKeyCode::Space => {
                self.is_up_pressed = is_pressed;
                true
            }
            UnifiedKeyCode::Shift => {
                self.is_down_pressed = is_pressed;
                true
            }
            UnifiedKeyCode::U | UnifiedKeyCode::K | UnifiedKeyCode::Up => {
                self.is_forward_pressed = is_pressed;
                true
            }
            UnifiedKeyCode::H | UnifiedKeyCode::Left => {
                self.is_left_pressed = is_pressed;
                true
            }
            UnifiedKeyCode::D | UnifiedKeyCode::J | UnifiedKeyCode::Down => {
                self.is_backward_pressed = is_pressed;
                true
            }
            UnifiedKeyCode::L | UnifiedKeyCode::Right => {
                self.is_right_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(&camera.up);

        // Redo radius calc in case the up/ down is pressed.
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and eye so
            // that it doesn't change. The eye therefore still
            // lies on the circle made by the target and eye.
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
}
