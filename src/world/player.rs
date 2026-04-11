use glam::Vec3;
use winit::keyboard::KeyCode;

pub struct Player {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl Player {
    pub fn new(position: Vec3, yaw: f32, pitch: f32) -> Self {
        Self {
            position,
            yaw,
            pitch,
        }
    }

    pub fn forward(&self) -> Vec3 {
        let (pitch_sin, pitch_cos) = self.pitch.to_radians().sin_cos();
        let (yaw_sin, yaw_cos) = self.yaw.to_radians().sin_cos();

        (yaw_sin * pitch_cos, pitch_sin, yaw_cos * pitch_cos).into()
    }

    pub fn right(&self) -> Vec3 {
        Vec3::Y.cross(self.forward()).normalize()
    }
}

pub struct PlayerController {
    speed: f32,
    sensitivity_x: f32,
    sensitivity_y: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
}

impl Default for PlayerController {
    fn default() -> Self {
        Self::new(1.0, (0.15, 0.1))
    }
}

impl PlayerController {
    pub fn new(speed: f32, sensitivity: (f32, f32)) -> Self {
        let (sensitivity_x, sensitivity_y) = sensitivity;

        Self {
            speed,
            sensitivity_x,
            sensitivity_y,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64, player: &mut Player) {
        player.yaw += mouse_dx as f32 * self.sensitivity_x;
        player.pitch -= mouse_dy as f32 * self.sensitivity_y;

        player.pitch = player.pitch.clamp(-89.0, 89.0);
    }

    pub fn process_events(&mut self, key: KeyCode, is_pressed: bool) -> bool {
        match key {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            }
            KeyCode::Space => {
                self.is_up_pressed = is_pressed;
                true
            }
            KeyCode::ShiftLeft => {
                self.is_down_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub fn update_player(&self, player: &mut Player) {
        let forward = player.forward();
        let right = player.right();

        if self.is_forward_pressed {
            player.position += forward * self.speed;
        }
        if self.is_backward_pressed {
            player.position -= forward * self.speed;
        }
        if self.is_right_pressed {
            player.position += right * self.speed;
        }
        if self.is_left_pressed {
            player.position -= right * self.speed;
        }
        if self.is_up_pressed {
            player.position += Vec3::Y * self.speed;
        }
        if self.is_down_pressed {
            player.position -= Vec3::Y * self.speed;
        }
    }
}
