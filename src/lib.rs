use std::collections::HashMap;

pub use bevy;
pub mod prelude {
    pub use crate::{getters::*, Config, EventHandler};
    pub use bevy::{input::ElementState, math::vec2, prelude::*};
}
mod render;

use bevy::{
    input::{
        keyboard::KeyboardInput,
        mouse::{MouseButtonInput, MouseMotion},
    },
    window::{WindowCloseRequested, WindowResized},
};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use prelude::*;

pub trait EventHandler: Send + Sync + 'static {
    fn config() -> Config {
        Config {
            window: WindowDescriptor::default(),
        }
    }
    fn init_app(_app: &mut App) {}
    fn update(&mut self, dt: f32);
    fn draw(&mut self) {}
    fn keyboard(&mut self, _key: KeyCode, _scan_code: u32, _state: ElementState, _repeat: bool) {}
    fn mouse_button(&mut self, _button: MouseButton, _state: ElementState) {}
    fn mouse_relative(&mut self, _delta: Vec2) {}
    fn mouse_absolute(&mut self, _pos: Vec2) {}
    fn window_resized(&mut self, _new_size: Vec2) {}
    fn close_requested(&mut self) -> bool {
        true
    }
}

struct Context {
    window_size: Vec2,
    mouse_position: Vec2,
    keys: HashMap<KeyCode, ButtonState>,
    mouse_buttons: HashMap<MouseButton, ButtonState>,
}

pub fn run<T>(state: T)
where
    T: EventHandler,
{
    let config = T::config();
    *CONTEXT.write() = Some(Context {
        window_size: vec2(config.window.width, config.window.height),
        mouse_position: Vec2::ZERO,
        keys: HashMap::new(),
        mouse_buttons: HashMap::new(),
    });

    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .insert_resource(state)
    .add_system(update_keys::<T>)
    .add_system(move |time: Res<Time>, mut state: ResMut<T>| {
        T::update(&mut *state, time.delta_seconds());
    });

    T::init_app(&mut app);

    app.run();

    *CONTEXT.write() = None;
}

pub struct Config {
    window: WindowDescriptor,
}

#[derive(Default)]
struct ButtonState {
    down: bool,
    pressed: bool,
    released: bool,
}

static CONTEXT: Lazy<RwLock<Option<Context>>> = Lazy::new(Default::default);

fn ctx<F, T>(mut f: F) -> T
where
    F: FnMut(&Context) -> T,
{
    f(CONTEXT.read().as_ref().expect("bevl was not initialized"))
}

fn ctx_mut<F, T>(mut f: F) -> T
where
    F: FnMut(&mut Context) -> T,
{
    f(CONTEXT.write().as_mut().expect("bevl was not initialized"))
}

mod getters {
    use super::*;
    pub fn window_size() -> Vec2 {
        ctx(|ctx| ctx.window_size)
    }

    pub fn mouse_position() -> Vec2 {
        ctx(|ctx| ctx.mouse_position)
    }

    pub fn is_key_down(key: KeyCode) -> bool {
        ctx(|ctx| ctx.keys.get(&key).map_or(false, |s| s.down))
    }
    pub fn is_key_pressed(key: KeyCode) -> bool {
        ctx(|ctx| ctx.keys.get(&key).map_or(false, |s| s.pressed))
    }
    pub fn is_key_released(key: KeyCode) -> bool {
        ctx(|ctx| ctx.keys.get(&key).map_or(false, |s| s.released))
    }

    pub fn is_mouse_button_down(mb: MouseButton) -> bool {
        ctx(|ctx| ctx.mouse_buttons.get(&mb).map_or(false, |s| s.down))
    }
    pub fn is_mouse_button_pressed(mb: MouseButton) -> bool {
        ctx(|ctx| ctx.mouse_buttons.get(&mb).map_or(false, |s| s.pressed))
    }
    pub fn is_mouse_button_released(mb: MouseButton) -> bool {
        ctx(|ctx| ctx.mouse_buttons.get(&mb).map_or(false, |s| s.released))
    }
}
pub use getters::*;

fn update_keys<T: EventHandler>(
    mut state: ResMut<T>,
    mut keyboard: EventReader<KeyboardInput>,
    mut mouse_buttons: EventReader<MouseButtonInput>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut cursor_motion: EventReader<CursorMoved>,
    mut resize: EventReader<WindowResized>,
    close: EventReader<WindowCloseRequested>,
) {
    // Keyboard
    if !keyboard.is_empty() {
        for input in keyboard.iter() {
            if let Some(key) = input.key_code {
                let mut repeat = false;
                ctx_mut(|ctx| {
                    let key_state = ctx.keys.entry(key).or_default();
                    key_state.pressed = false;
                    key_state.released = false;
                    match input.state {
                        ElementState::Pressed => {
                            repeat = key_state.down;
                            key_state.down = true;
                            key_state.pressed = true;
                        }
                        ElementState::Released => {
                            key_state.down = false;
                            key_state.released = true
                        }
                    }
                });
                state.keyboard(key, input.scan_code, input.state, repeat);
            }
        }
    }

    // Mouse buttons
    if !mouse_buttons.is_empty() {
        for input in mouse_buttons.iter() {
            ctx_mut(|ctx| {
                let button_state = ctx.mouse_buttons.entry(input.button).or_default();
                button_state.pressed = false;
                button_state.released = false;
                match input.state {
                    ElementState::Pressed => {
                        button_state.down = true;
                        button_state.pressed = true;
                    }
                    ElementState::Released => {
                        button_state.down = false;
                        button_state.released = true
                    }
                }
            });
            state.mouse_button(input.button, input.state);
        }
    }

    // Mouse move
    if !mouse_motion.is_empty() {
        for motion in mouse_motion.iter() {
            state.mouse_relative(motion.delta);
        }
    }

    // Cursor move
    if !cursor_motion.is_empty() {
        for moved in cursor_motion.iter() {
            ctx_mut(|ctx| ctx.mouse_position = moved.position);
            state.mouse_absolute(moved.position);
        }
    }

    // Resize window
    if !resize.is_empty() {
        for resize in resize.iter() {
            let size = vec2(resize.width, resize.height);
            ctx_mut(|ctx| ctx.window_size = size);
            state.window_resized(size);
        }
    }

    // Close
    if !close.is_empty() && state.close_requested() {}
}
