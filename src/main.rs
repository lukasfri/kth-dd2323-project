#![allow(non_snake_case)]
use sdl2::{mouse::MouseButton, Sdl};
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

    meshes
}

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
            let color = Color::new_from_vector(model.material().get_base_color(cgmath::Vector2::<
                f32,
            >::new(
                0.0, 0.0
            )));
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

// ----------------------------------------------------------------------------
// GLOBAL VARIABLES

use std::f32::consts::PI;

use kth_dd2323_project::{Color, Intersectable, Intersection, Ray, TriMesh, Triangle};
use nalgebra::{Rotation, Rotation3, Vector2, Vector3};
use once_cell::sync::Lazy;
use sdl2::{
    event::Event,
    keyboard::{KeyboardState, Keycode, Scancode},
    mouse::MouseState,
    render::Canvas,
    video::Window,
};

const SCREEN_WIDTH: u32 = 500;
const SCREEN_HEIGHT: u32 = 500;
const FOCAL_LENGTH: u32 = SCREEN_HEIGHT / 2;
const CAMERA_MOVEMENT_SPEED: f32 = 1.0;
const CAMERA_ROTATION_SPEED: f32 = 1.0;
const CAMERA_MOUSE_ROTATION_SPEED: f32 = 0.02;
const LIGHT_COLOR: Vector3<f32> = Vector3::new(14.0, 14.0, 14.0);
const INDRECT_LIGHT: Vector3<f32> = Vector3::new(0.5, 0.5, 0.5);

static TRIANGLES: Lazy<Vec<Triangle>> = Lazy::new(|| {
    // load_gltf_model("./resources/test_model.glb")
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

    let mut camera_position: Vector3<f32> = Vector3::new(-4.0, 0.0, 0.0);
    let mut camera_rotation: Rotation3<f32> = Rotation3::default();
    let light_pos: Vector3<f32> = Vector3::new(-0.5, 0.0, 0.7);
    let mut mouse_reference_position: Option<Vector2<f32>> = None;

    let mut yaw = 0.0;
    let mut pitch = 0.0;

    let mut event_pump = sdl_context.event_pump().unwrap();

    let binds = Keybinds::default();
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
        println!(
            "Render time: {}ms camera pos {:?}, camera rot {:?}",
            dt, camera_position, camera_rotation
        );
        t = t2;

        let keyboard = KeyboardState::new(&event_pump);
        let mouse = MouseState::new(&event_pump);
        let state = binds.current_state(&keyboard, &mouse);
        update(
            dt as f32,
            &state,
            &sdl_context,
            canvas.window(),
            &mut yaw,
            &mut pitch,
            &mut mouse_reference_position,
            &mut camera_position,
            &mut camera_rotation,
        );

        draw(&mut canvas, &camera_position, &camera_rotation, &light_pos);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Keybinds {
    pub camera_forward: Option<Scancode>,
    pub camera_backward: Option<Scancode>,
    pub camera_left: Option<Scancode>,
    pub camera_right: Option<Scancode>,
    pub camera_up: Option<Scancode>,
    pub camera_down: Option<Scancode>,
    pub camera_rotate_left: Option<Scancode>,
    pub camera_rotate_right: Option<Scancode>,
    pub camera_rotate_up: Option<Scancode>,
    pub camera_rotate_down: Option<Scancode>,
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            camera_forward: Some(Scancode::W),
            camera_backward: Some(Scancode::S),
            camera_left: Some(Scancode::A),
            camera_right: Some(Scancode::D),
            camera_up: Some(Scancode::Space),
            camera_down: Some(Scancode::LShift),
            camera_rotate_left: Some(Scancode::Left),
            camera_rotate_right: Some(Scancode::Right),
            camera_rotate_up: Some(Scancode::Up),
            camera_rotate_down: Some(Scancode::Down),
        }
    }
}

