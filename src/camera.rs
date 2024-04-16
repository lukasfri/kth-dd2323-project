use nalgebra::{Rotation, Rotation3, Vector3};

pub struct Camera {
    pub focal_length: f32,
    pub position: Vector3<f32>,
    pub rotation: Rotation3<f32>,
    pub yaw: f32,
    pub pitch: f32,
}

impl Camera {
    pub fn update_rotation(&mut self) {
        self.rotation = Rotation::from_euler_angles(0.0, self.pitch, self.yaw);
    }

    pub fn new(focal_length: f32, position: Vector3<f32>) -> Camera {
        Camera {
            focal_length,
            position,
            rotation: Rotation3::default(),
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}
