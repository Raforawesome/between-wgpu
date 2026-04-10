use std::sync::Arc;

use between_wgpu::{GameState, render::RenderState, run_update_loop};

use thread_priority::set_current_thread_priority;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{DeviceEvent, DeviceId, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window, WindowId},
};

#[derive(Default)]
/// Main app container for the winit event loop.
pub struct App {
    render_state: Option<RenderState>,
    game_state: GameState,
}

impl ApplicationHandler<RenderState> for App {
    /// Called when the application is resumed and the window is created.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attrs = Window::default_attributes()
            .with_title("Between")
            .with_inner_size(LogicalSize::new(1500.0, 900.0))
            .with_min_inner_size(LogicalSize::new(800.0, 300.0));
        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

        // cross platform window locking
        let _ = window
            .set_cursor_grab(CursorGrabMode::Locked)
            .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
        window.set_cursor_visible(false);

        let state = pollster::block_on(RenderState::new(window)).unwrap();
        self.render_state = Some(state);
    }

    /// Handles raw device events such as mouse motion for camera look.
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.game_state.handle_mouse_motion(delta);
        }
    }

    /// Handles window events such as resizing, keyboard input, and close requests.
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(render_state) = self.render_state.as_mut() else {
            return;
        };
        let game_state = &mut self.game_state;

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => render_state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => match run_update_loop(game_state, render_state) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("{e}");
                    event_loop.exit();
                }
            },
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                // let game state handle game-related inputs first
                if game_state.handle_input(code, key_state) {
                    return;
                }

                if let (KeyCode::Escape, true) = (code, key_state.is_pressed()) {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
}

/// Entry point for the application. Initializes logging, sets thread priority,
/// and runs the winit event loop.
fn main() -> anyhow::Result<()> {
    env_logger::init();

    if let Err(e) = set_current_thread_priority(thread_priority::ThreadPriority::Max) {
        println!("Error: failed to set current main priority.\n{e}");
    }

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}
