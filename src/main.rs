use std::sync::Arc;

use between_wgpu::render::RenderState;

use thread_priority::set_current_thread_priority;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{DeviceEvent, DeviceId, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};

#[derive(Default)]
pub struct App {
    state: Option<RenderState>,
}

impl ApplicationHandler<RenderState> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attrs = Window::default_attributes()
            .with_title("Between")
            .with_inner_size(LogicalSize::new(1500.0, 900.0))
            .with_min_inner_size(LogicalSize::new(800.0, 300.0));
        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

        // cross platform window locking
        let _ = window
            .set_cursor_grab(winit::window::CursorGrabMode::Locked)
            .or_else(|_| window.set_cursor_grab(winit::window::CursorGrabMode::Confined));
        window.set_cursor_visible(false);

        let state = pollster::block_on(RenderState::new(window)).unwrap();
        self.state = Some(state);
    }

    // fn user_event(&mut self, event_loop: &ActiveEventLoop, event: State) {
    //     self.state = Some(event);
    // }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(state) = self.state.as_mut()
            && let DeviceEvent::MouseMotion { delta } = event
        {
            state.handle_mouse_motion(delta.0, delta.1);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(render_state) = self.state.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => render_state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                render_state.update();

                match render_state.render() {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("{e}");
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => render_state.handle_key(event_loop, code, key_state.is_pressed()),
            _ => {}
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    if set_current_thread_priority(thread_priority::ThreadPriority::Max).is_err() {
        println!("Error: failed to set current main priority");
    }

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}
