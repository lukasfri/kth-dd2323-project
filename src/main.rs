use kth_dd2323_project::controls::{Action, Controls};
use kth_dd2323_project::model_loader::ModelLoader;
use kth_dd2323_project::raster::{Rasterizer, RenderSurface, Texture, WgpuRenderProps};
use kth_dd2323_project::renderer::Renderer;
use kth_dd2323_project::{
    camera::Camera, controls::ControlState, scene::Scene, wave_function_collapse::WFC,
};
use nalgebra::{Vector2, Vector3};
use std::error::Error;
use std::fmt::Debug;
use std::mem;
use std::num::NonZeroU32;
use std::sync::Arc;

use tracing::{debug, error, info};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{DeviceEvent, DeviceId, Ime, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, ModifiersState};
use winit::window::{CursorIcon, ResizeDirection, Theme, Window, WindowId};

#[cfg(target_os = "linux")]
use winit::platform::startup_notify::{
    self, EventLoopExtStartupNotify, WindowAttributesExtStartupNotify,
};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::SubscriberBuilder::default().init();

    let event_loop = EventLoop::<ExternalEvent>::with_user_event().build()?;
    let _event_loop_proxy = event_loop.create_proxy();

    // Wire the user event from another thread.
    std::thread::spawn(move || {
        // Wake up the `event_loop` once every second and dispatch a custom event
        // from a different thread.
        debug!("Starting to send user event every second");
        loop {
            let _ = _event_loop_proxy.send_event(ExternalEvent::GameLoop);
            const GAME_LOOP_INTERVAL: std::time::Duration =
                std::time::Duration::from_millis(1000 / 60);
            std::thread::sleep(GAME_LOOP_INTERVAL);
        }
    });

    let instance = wgpu::Instance::default();

    let focal_length: u32 = 500 / 2;
    let camera = Camera::new(
        focal_length as f32,
        Vector3::new(-3.5f32, 2.0, 2.0),
        -Vector3::new(-3.5f32, 2.0, 2.0),
    );
    let scene = setup_scene()?;

    let mut state = pollster::block_on(Application::new(&instance, scene, camera));

    event_loop.run_app(&mut state)?;

    Ok(())
}

fn setup_scene() -> anyhow::Result<Scene> {
    // let scene = Scene::load_cornell_box();
    let mut scene = Scene::new();
    let mut wfc = WFC::new(&mut scene);

    wfc.place_tiles()?;

    scene.triangles.extend(ModelLoader::load_cornell_box());

    Ok(scene)
}

// fn program_loop(
//     sdl_context: Sdl,
//     scene: Scene,
//     mut canvas: Canvas<Window>,
//     mut camera: Camera,
//     timer: TimerSubsystem,
// ) -> anyhow::Result<()> {
//     let mut mouse_reference_position: Option<Vector2<f32>> = None;
//     let mut event_pump = sdl_context.event_pump().unwrap();
//     let binds = Keybinds::default();
//     let mut t = timer.ticks();

//     'running: loop {
//         for event in event_pump.poll_iter() {
//             match event {
//                 Event::Quit { .. }
//                 | Event::KeyDown {
//                     keycode: Some(Keycode::Escape),
//                     ..
//                 } => break 'running,
//                 _ => {}
//             }
//         }

//         let t2 = timer.ticks();
//         let dt = t2 - t;
//         println!("Render time: {}ms", dt);
//         t = t2;

//         let keyboard = KeyboardState::new(&event_pump);
//         let mouse = MouseState::new(&event_pump);
//         let state = binds.get_current_state(&keyboard, &mouse, canvas.window());
//         update(
//             dt as f32,
//             &state,
//             &mut mouse_reference_position,
//             &mut camera,
//         );

//         Raytracer.render(&mut canvas, &scene, &camera).unwrap();
//     }

