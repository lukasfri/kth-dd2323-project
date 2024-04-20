use nalgebra::Vector3;

use crate::Triangle;

pub struct Scene {
    pub triangles: Vec<Triangle>,
    pub light_pos: Vector3<f32>,
    pub light_color: Vector3<f32>,
    pub indirect_light: Vector3<f32>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    // Loads a model into the world by copying
    pub fn instantiate_model(&mut self, model: &[Triangle], position: Vector3<f32>) {
        let mut instantiated_model = model
            .to_owned()
            .iter()
            .map(|triangle| triangle.translate(position))
            .collect::<Vec<Triangle>>();
        self.triangles.append(&mut instantiated_model);
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            triangles: vec![],
            light_pos: Vector3::new(-0.5, 0.0, 0.7),
            light_color: Vector3::new(14.0, 14.0, 14.0),
            indirect_light: Vector3::new(0.5, 0.5, 0.5),
        }
    }
}