impl Keybinds {
    pub fn current_state(
        &self,
        keyboard_state: &KeyboardState<'_>,
        mouse_state: &MouseState,
    ) -> KeyState {
        KeyState {
            camera_forward: self
                .camera_forward
                .map_or(false, |key| keyboard_state.is_scancode_pressed(key)),
            camera_backward: self
                .camera_backward
                .map_or(false, |key| keyboard_state.is_scancode_pressed(key)),
            camera_left: self
                .camera_left
                .map_or(false, |key| keyboard_state.is_scancode_pressed(key)),
            camera_right: self
                .camera_right
                .map_or(false, |key| keyboard_state.is_scancode_pressed(key)),
            camera_up: self
                .camera_up
                .map_or(false, |key| keyboard_state.is_scancode_pressed(key)),
            camera_down: self
                .camera_down
                .map_or(false, |key| keyboard_state.is_scancode_pressed(key)),
            camera_rotate_left: self
                .camera_rotate_left
                .map_or(false, |key| keyboard_state.is_scancode_pressed(key)),
            camera_rotate_right: self
                .camera_rotate_right
                .map_or(false, |key| keyboard_state.is_scancode_pressed(key)),
            camera_rotate_up: self
                .camera_rotate_up
                .map_or(false, |key| keyboard_state.is_scancode_pressed(key)),
            camera_rotate_down: self
                .camera_rotate_down
                .map_or(false, |key| keyboard_state.is_scancode_pressed(key)),
            right_button_pressed: mouse_state.is_mouse_button_pressed(MouseButton::Left),
            mouse_position: Vector2::new(
                (mouse_state.x() - (SCREEN_WIDTH / 2) as i32) as f32,
                (mouse_state.y() - (SCREEN_HEIGHT / 2) as i32) as f32,
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]

pub struct KeyState {
    pub camera_forward: bool,
    pub camera_backward: bool,
    pub camera_left: bool,
    pub camera_right: bool,
    pub camera_up: bool,
    pub camera_down: bool,
    pub camera_rotate_left: bool,
    pub camera_rotate_right: bool,
    pub camera_rotate_up: bool,
    pub camera_rotate_down: bool,
    pub right_button_pressed: bool,
    pub mouse_position: Vector2<f32>,
}

// Move and rotate camera
fn update(
    dt: f32,
    state: &KeyState,
    sdl_context: &Sdl,
    window: &Window,
    yaw: &mut f32,
    pitch: &mut f32,
    mouse_reference_position: &mut Option<Vector2<f32>>,
    camera_position: &mut Vector3<f32>,
    camera_rotation: &mut Rotation3<f32>,
) {
    let camera_speed = (dt / 1000.0) * CAMERA_MOVEMENT_SPEED;
    if state.camera_forward {
        // Move camera forwards
        *camera_position += (*camera_rotation * Vector3::x()) * camera_speed;
    }
    if state.camera_backward {
        // Move camera backward
        *camera_position += (*camera_rotation * -Vector3::x()) * camera_speed;
    }
    if state.camera_left {
        // Move camera left
        *camera_position += (*camera_rotation * -Vector3::y()) * camera_speed;
    }
    if state.camera_right {
        // Move camera right
        *camera_position += (*camera_rotation * Vector3::y()) * camera_speed;
    }
    if state.camera_up {
        // Move camera up
        *camera_position += (*camera_rotation * Vector3::z()) * camera_speed;
    }
    if state.camera_down {
        // Move camera down
        *camera_position += (*camera_rotation * -Vector3::z()) * camera_speed;
    }
    if state.camera_rotate_left {
        // Rotate camera left
        *yaw -= dt / 1000.0 * CAMERA_ROTATION_SPEED;
    }
    if state.camera_rotate_right {
        // Rotate camera right
        *yaw += dt / 1000.0 * CAMERA_ROTATION_SPEED;
    }
    if state.camera_rotate_up {
        // Rotate camera up
        *pitch -= dt / 1000.0 * CAMERA_ROTATION_SPEED;
    }
    if state.camera_rotate_down {
        // Rotate camera down
        *pitch += dt / 1000.0 * CAMERA_ROTATION_SPEED;
    }

    if let Some(mouse_ref) = mouse_reference_position.as_mut() {
        *yaw += (state.mouse_position.x - mouse_ref.x) * dt / 1000.0 * CAMERA_MOUSE_ROTATION_SPEED;
        *pitch +=
            (state.mouse_position.y - mouse_ref.y) * dt / 1000.0 * CAMERA_MOUSE_ROTATION_SPEED;
        *mouse_reference_position = None;
    }
    if state.right_button_pressed {
        *mouse_reference_position = Some(state.mouse_position);
    }

    *camera_rotation = Rotation::from_euler_angles(0.0, *pitch, *yaw);
}

fn draw(
    canvas: &mut Canvas<Window>,
    camera_position: &Vector3<f32>,
    camera_rotation: &Rotation3<f32>,
    light_pos: &Vector3<f32>,
) {
    canvas.clear();
    for y in 0..SCREEN_HEIGHT as i32 {
        for x in 0..SCREEN_WIDTH as i32 {
            let direction = camera_rotation
                * Vector3::new(
                    FOCAL_LENGTH as f32,
                    (x - (SCREEN_WIDTH as i32 / 2)) as f32,
                    (-y + (SCREEN_HEIGHT as i32 / 2)) as f32,
                );

            let mut color = Color::new(0.0, 0.0, 0.0);

            // Get color from triangle
            if let Some((intersection, triangle_index)) =
                closest_intersection(*camera_position, direction, TRIANGLES.iter())
            {
                let reflect_fraction = TRIANGLES[triangle_index].color;
                let light = direct_light(&intersection, &TRIANGLES[triangle_index], light_pos)
                    + INDRECT_LIGHT;
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

fn direct_light(i: &Intersection, triangle: &Triangle, light_pos: &Vector3<f32>) -> Vector3<f32> {
    let P = LIGHT_COLOR;

    let mut rvec = *light_pos - i.position;
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