//     Ok(())
// }

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
    const CAMERA_MOUSE_ROTATION_SPEED: f32 = 0.1;

    let camera_speed = dt * CAMERA_MOVEMENT_SPEED;
    let mut movement = Vector3::new(0.0, 0.0, 0.0);
    if state.camera_forward {
        movement += Vector3::z();
    }
    if state.camera_backward {
        movement -= Vector3::z();
    }
    if state.camera_left {
        movement += Vector3::x();
    }
    if state.camera_right {
        movement -= Vector3::x();
    }
    if state.camera_up {
        movement += Vector3::y();
    }
    if state.camera_down {
        movement -= Vector3::y();
    }
    movement *= camera_speed;

    camera.move_relative(movement);

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
        let camera_mouse_rotation_speed = dt * CAMERA_MOUSE_ROTATION_SPEED;

        new_yaw -= (state.mouse_position.x - mouse_ref.x) * camera_mouse_rotation_speed;
        new_pitch += (state.mouse_position.y - mouse_ref.y) * camera_mouse_rotation_speed;
        *mouse_reference_position = None;
    }
    if state.drag_mouse {
        *mouse_reference_position = Some(state.mouse_position);
    }

    camera.update_rotation(new_pitch, new_yaw);
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum ExternalEvent {
    GameLoop,
}

/// Application state and event handling.
struct Application<'window> {
    window: Option<WindowState<'window>>,

    instance: &'window wgpu::Instance,
    control_state: ControlState,
    controls: Controls,

    scene: Scene,
    camera: Camera,

    last_update: std::time::Instant,

    mouse_reference_position: Option<Vector2<f32>>,
}

impl<'window> Application<'window> {
    async fn new(instance: &'window wgpu::Instance, scene: Scene, camera: Camera) -> Self {
        Self {
            // context,
            window: None,
            instance,
            control_state: ControlState::default(),
            controls: Controls::default(),
            scene,
            camera,
            mouse_reference_position: None,
            last_update: std::time::Instant::now(),
        }
    }

    fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<WindowState<'window>, Box<dyn Error>> {
        // TODO read-out activation token.

        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes()
            .with_title("Winit window")
            .with_transparent(true);

        #[cfg(target_os = "linux")]
        if let Some(token) = event_loop.read_token_from_env() {
            startup_notify::reset_activation_token_env();
            debug!("Using token {:?} to activate a window", token);
            window_attributes = window_attributes.with_activation_token(token);
        }

        let window = event_loop.create_window(window_attributes)?;

        let window_state = pollster::block_on(WindowState::new(self, window))?;
        let window_id = window_state.window.id();
        debug!("Created new window with id={window_id:?}");
        Ok(window_state)
    }

    fn handle_action(&mut self, _event_loop: &ActiveEventLoop, action: Action) {
        // let cursor_position = self.cursor_position;
        let window = self.window.as_mut().unwrap();
        debug!("Executing action: {action:?}");
        match action {
            Action::CloseWindow => {
                self.window = None;
            }
            Action::ToggleDecorations => window.toggle_decorations(),
            Action::ToggleImeInput => window.toggle_ime(),
            Action::Minimize => window.minimize(),
            Action::DragWindow => window.drag_window(),
            Action::DragResizeWindow => window.drag_resize_window(),
            Action::PrintHelp => (), // self.print_help(),
            Action::RequestResize => window.swap_dimensions(),
            _ => (),
        }
    }

    fn dump_monitors(&self, event_loop: &ActiveEventLoop) {
        debug!("Monitors information");
        let primary_monitor = event_loop.primary_monitor();
        for monitor in event_loop.available_monitors() {
            let intro = if primary_monitor.as_ref() == Some(&monitor) {
                "Primary monitor"
            } else {
                "Monitor"
            };

            if let Some(name) = monitor.name() {
                debug!("{intro}: {name}");
            } else {
                debug!("{intro}: [no name]");
            }

            let PhysicalSize { width, height } = monitor.size();
            debug!(
                "  Current mode: {width}x{height}{}",
                if let Some(m_hz) = monitor.refresh_rate_millihertz() {
                    format!(" @ {}.{} Hz", m_hz / 1000, m_hz % 1000)
                } else {
                    String::new()
                }
            );

            let PhysicalPosition { x, y } = monitor.position();
            debug!("  Position: {x},{y}");

            debug!("  Scale factor: {}", monitor.scale_factor());

            debug!("  Available modes (width x height x bit-depth):");
            for mode in monitor.video_modes() {
                let PhysicalSize { width, height } = mode.size();
                let bits = mode.bit_depth();
                let m_hz = mode.refresh_rate_millihertz();
                debug!(
                    "    {width}x{height}x{bits} @ {}.{} Hz",
                    m_hz / 1000,
                    m_hz % 1000
                );
            }
        }
    }
}

