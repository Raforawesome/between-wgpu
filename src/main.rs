use std::sync::Arc;

use between_wgpu::gpu::RenderState;

use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
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
            .with_inner_size(winit::dpi::LogicalSize::new(1500.0, 900.0))
            .with_min_inner_size(winit::dpi::LogicalSize::new(800.0, 300.0));
        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());
        let state = pollster::block_on(RenderState::new(window)).unwrap();
        self.state = Some(state);
    }

    // fn user_event(&mut self, event_loop: &ActiveEventLoop, event: State) {
    //     self.state = Some(event);
    // }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = self.state.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                state.update();

                match state.render() {
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
            } => state.handle_key(event_loop, code, key_state.is_pressed()),
            _ => {}
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}
