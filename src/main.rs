#![allow(non_snake_case)]
// Defines a simple test model: The Cornel Box

// Loads the Cornell Box. It is scaled to fill the volume:
// -1 <= x <= +1
// -1 <= y <= +1
// -1 <= z <= +1
fn load_test_model() -> Vec<TriMesh> {
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

        triangle.v0 -= Vector3::new(1.0, 1.0, 1.0);
        triangle.v1 -= Vector3::new(1.0, 1.0, 1.0);
        triangle.v2 -= Vector3::new(1.0, 1.0, 1.0);

        triangle.v0.x *= -1.0;
        triangle.v1.x *= -1.0;
        triangle.v2.x *= -1.0;

        triangle.v0.y *= -1.0;
        triangle.v1.y *= -1.0;
        triangle.v2.y *= -1.0;

        triangle.update_normal();
    }

    meshes
}

// ----------------------------------------------------------------------------
// GLOBAL VARIABLES

use std::f32::consts::PI;

use kth_dd2323_project::{Color, Intersection, Ray, TriMesh, Triangle};
use nalgebra::{Matrix3, Vector3};
use once_cell::sync::Lazy;
use sdl2::{event::Event, keyboard::Keycode, render::Canvas, video::Window};

const SCREEN_WIDTH: u32 = 500;
const SCREEN_HEIGHT: u32 = 500;
const FOCAL_LENGTH: u32 = SCREEN_HEIGHT / 2;
const CAMERA_MOVEMENT_SPEED: f32 = 1.0;
const CAMERA_ROTATION_SPEED: f32 = 1.0;
const LIGHT_MOVEMENT_SPEED: f32 = 1.0;
const cameraPosition: Vector3<f32> = Vector3::new(0.0, 0.0, -2.0);
const cameraRotation: Matrix3<f32> = Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);
// float yaw;
// SDL2Aux *sdlAux;
// int t;
const lightPos: Vector3<f32> = Vector3::new(0.0, -0.5, -0.7);
const lightColor: Vector3<f32> = Vector3::new(14.0, 14.0, 14.0);
const indirectLight: Vector3<f32> = Vector3::new(0.5, 0.5, 0.5);

static TRIANGLES: Lazy<Vec<Triangle>> = Lazy::new(|| {
    load_test_model()
        .into_iter()
        .flat_map(|a| a.into_triangles())
        .collect()
});

// ----------------------------------------------------------------------------
// FUNCTIONS

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::BLUE.to_sdl());
    canvas.clear();

    canvas.present();

    let timer = sdl_context.timer().unwrap();

    let mut t = timer.ticks();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        let t2 = timer.ticks();

        let dt = t2 - t;

        println!("Render time: {}ms", dt);

        t = t2;

        Update(dt as f32);
        draw(&mut canvas);
    }
}

fn Update(_dt: f32) {

    // Move and rotate camera
    // const Uint8 *keystate = SDL_GetKeyboardState(NULL);
    // if (keystate[SDL_SCANCODE_UP])
    // {
    // 	// cameraPosition += vec3(0, 0, 1).rot() * dt / 1000 * CAMERA_MOVEMENT_SPEED;
    // 	cameraPosition += (cameraRotation * vec3(0, 0, 1)) * (dt / 1000) * CAMERA_MOVEMENT_SPEED;
    // }
    // if (keystate[SDL_SCANCODE_DOWN])
    // {
    // 	// Move camera backward
    // 	cameraPosition += (cameraRotation * vec3(0, 0, -1)) * (dt / 1000) * CAMERA_MOVEMENT_SPEED;
    // }
    // if (keystate[SDL_SCANCODE_LEFT])
    // {
    // 	// Rotate camera left
    // 	// cameraPosition.x -= dt / 1000 * CAMERA_MOVEMENT_SPEED;
    // 	yaw += dt / 1000 * CAMERA_ROTATION_SPEED;

    // 	// Update values that don't change
    // 	cameraRotation[0][0] = cos(yaw);
    // 	cameraRotation[0][2] = sin(yaw);
    // 	cameraRotation[2][0] = -sin(yaw);
    // 	cameraRotation[2][2] = cos(yaw);
    // }
    // if (keystate[SDL_SCANCODE_RIGHT])
    // {
    // 	// Rotate camera right
    // 	// cameraPosition.x += dt / 1000 * CAMERA_MOVEMENT_SPEED;
    // 	yaw -= dt / 1000 * CAMERA_ROTATION_SPEED;

    // 	// Update values that don't change
    // 	cameraRotation[0][0] = cos(yaw);
    // 	cameraRotation[0][2] = sin(yaw);
    // 	cameraRotation[2][0] = -sin(yaw);
    // 	cameraRotation[2][2] = cos(yaw);
    // }

    // // Move light position
    // if (keystate[SDL_SCANCODE_W])
    // {
    // 	// Move light forward
    // 	lightPos.z += (dt / 1000) * LIGHT_MOVEMENT_SPEED;
    // }
    // if (keystate[SDL_SCANCODE_S])
    // {
    // 	// Move light backwards
    // 	lightPos.z -= (dt / 1000) * LIGHT_MOVEMENT_SPEED;
    // }
    // if (keystate[SDL_SCANCODE_A])
    // {
    // 	// Move light left
    // 	lightPos.x -= (dt / 1000) * LIGHT_MOVEMENT_SPEED;
    // }
    // if (keystate[SDL_SCANCODE_D])
    // {
    // 	// Move light right
    // 	lightPos.x += (dt / 1000) * LIGHT_MOVEMENT_SPEED;
    // }
    // if (keystate[SDL_SCANCODE_Q])
    // {
    // 	// Move light up
    // 	lightPos.y -= (dt / 1000) * LIGHT_MOVEMENT_SPEED;
    // }
    // if (keystate[SDL_SCANCODE_E])
    // {
    // 	// Move light down
    // 	lightPos.y += (dt / 1000) * LIGHT_MOVEMENT_SPEED;
    // }
}

fn draw(canvas: &mut Canvas<Window>) {
    canvas.clear();

    for y in 0..SCREEN_HEIGHT as i32 {
        for x in 0..SCREEN_WIDTH as i32 {
            let direction = cameraRotation
                * Vector3::new(
                    (x - (SCREEN_WIDTH as i32 / 2)) as f32,
                    (y - (SCREEN_HEIGHT as i32 / 2)) as f32,
                    FOCAL_LENGTH as f32,
                );

            let mut color = Color::new(0.0, 0.0, 0.0);

            // Get color from triangle
            if let Some((intersection, triangle_index)) =
                closest_intersection(cameraPosition, direction, TRIANGLES.iter())
            {
                let reflect_fraction = TRIANGLES[triangle_index].color;
                let light = direct_light(&intersection, &TRIANGLES[triangle_index]) + indirectLight;
                color.0 = reflect_fraction.0.component_mul(&light);
            }

            canvas.set_draw_color(color.to_sdl());
            canvas.draw_point((x, y)).unwrap();
        }
    }

    canvas.present();
}

fn closest_intersection<'a>(
    start: Vector3<f32>,
    dir: Vector3<f32>,
    local_triangles: impl IntoIterator<Item = &'a Triangle>,
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

fn direct_light(i: &Intersection, triangle: &Triangle) -> Vector3<f32> {
    let P = lightColor;

    let mut rvec = lightPos - i.position;
    let r = rvec.norm();
    rvec = rvec.normalize();

    if closest_intersection(i.position, rvec, TRIANGLES.iter())
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
