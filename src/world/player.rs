use glam::Vec3;
use winit::keyboard::KeyCode;

use crate::constants::{
    BASE_SPEED, PLAYER_EYE_HEIGHT, PLAYER_FOREHEAD, PLAYER_RADIUS, SPRINT_MULTI,
};

/// Contains player-local information, such as eye/camera position and rotation.
pub struct Player {
    /// The player's eye/camera position in world space.
    pub eye: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl Player {
    pub fn new(eye: Vec3, yaw: f32, pitch: f32) -> Self {
        Self { eye, yaw, pitch }
    }

    /// Returns the player's collision box as `(min, max)` world-space corners.
    pub fn collider_aabb(&self) -> (Vec3, Vec3) {
        let min = self.eye + Vec3::new(-PLAYER_RADIUS, -PLAYER_EYE_HEIGHT, -PLAYER_RADIUS);
        let max = self.eye + Vec3::new(PLAYER_RADIUS, PLAYER_FOREHEAD, PLAYER_RADIUS);

        (min, max)
    }

    /// Returns the forward vector of the player. This returns a unit vector which
    /// is looking in the direction of the player's orientation.
    pub fn forward(&self) -> Vec3 {
        // pitch controls the vertical component directly, while pitch cosine
        // shrinks the horizontal X/Z components as the player looks up/down
        let pitch_sin = self.pitch.to_radians().sin();

        // yaw rotates around the Y axis. At yaw = 0, the player looks along +Z;
        // positive yaw turns the forward vector toward +X.
        let (yaw_sin, yaw_cos) = self.yaw.to_radians().sin_cos();

        (yaw_sin, pitch_sin, yaw_cos).into()
    }

    /// Returns the right vector of the player. This returns a unit vector which
    /// is perpendicular to the player's forward vector.
    pub fn right(&self) -> Vec3 {
        Vec3::Y.cross(self.forward()).normalize()
    }
}

/// Handles input for the player and contains controller-related state.
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
        Self::new(BASE_SPEED, (0.15, 0.1))
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
            KeyCode::KeyE | KeyCode::Space => {
                self.is_up_pressed = is_pressed;
                true
            }
            KeyCode::KeyQ | KeyCode::ControlLeft => {
                self.is_down_pressed = is_pressed;
                true
            }
            KeyCode::ShiftLeft => {
                if is_pressed {
                    self.speed = BASE_SPEED * SPRINT_MULTI;
                } else {
                    self.speed = BASE_SPEED;
                }
                true
            }
            _ => false,
        }
    }

    pub fn update_player(&self, player: &mut Player, dt_seconds: f32) {
        let forward = player.forward();
        let right = player.right();
        let mut direction = Vec3::ZERO;

        if self.is_forward_pressed {
            direction += forward;
        }
        if self.is_backward_pressed {
            direction -= forward;
        }
        if self.is_right_pressed {
            direction += right;
        }
        if self.is_left_pressed {
            direction -= right;
        }
        if self.is_up_pressed {
            direction += Vec3::Y;
        }
        if self.is_down_pressed {
            direction -= Vec3::Y;
        }

        // normalize player direction to avoid speedup when moving diagonally
        if direction.length_squared() > 0.0 {
            player.eye += direction.normalize() * self.speed * dt_seconds;
        }
    }
}
