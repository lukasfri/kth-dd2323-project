use easy_gltf::model::Vertex;
use nalgebra::{Matrix3, Vector3};

pub struct Ray {
    pub origin: Vector3<f32>,
    pub direction: Vector3<f32>,
}

pub struct TriMesh {
    pub triangles: Vec<Triangle>,
}

impl TriMesh {
    pub fn new(triangles: Vec<Triangle>) -> Self {
        Self { triangles }
    }

    pub fn triangles(&self) -> &Vec<Triangle> {
        &self.triangles
    }

    pub fn triangles_mut(&mut self) -> &mut Vec<Triangle> {
        &mut self.triangles
    }

    pub fn into_triangles(self) -> Vec<Triangle> {
        self.triangles
    }
}

pub struct Triangle {
    pub v0: Vector3<f32>,
    pub v1: Vector3<f32>,
    pub v2: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub color: Color,
}

impl Triangle {
    pub fn new(v0: Vector3<f32>, v1: Vector3<f32>, v2: Vector3<f32>, color: Color) -> Self {
        Self {
            v0,
            v1,
            v2,
            color,
            normal: Self::calculate_normal(v0, v1, v2),
        }
    }

    pub fn new_from_gltf(vertices: [Vertex; 3], color: Color) -> Self {
        let mut triangle = Self {
            v0: Vector3::new(
                vertices[0].position.x,
                vertices[0].position.y,
                vertices[0].position.z,
            ),
            v1: Vector3::new(
                vertices[1].position.x,
                vertices[1].position.y,
                vertices[1].position.z,
            ),
            v2: Vector3::new(
                vertices[2].position.x,
                vertices[2].position.y,
                vertices[2].position.z,
            ),
            color,
            normal: Vector3::zeros(),
        };
        triangle.update_normal();
        triangle
    }

    fn calculate_normal(v0: Vector3<f32>, v1: Vector3<f32>, v2: Vector3<f32>) -> Vector3<f32> {
        let e1 = v1 - v0;
        let e2 = v2 - v0;
        e2.cross(&e1).normalize()
    }

    pub fn update_normal(&mut self) {
        self.normal = Self::calculate_normal(self.v0, self.v1, self.v2);
    }
}

pub struct Intersection {
    pub position: Vector3<f32>,
    pub distance: f32,
}

pub trait Intersectable {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
}

impl Intersectable for Triangle {
    #[inline(always)]
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let v0 = self.v0;
        let v1 = self.v1;
        let v2 = self.v2;
        let e1 = v1 - v0;
        let e2 = v2 - v0;
        let b = ray.origin - v0;
        let a_matrix = Matrix3::from_columns(&[-ray.direction, e1, e2]);
        let tuv = a_matrix.try_inverse()? * b;
        let t = tuv.x;
        let u = tuv.y;
        let v = tuv.z;

        // Inside triangle
        // Continues if any of the conditions are false
        // We know the following conditions could be written without the not-operators,
        // but we've kept them to keep the original equations clear.
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        if !(t >= 0.0) || !(u >= 0.0) || !(v >= 0.0) || !(u + v <= 1.0) {
            return None;
        }

        Some(Intersection {
            position: v0 + u * e1 + v * e2,
            distance: t,
        })
    }
}

impl Ray {
    #[inline(always)]
    pub fn new(origin: Vector3<f32>, direction: Vector3<f32>) -> Ray {
        Ray { origin, direction }
    }

    #[inline(always)]
    pub fn intersect(&self, intersectable: &impl Intersectable) -> Option<Intersection> {
        intersectable.intersect(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color(pub Vector3<f32>);

impl Color {
    pub const RED: Color = Color::new(1.0, 0.0, 0.0);
    pub const GREEN: Color = Color::new(0.0, 1.0, 0.0);
    pub const BLUE: Color = Color::new(0.0, 0.0, 1.0);
    pub const BLACK: Color = Color::new(0.0, 0.0, 0.0);
    pub const WHITE: Color = Color::new(1.0, 1.0, 1.0);

    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Color(Vector3::new(r, g, b))
    }

    #[inline(always)]
    pub fn to_sdl(&self) -> sdl2::pixels::Color {
        sdl2::pixels::Color::RGB(
            (self.0.x * 255.0) as u8,
            (self.0.y * 255.0) as u8,
            (self.0.z * 255.0) as u8,
        )
    }
}
