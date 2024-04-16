use nalgebra::Vector2;
use sdl2::{
    keyboard::{KeyboardState, Scancode},
    mouse::{MouseButton, MouseState},
    video::Window,
};

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
    pub fn get_current_state(
        &self,
        keyboard_state: &KeyboardState<'_>,
        mouse_state: &MouseState,
        window: &Window,
    ) -> ControlState {
        let (width, height) = window.drawable_size();
        ControlState {
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
                (mouse_state.x() - (width / 2) as i32) as f32,
                (mouse_state.y() - (height / 2) as i32) as f32,
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]

pub struct ControlState {
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