impl ApplicationHandler<ExternalEvent> for Application<'_> {
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: ExternalEvent) {
        info!("User event: {event:?}");

        let dt_secs = self.last_update.elapsed().as_secs_f32();

        update(
            dt_secs,
            &self.control_state,
            &mut self.mouse_reference_position,
            &mut self.camera,
        );

        self.last_update = std::time::Instant::now();

        let Some(window) = self.window.as_mut() else {
            return;
        };

        window.window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_mut() else {
            return;
        };

        match event {
            WindowEvent::Resized(size) => {
                window.resize(size);
            }
            WindowEvent::Focused(focused) => {
                if focused {
                    debug!("Window={window_id:?} focused");
                } else {
                    debug!("Window={window_id:?} unfocused");
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                debug!("Window={window_id:?} changed scale to {scale_factor}");
            }
            WindowEvent::ThemeChanged(theme) => {
                debug!("Theme changed to {theme:?}");
                window.set_theme(theme);
            }
            WindowEvent::RedrawRequested => {
                update(
                    0.0,
                    &self.control_state,
                    &mut self.mouse_reference_position,
                    &mut self.camera,
                );

                if let Err(err) = window.draw(&self.scene, &self.camera) {
                    error!("Error drawing window: {err}");
                }
            }
            WindowEvent::Occluded(occluded) => {
                window.set_occluded(occluded);
            }
            WindowEvent::CloseRequested => {
                debug!("Closing Window={window_id:?}");
                self.window = None;
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                window.modifiers = modifiers.state();
                debug!("Modifiers changed to {:?}", window.modifiers);
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    debug!("Mouse wheel Line Delta: ({x},{y})");
                }
                MouseScrollDelta::PixelDelta(px) => {
                    debug!("Mouse wheel Pixel Delta: ({},{})", px.x, px.y);
                }
            },
            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } => {
                let mods = window.modifiers;

                let action = if let Key::Character(ch) = event.logical_key.as_ref() {
                    self.controls.process_key_binding(
                        &mut self.control_state,
                        &ch.to_uppercase(),
                        &mods,
                        event.state.is_pressed(),
                    )
                } else {
                    None
                };
                // Dispatch actions only on press.
                if event.state.is_pressed() {
                    if let Some(action) = action {
                        self.handle_action(event_loop, action);
                    }
                }
            }
            WindowEvent::MouseInput { button, state, .. } => {
                let mods = window.modifiers;

                let action = self.controls.process_mouse_binding(
                    &mut self.control_state,
                    button,
                    &mods,
                    state.is_pressed(),
                );

                // Dispatch actions only on press.
                if state.is_pressed() {
                    if let Some(action) = action {
                        self.handle_action(event_loop, action);
                    }
                }
            }
            WindowEvent::CursorLeft { .. } => {
                debug!("Cursor left Window={window_id:?}");
                window.cursor_left();
            }
            WindowEvent::CursorMoved { position, .. } => {
                debug!("Moved cursor to {position:?}");
                window.cursor_moved(position);

                self.control_state.mouse_position =
                    Vector2::new(position.x as f32, position.y as f32);
            }
            WindowEvent::ActivationTokenDone { token: _token, .. } => {
                #[cfg(target_os = "linux")]
                {
                    startup_notify::set_activation_token_env(_token);
                    if let Err(err) = self.create_window(event_loop) {
                        error!("Error creating new window: {err}");
                    }
                }
            }
            WindowEvent::Ime(event) => match event {
                Ime::Enabled => debug!("IME enabled for Window={window_id:?}"),
                Ime::Preedit(text, caret_pos) => {
                    debug!("Preedit: {}, with caret at {:?}", text, caret_pos);
                }
                Ime::Commit(text) => {
                    debug!("Committed: {}", text);
                }
                Ime::Disabled => debug!("IME disabled for Window={window_id:?}"),
            },
            WindowEvent::PinchGesture { delta, .. } => {
                window.zoom += delta;
                let zoom = window.zoom;
                if delta > 0.0 {
                    debug!("Zoomed in {delta:.5} (now: {zoom:.5})");
                } else {
                    debug!("Zoomed out {delta:.5} (now: {zoom:.5})");
                }
            }
            WindowEvent::RotationGesture { delta, .. } => {
                window.rotated += delta;
                let rotated = window.rotated;
                if delta > 0.0 {
                    debug!("Rotated counterclockwise {delta:.5} (now: {rotated:.5})");
                } else {
                    debug!("Rotated clockwise {delta:.5} (now: {rotated:.5})");
                }
            }
            WindowEvent::PanGesture { delta, phase, .. } => {
                window.panned.x += delta.x;
                window.panned.y += delta.y;
                debug!("Panned ({delta:?})) (now: {:?}), {phase:?}", window.panned);
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        debug!("Device {device_id:?} event: {event:?}");
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        info!("Resumed the event loop");
        self.dump_monitors(event_loop);

        // Create initial window.
        self.window = Some(
            self.create_window(event_loop)
                .expect("failed to create initial window"),
        );

        self.controls.print_help();
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            info!("No windows left, exiting...");
            event_loop.exit();
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        // We must drop the context here.
        // drop(self.context);
        //UNSAFE: WE SHOULD DROP THE CONTEXT HERE
    }
}

