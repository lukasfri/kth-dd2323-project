use std::f32::consts::PI;

use nalgebra::Vector3;
use sdl2::{render::Canvas, video::Window};

use crate::{camera::Camera, scene::Scene, Color, Intersectable, Intersection, Ray, Triangle};

use super::renderer::Renderer;

pub struct Raytracer;

impl Renderer for Raytracer {
    type Error = ();
    type Canvas<'a> = Canvas<Window>;

    fn render(
        &mut self,
        canvas: &mut Canvas<Window>,
        scene: &Scene,
        camera: &Camera,
    ) -> Result<(), Self::Error> {
        canvas.clear();
        let (width, height) = canvas.window().drawable_size();
        for y in 0..width as i32 {
            for x in 0..height as i32 {
                let direction = camera.rotation()
                    * Vector3::new(
                        camera.focal_length,
                        (x - (width as i32 / 2)) as f32,
                        (-y + (height as i32 / 2)) as f32,
                    );

                let mut color = Color::new(0.0, 0.0, 0.0);

                // Get color from triangle
                if let Some((intersection, triangle_index)) =
                    closest_intersection(camera.position, direction, scene.triangles.iter())
                {
                    let reflect_fraction = scene.triangles[triangle_index].color;
                    let light =
                        direct_light(scene, &intersection, &scene.triangles[triangle_index])
                            + scene.indirect_light;
                    color.0 = reflect_fraction.0.component_mul(&light);
                }

                canvas.set_draw_color(color.to_sdl());
                canvas.draw_point((x, y)).unwrap();
            }
        }

        canvas.present();
        Ok(())
    }
}

fn closest_intersection<'a>(
    start: Vector3<f32>,
    dir: Vector3<f32>,
    local_triangles: impl IntoIterator<Item = &'a (impl Intersectable + 'a)>,
) -> Option<(Intersection, usize)> {
    let mut closest_intersection: Option<(Intersection, usize)> = None;

    for (intersection, i) in local_triangles
        .into_iter()
        .enumerate()
        .filter_map(|(i, triangle)| {
            Ray::new(start, dir)
                .intersect(triangle)
                .map(|intersection| (intersection, i))
        })
        // If intersection is start, ignore it.
        .filter(|(intersection, _)| intersection.distance >= 0.000001)
    {
        // If intersection is not closer than the current closest one
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        if !(intersection.distance
            < closest_intersection
                .as_ref()
                .map_or(f32::MAX, |(i, _)| i.distance))
        {
            continue;
        }

        closest_intersection = Some((intersection, i));
    }

    closest_intersection
}

fn direct_light(scene: &Scene, i: &Intersection, triangle: &Triangle) -> Vector3<f32> {
    #![allow(non_snake_case)]
    let P = scene.light_color;

    let mut rvec = scene.light_pos - i.position;
    let r = rvec.norm();
    rvec = rvec.normalize();

    if closest_intersection(i.position, rvec, scene.triangles.iter())
        // Is there an intersection closer than the light?
        .filter(|(intersection, _)| intersection.distance < r)
        .is_some()
    {
        return Vector3::new(0.0, 0.0, 0.0);
    }

    let mut B = P;
    B /= 4.0 * PI * r * r;

    B * rvec.dot(&triangle.normal).max(0.0)
}
