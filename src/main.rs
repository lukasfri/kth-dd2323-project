use kth_dd2323_project::{
    camera::Camera,
    controls::{ControlState, Keybinds},
    renderer::{Raytracer, Renderer},
    scene::Scene,
    Color,
};
use nalgebra::{Vector2, Vector3};
use sdl2::{
    event::Event,
    keyboard::{KeyboardState, Keycode},
    mouse::MouseState,
};

const SCREEN_WIDTH: u32 = 500;
const SCREEN_HEIGHT: u32 = 500;
const CAMERA_MOVEMENT_SPEED: f32 = 1.0;
const CAMERA_ROTATION_SPEED: f32 = 1.0;
const CAMERA_MOUSE_ROTATION_SPEED: f32 = 0.02;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Wave Function Collapse", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::BLUE.to_sdl());
    canvas.clear();

    canvas.present();

    let timer = sdl_context.timer().unwrap();
    let mut t = timer.ticks();

    const FOCAL_LENGTH: u32 = SCREEN_HEIGHT / 2;
    let mut camera = Camera::new(FOCAL_LENGTH as f32, Vector3::new(-4.0, 0.0, 0.0));

    let scene = Scene::load_cornell_box();

    let mut mouse_reference_position: Option<Vector2<f32>> = None;

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
            dt, camera.position, camera.rotation
        );
        t = t2;

        let keyboard = KeyboardState::new(&event_pump);
        let mouse = MouseState::new(&event_pump);
        let state = binds.get_current_state(&keyboard, &mouse, canvas.window());
        update(
            dt as f32,
            &state,
            &mut mouse_reference_position,
            &mut camera,
        );

        Raytracer.render(&mut canvas, &scene, &camera).unwrap();
    }
}

// Move and rotate camera
fn update(
    dt: f32,
    state: &ControlState,
    mouse_reference_position: &mut Option<Vector2<f32>>,
    camera: &mut Camera,
) {
    let camera_speed = (dt / 1000.0) * CAMERA_MOVEMENT_SPEED;
    if state.camera_forward {
        // Move camera forwards
        camera.position += (camera.rotation * Vector3::x()) * camera_speed;
    }
    if state.camera_backward {
        // Move camera backward
        camera.position += (camera.rotation * -Vector3::x()) * camera_speed;
    }
    if state.camera_left {
        // Move camera left
        camera.position += (camera.rotation * -Vector3::y()) * camera_speed;
    }
    if state.camera_right {
        // Move camera right
        camera.position += (camera.rotation * Vector3::y()) * camera_speed;
    }
    if state.camera_up {
        // Move camera up
        camera.position += (camera.rotation * Vector3::z()) * camera_speed;
    }
    if state.camera_down {
        // Move camera down
        camera.position += (camera.rotation * -Vector3::z()) * camera_speed;
    }
    if state.camera_rotate_left {
        // Rotate camera left
        camera.yaw -= dt / 1000.0 * CAMERA_ROTATION_SPEED;
    }
    if state.camera_rotate_right {
        // Rotate camera right
        camera.yaw += dt / 1000.0 * CAMERA_ROTATION_SPEED;
    }
    if state.camera_rotate_up {
        // Rotate camera up
        camera.pitch -= dt / 1000.0 * CAMERA_ROTATION_SPEED;
    }
    if state.camera_rotate_down {
        // Rotate camera down
        camera.pitch += dt / 1000.0 * CAMERA_ROTATION_SPEED;
    }

    if let Some(mouse_ref) = mouse_reference_position.as_mut() {
        camera.yaw +=
            (state.mouse_position.x - mouse_ref.x) * dt / 1000.0 * CAMERA_MOUSE_ROTATION_SPEED;
        camera.pitch +=
            (state.mouse_position.y - mouse_ref.y) * dt / 1000.0 * CAMERA_MOUSE_ROTATION_SPEED;
        *mouse_reference_position = None;
    }
    if state.right_button_pressed {
        *mouse_reference_position = Some(state.mouse_position);
    }

    camera.update_rotation();
}
