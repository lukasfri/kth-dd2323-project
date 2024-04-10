#![allow(non_snake_case)]
// Defines a simple test model: The Cornel Box

// Used to describe a triangular surface:
struct Triangle {
    v0: Vector3<f32>,
    v1: Vector3<f32>,
    v2: Vector3<f32>,
    normal: Vector3<f32>,
    color: Vector3<f32>,
}

impl Triangle {
    pub fn new(v0: Vector3<f32>, v1: Vector3<f32>, v2: Vector3<f32>, color: Vector3<f32>) -> Self {
        let mut triangle = Self {
            v0,
            v1,
            v2,
            color,
            normal: Vector3::default(),
        };
        triangle.compute_normal();
        triangle
    }

    pub fn compute_normal(&mut self) {
        let e1 = self.v1 - self.v0;
        let e2 = self.v2 - self.v0;
        self.normal = e2.cross(&e1).normalize();
    }
}

// Loads the Cornell Box. It is scaled to fill the volume:
// -1 <= x <= +1
// -1 <= y <= +1
// -1 <= z <= +1
fn load_test_model(local_triangles: &mut Vec<Triangle>) {
    // Defines colors:
    let red = Vector3::new(0.75, 0.15, 0.15);
    let yellow = Vector3::new(0.75, 0.75, 0.15);
    let green = Vector3::new(0.15, 0.75, 0.15);
    let cyan = Vector3::new(0.15, 0.75, 0.75);
    let blue = Vector3::new(0.15, 0.15, 0.75);
    let purple = Vector3::new(0.75, 0.15, 0.75);
    let white = Vector3::new(0.75, 0.75, 0.75);

    local_triangles.clear();
    local_triangles.reserve(5 * 2 * 3);

    // ---------------------------------------------------------------------------
    // Room

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

    // ---------------------------------------------------------------------------
    // Short block

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

    // ---------------------------------------------------------------------------
    // Tall block

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

    // ----------------------------------------------
    // Scale to the volume [-1,1]^3

    for local_triangle in local_triangles.iter_mut() {
        local_triangle.v0 *= 2.0 / L;
        local_triangle.v1 *= 2.0 / L;
        local_triangle.v2 *= 2.0 / L;

        local_triangle.v0 -= Vector3::new(1.0, 1.0, 1.0);
        local_triangle.v1 -= Vector3::new(1.0, 1.0, 1.0);
        local_triangle.v2 -= Vector3::new(1.0, 1.0, 1.0);

        local_triangle.v0.x *= -1.0;
        local_triangle.v1.x *= -1.0;
        local_triangle.v2.x *= -1.0;

        local_triangle.v0.y *= -1.0;
        local_triangle.v1.y *= -1.0;
        local_triangle.v2.y *= -1.0;

        local_triangle.compute_normal();
    }
}

// ----------------------------------------------------------------------------
// GLOBAL VARIABLES

use std::f32::consts::PI;

use nalgebra::{Matrix3, Vector3};
use once_cell::sync::Lazy;
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, render::Canvas, video::Window};

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
    let mut init_triangles = Vec::new();
    load_test_model(&mut init_triangles);
    init_triangles
});

#[derive(Debug, Clone, Copy, Default)]
struct Intersection {
    position: Vector3<f32>,
    distance: f32,
    triangle_index: usize,
}

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

    canvas.set_draw_color(Color::BLUE);
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

            let mut color = Vector3::<f32>::new(0.0, 0.0, 0.0);

            // Get color from triangle
            if let Some(intersection) = closest_intersection(cameraPosition, direction, &TRIANGLES)
            {
                let reflect_fraction = TRIANGLES[intersection.triangle_index].color;
                let light = direct_light(&intersection) + indirectLight;
                color = reflect_fraction.component_mul(&light);
            }

            let color = Color::RGB(
                (color.x * 255.0) as u8,
                (color.y * 255.0) as u8,
                (color.z * 255.0) as u8,
            );

            canvas.set_draw_color(color);
            canvas.draw_point((x, y)).unwrap();
        }
    }

    canvas.present();
}

fn closest_intersection(
    start: Vector3<f32>,
    dir: Vector3<f32>,
    local_triangles: &[Triangle],
) -> Option<Intersection> {
    let mut m = f32::MAX;
    let mut closest_intersection = None;

    for (i, triangle) in local_triangles.iter().enumerate() {
        let v0 = triangle.v0;
        let v1 = triangle.v1;
        let v2 = triangle.v2;
        let e1 = v1 - v0;
        let e2 = v2 - v0;
        let b = start - v0;
        let A = Matrix3::from_columns(&[-dir, e1, e2]);
        let tuv = A.try_inverse().map(|A| A * b);
        let Some(tuv) = tuv else {
            continue;
        };
        let t = tuv.x;
        let u = tuv.y;
        let v = tuv.z;

        // Inside triangle
        // Continues if any of the conditions are false
        // We know the following conditions could be written without the not-operators,
        // but we've kept them to keep the original equations clear.
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        if !(t >= 0.0) || !(u >= 0.0) || !(v >= 0.0) || !(u + v <= 1.0) {
            continue;
        }

        // If intersection is not closer than the current closest one
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        if !(t < m) {
            continue;
        }

        // If intersection is start, ignore it.
        if t.abs() < 0.000001 {
            continue;
        }

        m = t;
        closest_intersection = Some(Intersection {
            position: v0 + u * e1 + v * e2,
            distance: t,
            triangle_index: i,
        });
    }

    // If m is not max value anymore, we've found an intersection.
    closest_intersection
}

fn direct_light(i: &Intersection) -> Vector3<f32> {
    let triangle = &TRIANGLES[i.triangle_index];

    let P = lightColor;

    let mut rvec = lightPos - i.position;
    let r = rvec.norm();
    rvec = rvec.normalize();

    if let Some(intersection) = closest_intersection(i.position, rvec, &TRIANGLES) {
        if intersection.distance < r {
            return Vector3::new(0.0, 0.0, 0.0);
        }
    }

    let mut B = P;
    B /= 4.0 * PI * r * r;

    B * rvec.dot(&triangle.normal).max(0.0)
}
