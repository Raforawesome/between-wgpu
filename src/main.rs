use std::{sync::Arc, time::Instant};

use between_wgpu::{render::RenderState, run_update_loop, world::WorldState};

use thread_priority::set_current_thread_priority;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{DeviceEvent, DeviceId, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window, WindowId},
};

/// Main app container for the winit event loop.
pub struct App {
    render_state: Option<RenderState>,
    game_state: WorldState,
    last_frame: Instant,
    stats_start: Instant,
    frame_times: Vec<f32>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            render_state: None,
            game_state: WorldState::default(),
            last_frame: Instant::now(),
            stats_start: Instant::now(),
            frame_times: Vec::with_capacity(240),
        }
    }
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
        self.last_frame = Instant::now();
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
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt_seconds = now.duration_since(self.last_frame).as_secs_f32();
                self.last_frame = now;

                run_update_loop(game_state, render_state, dt_seconds);
                self.frame_times.push(dt_seconds * 1000.0);
                if self.stats_start.elapsed().as_secs_f32() >= 1.0 {
                    self.frame_times.sort_unstable_by(f32::total_cmp);
                    let n = self.frame_times.len();
                    let p50 = self.frame_times[n / 2];
                    let p95 = self.frame_times[((n - 1) * 95) / 100];
                    let r = render_state.stats;
                    let s = game_state.streaming.stats;
                    render_state.gpu.window.set_title(&format!(
                        "Between | frame p50 {p50:.1} / p95 {p95:.1} ms | draw {} / {} | culled {} | triangles {} | CPU {} | jobs {}",
                        r.drawn, r.resident, r.culled, r.triangles, s.resident, s.in_flight));
                    self.frame_times.clear();
                    self.stats_start = Instant::now();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        repeat,
                        ..
                    },
                ..
            } => {
                // let game state handle game-related inputs first
                if game_state.handle_input(code, key_state) {
                    return;
                }

                // otherwise let render state handle the input
                if render_state.handle_input(code, key_state, repeat) {
                    return;
                }

                // fallback input handling if none of the other handlers triggered
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

    let Some(config) = between_wgpu::config::Config::parse(std::env::args().skip(1))? else {
        println!(
            "Between: cargo run --release -- [--seed I32] [--view-distance 1..32] [--vertical-distance 1..16]\nDistances are inclusive chunk radii around the camera.\nWASD: fly, mouse: look, Space/E: up, Ctrl/Q: down, Shift: sprint, P: wireframe, C: culling, Esc: quit"
        );
        return Ok(());
    };
    log::info!("run settings: {config:?}");
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}
