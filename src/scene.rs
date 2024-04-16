use std::f32::consts::PI;

use nalgebra::{Rotation3, Vector3};

use crate::{Color, TriMesh, Triangle};

pub struct Scene {
    pub triangles: Vec<Triangle>,
    pub light_pos: Vector3<f32>,
    pub light_color: Vector3<f32>,
    pub indirect_light: Vector3<f32>,
}

impl Scene {
    /// Defines a simple test model: The Cornel Box
    ///
    /// Loads the Cornell Box. It is scaled to fill the volume:
    /// -1 <= x <= +1
    /// -1 <= y <= +1
    /// -1 <= z <= +1
    pub fn load_cornell_box() -> Self {
        #![allow(non_snake_case)]

        let mut meshes = Vec::new();

        // Defines colors:
        let red = Color::new(0.75, 0.15, 0.15);
        let yellow = Color::new(0.75, 0.75, 0.15);
        let green = Color::new(0.15, 0.75, 0.15);
        let cyan = Color::new(0.15, 0.75, 0.75);
        let blue = Color::new(0.15, 0.15, 0.75);
        let purple = Color::new(0.75, 0.15, 0.75);
        let white = Color::new(0.75, 0.75, 0.75);

        // ---------------------------------------------------------------------------
        // Room

        let mut local_triangles = Vec::new();

        let L = 555.0; // Length of Cornell Box side.

        let A = Vector3::new(L, 0.0, 0.0);
        let B = Vector3::new(0.0, 0.0, 0.0);
        let C = Vector3::new(L, 0.0, L);
        let D = Vector3::new(0.0, 0.0, L);

        let E = Vector3::new(L, L, 0.0);
        let F = Vector3::new(0.0, L, 0.0);
        let G = Vector3::new(L, L, L);
        let H = Vector3::new(0.0, L, L);

        // Floor:
        local_triangles.push(Triangle::new(C, B, A, green));
        local_triangles.push(Triangle::new(C, D, B, green));

        // Left wall
        local_triangles.push(Triangle::new(A, E, C, purple));
        local_triangles.push(Triangle::new(C, E, G, purple));

        // Right wall
        local_triangles.push(Triangle::new(F, B, D, yellow));
        local_triangles.push(Triangle::new(H, F, D, yellow));

        // Ceiling
        local_triangles.push(Triangle::new(E, F, G, cyan));
        local_triangles.push(Triangle::new(F, H, G, cyan));

        // Back wall
        local_triangles.push(Triangle::new(G, D, C, white));
        local_triangles.push(Triangle::new(G, H, D, white));

        meshes.push(TriMesh::new(local_triangles));

        // ---------------------------------------------------------------------------
        // Short block

        let mut local_triangles = Vec::new();

        let A = Vector3::new(290.0, 0.0, 114.0);
        let B = Vector3::new(130.0, 0.0, 65.0);
        let C = Vector3::new(240.0, 0.0, 272.0);
        let D = Vector3::new(82.0, 0.0, 225.0);

        let E = Vector3::new(290.0, 165.0, 114.0);
        let F = Vector3::new(130.0, 165.0, 65.0);
        let G = Vector3::new(240.0, 165.0, 272.0);
        let H = Vector3::new(82.0, 165.0, 225.0);

        // Front
        local_triangles.push(Triangle::new(E, B, A, red));
        local_triangles.push(Triangle::new(E, F, B, red));

        // Front
        local_triangles.push(Triangle::new(F, D, B, red));
        local_triangles.push(Triangle::new(F, H, D, red));

        // BACK
        local_triangles.push(Triangle::new(H, C, D, red));
        local_triangles.push(Triangle::new(H, G, C, red));

        // LEFT
        local_triangles.push(Triangle::new(G, E, C, red));
        local_triangles.push(Triangle::new(E, A, C, red));

        // TOP
        local_triangles.push(Triangle::new(G, F, E, red));
        local_triangles.push(Triangle::new(G, H, F, red));

        meshes.push(TriMesh::new(local_triangles));

        // ---------------------------------------------------------------------------
        // Tall block

        let mut local_triangles = Vec::new();

        let A = Vector3::new(423.0, 0.0, 247.0);
        let B = Vector3::new(265.0, 0.0, 296.0);
        let C = Vector3::new(472.0, 0.0, 406.0);
        let D = Vector3::new(314.0, 0.0, 456.0);

        let E = Vector3::new(423.0, 330.0, 247.0);
        let F = Vector3::new(265.0, 330.0, 296.0);
        let G = Vector3::new(472.0, 330.0, 406.0);
        let H = Vector3::new(314.0, 330.0, 456.0);

        // Front
        local_triangles.push(Triangle::new(E, B, A, blue));
        local_triangles.push(Triangle::new(E, F, B, blue));

        // Front
        local_triangles.push(Triangle::new(F, D, B, blue));
        local_triangles.push(Triangle::new(F, H, D, blue));

        // BACK
        local_triangles.push(Triangle::new(H, C, D, blue));
        local_triangles.push(Triangle::new(H, G, C, blue));

        // LEFT
        local_triangles.push(Triangle::new(G, E, C, blue));
        local_triangles.push(Triangle::new(E, A, C, blue));

        // TOP
        local_triangles.push(Triangle::new(G, F, E, blue));
        local_triangles.push(Triangle::new(G, H, F, blue));

        meshes.push(TriMesh::new(local_triangles));

        // ----------------------------------------------
        // Scale to the volume [-1,1]^3

        for triangle in meshes.iter_mut().flat_map(|a| a.triangles_mut()) {
            triangle.v0 *= 2.0 / L;
            triangle.v1 *= 2.0 / L;
            triangle.v2 *= 2.0 / L;

            let rotation = Rotation3::from_euler_angles(PI / 2.0, 0.0, PI / 2.0);
            // let rotation = Rotation3::from_euler_angles(0.0, PI / 2.0, PI / 2.0);
            triangle.v0 = rotation * triangle.v0;
            triangle.v1 = rotation * triangle.v1;
            triangle.v2 = rotation * triangle.v2;

            triangle.v0 -= Vector3::new(1.0, 1.0, 1.0);
            triangle.v1 -= Vector3::new(1.0, 1.0, 1.0);
            triangle.v2 -= Vector3::new(1.0, 1.0, 1.0);

            triangle.update_normal();
        }

        Scene {
            triangles: meshes
                .into_iter()
                .flat_map(|a| a.into_triangles())
                .collect(),

            light_pos: Vector3::new(-0.5, 0.0, 0.7),
            light_color: Vector3::new(14.0, 14.0, 14.0),
            indirect_light: Vector3::new(0.5, 0.5, 0.5),
        }
    }

    #[allow(dead_code)]
    fn load_gltf_model(path: &str) -> Vec<TriMesh> {
        let mut scenes = easy_gltf::load(path)
            .expect("Failed to load glTF")
            .into_iter();
        let scene = scenes.next().expect("No scenes in glTF file");
        assert!(scenes.next().is_none(), "More than one scene in gltf file");
        let mut meshes = Vec::new();

        for model in scene.models {
            let mut local_triangles = Vec::new();

            // Only support triangle meshes
            if model.mode() == easy_gltf::model::Mode::Triangles {
                let color = Color::new_from_vector(
                    model
                        .material()
                        .get_base_color(cgmath::Vector2::<f32>::new(0.0, 0.0)),
                );
                if let Ok(gltf_triangles) = model.triangles() {
                    for gltf_triangle in gltf_triangles {
                        let triangle = Triangle::new_from_gltf(gltf_triangle, color);
                        local_triangles.push(triangle);
                    }
                }
            }

            meshes.push(TriMesh::new(local_triangles));
        }

        meshes
    }
}