/// State of the window.
struct WindowState<'window> {
    /// IME input.
    pub ime: bool,
    /// Render surface.
    pub render_surface: RenderSurface<'window>,

    pub rasterizer: Rasterizer,

    /// The actual winit Window.
    pub window: Arc<Window>,
    /// The window theme we're drawing with.
    pub theme: Theme,
    /// Cursor position over the window.
    pub cursor_position: Option<PhysicalPosition<f64>>,
    /// Window modifiers state.
    pub modifiers: ModifiersState,
    /// Occlusion state of the window.
    pub occluded: bool,
    /// The amount of zoom into window.
    pub zoom: f64,
    /// The amount of rotation of the window.
    pub rotated: f32,
    /// The amount of pan of the window.
    pub panned: PhysicalPosition<f32>,
}

impl<'window> WindowState<'window> {
    async fn new(app: &Application<'window>, window: Window) -> Result<Self, Box<dyn Error>> {
        let window = Arc::new(window);

        // SAFETY: the surface is dropped before the `window` which provided it with handle, thus
        // it doesn't outlive it.
        // let surface = Surface::new(&app.context, Arc::clone(&window))?;

        // 1

        let instance = app.instance;

        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let mut config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();

        config.view_formats = vec![wgpu::TextureFormat::Bgra8UnormSrgb];

        surface.configure(&device, &config);

        // 2

        let theme = window.theme().unwrap_or(Theme::Dark);
        debug!("Theme: {theme:?}");
        window.set_cursor(CursorIcon::Default);

        // Allow IME out of the box.
        let ime = true;
        window.set_ime_allowed(ime);

        let props = WgpuRenderProps::init(&config, &adapter, &device, &queue);

        let rasterizer = Rasterizer::new(props);

        let size = window.inner_size();

        let render_surface = RenderSurface {
            surface,
            surface_config: config,
            device,
            queue,
        };

        let mut state = Self {
            render_surface,
            rasterizer,
            window,
            theme,
            ime,
            cursor_position: Default::default(),
            modifiers: Default::default(),
            occluded: Default::default(),
            rotated: Default::default(),
            panned: Default::default(),
            zoom: Default::default(),
        };

        state.resize(size);
        Ok(state)
    }

    pub fn toggle_ime(&mut self) {
        self.ime = !self.ime;
        self.window.set_ime_allowed(self.ime);
        if let Some(position) = self.ime.then_some(self.cursor_position).flatten() {
            self.window
                .set_ime_cursor_area(position, PhysicalSize::new(20, 20));
        }
    }

    pub fn minimize(&mut self) {
        self.window.set_minimized(true);
    }

