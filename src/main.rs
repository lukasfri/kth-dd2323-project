use kth_dd2323_project::{
    camera::Camera,
    controls::{ControlState, Keybinds},
    renderer::{Raytracer, Renderer},
    scene::Scene,
    wave_function_collapse::WFC,
    Color,
};
use nalgebra::{Vector2, Vector3};
use sdl2::{
    event::Event,
    keyboard::{KeyboardState, Keycode},
    mouse::MouseState,
    render::Canvas,
    video::Window,
    Sdl, TimerSubsystem,
};

fn main() -> anyhow::Result<()> {
    let sdl_context = sdl2::init().unwrap();

    const SCREEN_SIZE: Vector2<u32> = Vector2::new(500, 500);
    let canvas = setup_canvas(&sdl_context, SCREEN_SIZE);
    let timer = sdl_context.timer().unwrap();

    let focal_length: u32 = SCREEN_SIZE.y / 2;
    let camera = Camera::new(focal_length as f32, Vector3::new(-1.0, 0.0, 1.0));

    // let scene = Scene::load_cornell_box();
    let mut scene = Scene::new();
    const MAP_SIZE: usize = 10; // Width/height of map
    let mut wfc = WFC::new(&mut scene, MAP_SIZE);

    match wfc.place_tiles() {
        Ok(()) => program_loop(sdl_context, scene, canvas, camera, timer),
        Err(err) => Err(err),
    }
}

fn setup_canvas(sdl_context: &Sdl, screen_size: Vector2<u32>) -> Canvas<Window> {
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Wave Function Collapse", screen_size.x, screen_size.y)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::BLUE.to_sdl());
    canvas.clear();
    canvas.present();

    canvas
}

fn program_loop(
    sdl_context: Sdl,
    scene: Scene,
    mut canvas: Canvas<Window>,
    mut camera: Camera,
    timer: TimerSubsystem,
) -> anyhow::Result<()> {
    let mut mouse_reference_position: Option<Vector2<f32>> = None;
    let mut event_pump = sdl_context.event_pump().unwrap();
    let binds = Keybinds::default();
    let mut t = timer.ticks();

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

    Ok(())
}

fn update(
    dt: f32,
    state: &ControlState,
    mouse_reference_position: &mut Option<Vector2<f32>>,
    camera: &mut Camera,
) {
    move_camera(dt, state, mouse_reference_position, camera);
}

// Move and rotate camera
fn move_camera(
    dt: f32,
    state: &ControlState,
    mouse_reference_position: &mut Option<Vector2<f32>>,
    camera: &mut Camera,
) {
    const CAMERA_MOVEMENT_SPEED: f32 = 1.0;
    const CAMERA_ROTATION_SPEED: f32 = 1.0;
    const CAMERA_MOUSE_ROTATION_SPEED: f32 = 0.02;

    let camera_speed = (dt / 1000.0) * CAMERA_MOVEMENT_SPEED;
    let mut movement = Vector3::new(0.0, 0.0, 0.0);
    if state.camera_forward {
        movement += Vector3::x();
    }
    if state.camera_backward {
        movement -= Vector3::x();
    }
    if state.camera_left {
        movement -= Vector3::y();
    }
    if state.camera_right {
        movement += Vector3::y();
    }
    if state.camera_up {
        movement += Vector3::z();
    }
    if state.camera_down {
        movement -= Vector3::z();
    }
    camera.move_relative(movement * camera_speed);

    let camera_rotation_speed = (dt / 1000.0) * CAMERA_ROTATION_SPEED;
    let mut new_yaw = camera.yaw();
    let mut new_pitch = camera.pitch();
    if state.camera_rotate_left {
        new_yaw -= camera_rotation_speed;
    }
    if state.camera_rotate_right {
        new_yaw += camera_rotation_speed;
    }
    if state.camera_rotate_up {
        new_pitch -= camera_rotation_speed;
    }
    if state.camera_rotate_down {
        new_pitch += camera_rotation_speed;
    }

    if let Some(mouse_ref) = mouse_reference_position.as_mut() {
        let camera_mouse_rotation_speed = (dt / 1000.0) * CAMERA_MOUSE_ROTATION_SPEED;

        new_yaw += (state.mouse_position.x - mouse_ref.x) * camera_mouse_rotation_speed;
        new_pitch += (state.mouse_position.y - mouse_ref.y) * camera_mouse_rotation_speed;
        *mouse_reference_position = None;
    }
    if state.right_button_pressed {
        *mouse_reference_position = Some(state.mouse_position);
    }

    camera.update_rotation(new_pitch, new_yaw);
}
