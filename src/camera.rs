use nalgebra::{Rotation, Rotation3, Vector3};

pub struct Camera {
    pub focal_length: f32,
    pub position: Vector3<f32>,
    pub rotation: Rotation3<f32>,
}

impl Camera {
    pub fn new(focal_length: f32, position: Vector3<f32>, direction: Vector3<f32>) -> Camera {
        Camera {
            focal_length,
            position,
            rotation: Rotation3::face_towards(&direction, &Vector3::z()),
        }
    }

    pub fn yaw(&self) -> f32 {
        self.rotation.euler_angles().2
    }

    pub fn pitch(&self) -> f32 {
        self.rotation.euler_angles().0
    }

    pub fn rotation(&self) -> Rotation3<f32> {
        self.rotation
    }

    pub fn move_relative(&mut self, movement: Vector3<f32>) {
        self.position += self.rotation * movement;
    }

    pub fn update_rotation(&mut self, pitch: f32, yaw: f32) {
        self.rotation = Rotation::from_euler_angles(pitch, 0.0, yaw);
    }

    pub fn direction_vector(&self) -> Vector3<f32> {
        self.rotation * Vector3::new(0.0, 0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::Vector3;

    use super::Camera;

    #[test]
    fn camera_rotation() {
        let focal_length: u32 = 500 / 2;
        let camera = Camera::new(
            focal_length as f32,
            Vector3::new(-3.5f32, 2.0, 2.0),
            -Vector3::new(-3.5f32, 2.0, 2.0),
        );

        println!("{:?}", camera.direction_vector());

        println!("{:?}", camera.rotation().euler_angles());
        assert_eq!(camera.direction_vector(), -camera.position.normalize());
    }
}
