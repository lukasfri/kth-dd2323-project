use nalgebra::{Rotation, Rotation3, Vector3};

pub struct Camera {
    pub focal_length: f32,
    pub position: Vector3<f32>,
    rotation: Rotation3<f32>,
    yaw: f32,
    pitch: f32,
}

impl Camera {
    pub fn new(focal_length: f32, position: Vector3<f32>) -> Camera {
        Camera {
            focal_length,
            position,
            rotation: Rotation3::default(),
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    pub fn rotation(&self) -> Rotation3<f32> {
        self.rotation
    }

    pub fn move_relative(&mut self, movement: Vector3<f32>) {
        self.position += self.rotation * movement;
    }

    pub fn update_rotation(&mut self, pitch: f32, yaw: f32) {
        self.pitch = pitch;
        self.yaw = yaw;
        self.rotation = Rotation::from_euler_angles(0.0, pitch, yaw);
    }
}
