use std::fmt::{self, Debug};

use nalgebra::Vector2;
use tracing::info;
use winit::{event::MouseButton, keyboard::ModifiersState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    CameraMoveForward,
    CameraMoveBackward,
    CameraMoveLeft,
    CameraMoveRight,
    CameraMoveUp,
    CameraMoveDown,
    CameraRotateLeft,
    CameraRotateRight,
    CameraRotateUp,
    CameraRotateDown,
    DragMouse,

    //Window
    CloseWindow,
    ToggleImeInput,
    ToggleDecorations,
    Minimize,
    PrintHelp,
    DragWindow,
    DragResizeWindow,
    RequestResize,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

#[derive(Debug, Clone)]
pub struct Binding<T> {
    pub trigger: T,
    pub mods: ModifiersState,
    pub action: Action,
}

impl<T: PartialEq> Binding<T> {
    const fn new(trigger: T, mods: ModifiersState, action: Action) -> Self {
        Self {
            trigger,
            mods,
            action,
        }
    }

    fn is_triggered_by(&self, trigger: &T, mods: &ModifiersState) -> bool {
        &self.trigger == trigger && &self.mods == mods
    }
}

#[derive(Debug, Clone)]
pub struct Controls {
    mouse_bindings: Vec<Binding<MouseButton>>,
    key_bindings: Vec<Binding<&'static str>>,
}

impl Default for Controls {
    fn default() -> Self {
        let mouse_bindings = vec![Binding::new(
            MouseButton::Left,
            ModifiersState::empty(),
            Action::DragMouse,
        )];

        let key_bindings = vec![
            Binding::new("W", ModifiersState::empty(), Action::CameraMoveForward),
            Binding::new("S", ModifiersState::empty(), Action::CameraMoveBackward),
            Binding::new("A", ModifiersState::empty(), Action::CameraMoveLeft),
            Binding::new("D", ModifiersState::empty(), Action::CameraMoveRight),
            Binding::new("Q", ModifiersState::empty(), Action::CameraMoveDown),
            Binding::new("E", ModifiersState::empty(), Action::CameraMoveUp),
            Binding::new("LEFT", ModifiersState::empty(), Action::CameraRotateLeft),
            Binding::new("RIGHT", ModifiersState::empty(), Action::CameraRotateRight),
            Binding::new("UP", ModifiersState::empty(), Action::CameraRotateUp),
            Binding::new("DOWN", ModifiersState::empty(), Action::CameraRotateDown),
        ];

        Self {
            mouse_bindings,
            key_bindings,
        }
    }
}

impl Controls {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_mouse_binding(&mut self, key: MouseButton, action: Action, mods: ModifiersState) {
        self.mouse_bindings.push(Binding {
            trigger: key,
            action,
            mods,
        });
    }

    pub fn add_key_binding(&mut self, key: &'static str, action: Action, mods: ModifiersState) {
        self.key_bindings.push(Binding {
            trigger: key,
            action,
            mods,
        });
    }

    /// Process the key binding.
    pub fn parse_key_binding(&self, key: &str, mods: &ModifiersState) -> Option<Action> {
        self.key_bindings.iter().find_map(|binding| {
            binding
                .is_triggered_by(&key, mods)
                .then_some(binding.action)
        })
    }

    pub fn process_key_binding(
        &self,
        control_state: &mut ControlState,
        key: &str,
        mods: &ModifiersState,
        is_pressed: bool,
    ) -> Option<Action> {
        self.parse_key_binding(key, mods).map(|action| {
            control_state.action_updated(&action, is_pressed);
            action
        })
    }

    /// Process mouse binding.
    pub fn parse_mouse_binding(
        &self,
        button: MouseButton,
        mods: &ModifiersState,
    ) -> Option<Action> {
        self.mouse_bindings.iter().find_map(|binding| {
            binding
                .is_triggered_by(&button, mods)
                .then_some(binding.action)
        })
    }

    pub fn process_mouse_binding(
        &self,
        control_state: &mut ControlState,
        button: MouseButton,
        mods: &ModifiersState,
        is_pressed: bool,
    ) -> Option<Action> {
        self.parse_mouse_binding(button, mods).map(|action| {
            control_state.action_updated(&action, is_pressed);
            action
        })
    }

    pub fn print_help(&self) {
        info!("Keyboard bindings:");
        for binding in self.key_bindings.iter() {
            info!(
                "{}{:<10} - {}",
                modifiers_to_string(binding.mods),
                binding.trigger,
                binding.action,
            );
        }
        info!("Mouse bindings:");
        for binding in self.mouse_bindings.iter() {
            info!(
                "{}{:?} - {}",
                modifiers_to_string(binding.mods),
                binding.trigger,
                binding.action,
            );
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]

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
    pub drag_mouse: bool,
    pub mouse_position: Vector2<f32>,
}

impl ControlState {
    pub fn action_updated(&mut self, action: &Action, active: bool) {
        match action {
            Action::CameraMoveForward => {
                self.camera_forward = active;
            }
            Action::CameraMoveBackward => {
                self.camera_backward = active;
            }
            Action::CameraMoveLeft => {
                self.camera_left = active;
            }
            Action::CameraMoveRight => {
                self.camera_right = active;
            }
            Action::CameraMoveUp => {
                self.camera_up = active;
            }
            Action::CameraMoveDown => {
                self.camera_down = active;
            }
            Action::CameraRotateLeft => {
                self.camera_rotate_left = active;
            }
            Action::CameraRotateRight => {
                self.camera_rotate_right = active;
            }
            Action::CameraRotateUp => {
                self.camera_rotate_up = active;
            }
            Action::CameraRotateDown => {
                self.camera_rotate_down = active;
            }
            Action::DragMouse => {
                self.drag_mouse = active;
            }
            _ => {}
        }
    }
}

fn modifiers_to_string(mods: ModifiersState) -> String {
    let mut mods_line = String::new();
    // Always add + since it's printed as a part of the bindings.
    for (modifier, desc) in [
        (ModifiersState::SUPER, "Super+"),
        (ModifiersState::ALT, "Alt+"),
        (ModifiersState::CONTROL, "Ctrl+"),
        (ModifiersState::SHIFT, "Shift+"),
    ] {
        if !mods.contains(modifier) {
            continue;
        }

        mods_line.push_str(desc);
    }
    mods_line
}