    pub fn cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        self.cursor_position = Some(position);
        if self.ime {
            self.window
                .set_ime_cursor_area(position, PhysicalSize::new(20, 20));
        }
    }

    pub fn cursor_left(&mut self) {
        self.cursor_position = None;
    }

    /// Toggle window decorations.
    fn toggle_decorations(&self) {
        let decorated = self.window.is_decorated();
        self.window.set_decorations(!decorated);
    }

    /// Swap the window dimensions with `request_inner_size`.
    fn swap_dimensions(&mut self) {
        let old_inner_size = self.window.inner_size();
        let mut inner_size = old_inner_size;

        mem::swap(&mut inner_size.width, &mut inner_size.height);
        debug!("Requesting resize from {old_inner_size:?} to {inner_size:?}");

        if let Some(new_inner_size) = self.window.request_inner_size(inner_size) {
            if old_inner_size == new_inner_size {
                debug!("Inner size change got ignored");
            } else {
                self.resize(new_inner_size);
            }
        } else {
            debug!("Request inner size is asynchronous");
        }
    }

    /// Resize the window to the new size.
    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        info!("Resized to {new_size:?}");

        let (width, height) = match (
            NonZeroU32::new(new_size.width),
            NonZeroU32::new(new_size.height),
        ) {
            (Some(width), Some(height)) => (width, height),
            _ => return,
        };

        // let config = &mut self.render_surface.surface_config;
        // config.width = width.into();
        // config.height = height.into();
        // self.render_surface
        //     .surface
        //     .configure(&self.render_surface.device, config);
        self.render_surface.resize(new_size);

        self.rasterizer.props.depth_texture = Texture::create_depth_texture(
            &self.render_surface.device,
            &self.render_surface.surface_config,
            "depth_texture",
        );

        // let mx_total = WgpuRenderProps::generate_matrix(
        //     Into::<u32>::into(width) as f32 / Into::<u32>::into(height) as f32,

        // );
        // let mx_ref: &[f32; 16] = mx_total.as_ref();
        // self.render_surface.queue.write_buffer(
        //     &self.rasterizer.props.uniform_buf,
        //     0,
        //     bytemuck::cast_slice(mx_ref),
        // );

        self.window.request_redraw();
    }

    /// Change the theme.
    fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        self.window.request_redraw();
    }

    /// Drag the window.
    fn drag_window(&self) {
        if let Err(err) = self.window.drag_window() {
            debug!("Error starting window drag: {err}");
        } else {
            info!("Dragging window Window={:?}", self.window.id());
        }
    }

    /// Drag-resize the window.
    fn drag_resize_window(&self) {
        let position = match self.cursor_position {
            Some(position) => position,
            None => {
                debug!("Drag-resize requires cursor to be inside the window");
                return;
            }
        };

        /// The amount of points to around the window for drag resize direction calculations.
        const BORDER_SIZE: f64 = 20.;

        let win_size = self.window.inner_size();
        let border_size = BORDER_SIZE * self.window.scale_factor();

        let x_direction = if position.x < border_size {
            ResizeDirection::West
        } else if position.x > (win_size.width as f64 - border_size) {
            ResizeDirection::East
        } else {
            // Use arbitrary direction instead of None for simplicity.
            ResizeDirection::SouthEast
        };

        let y_direction = if position.y < border_size {
            ResizeDirection::North
        } else if position.y > (win_size.height as f64 - border_size) {
            ResizeDirection::South
        } else {
            // Use arbitrary direction instead of None for simplicity.
            ResizeDirection::SouthEast
        };

        let direction = match (x_direction, y_direction) {
            (ResizeDirection::West, ResizeDirection::North) => ResizeDirection::NorthWest,
            (ResizeDirection::West, ResizeDirection::South) => ResizeDirection::SouthWest,
            (ResizeDirection::West, _) => ResizeDirection::West,
            (ResizeDirection::East, ResizeDirection::North) => ResizeDirection::NorthEast,
            (ResizeDirection::East, ResizeDirection::South) => ResizeDirection::SouthEast,
            (ResizeDirection::East, _) => ResizeDirection::East,
            (_, ResizeDirection::South) => ResizeDirection::South,
            (_, ResizeDirection::North) => ResizeDirection::North,
            _ => return,
        };

        if let Err(err) = self.window.drag_resize_window(direction) {
            debug!("Error starting window drag-resize: {err}");
        } else {
            info!("Drag-resizing window Window={:?}", self.window.id());
        }
    }

    /// Change window occlusion state.
    fn set_occluded(&mut self, occluded: bool) {
        self.occluded = occluded;
        if !occluded {
            self.window.request_redraw();
        }
    }

    /// Draw the window contents.
    fn draw(&mut self, scene: &Scene, camera: &Camera) -> anyhow::Result<()> {
        if self.occluded {
            debug!("Skipping drawing occluded window={:?}", self.window.id());
            return Ok(());
        }

        self.rasterizer
            .render(&mut self.render_surface, scene, camera)?;

        Ok(())
    }
}
