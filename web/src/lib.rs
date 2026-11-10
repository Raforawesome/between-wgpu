//! Browser entry point for Between: the web counterpart of `src/main.rs`.
//!
//! The engine is the same; only the platform glue differs:
//! - WebGPU setup is async, so it's spawned and handed back as a user event
//!   instead of `pollster::block_on`.
//! - There are no threads, so each frame runs queued terrain jobs on the main
//!   thread for a fixed budget (`ChunkGenPipeline::run_next_task`).
//! - Mouse look uses pointer lock, which the browser only grants on a click.
//!   Where it's refused, dragging with the button held works instead.

use std::sync::Arc;

use between_wgpu::{render::RenderState, run_update_loop, world::WorldState};
use wasm_bindgen::prelude::*;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    keyboard::{KeyCode, PhysicalKey},
    platform::web::{EventLoopExtWebSys, WindowAttributesExtWebSys},
    window::{CursorGrabMode, Window, WindowId},
};

/// Main-thread time per frame spent generating and meshing chunks.
const TERRAIN_BUDGET_MS: f64 = 8.0;
/// Cap on a frame's simulated time, so returning to a backgrounded tab
/// doesn't fling the camera.
const MAX_DT_SECONDS: f32 = 0.1;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Info);

    let settings = Settings::from_url();
    log::info!("Between web: {settings:?}");

    let event_loop = EventLoop::<RenderState>::with_user_event()
        .build()
        .expect("event loop");
    let app = App {
        gpu_ready: event_loop.create_proxy(),
        window: None,
        render_state: None,
        game_state: WorldState::new(settings.seed, settings.view, settings.height),
        last_frame_ms: 0.0,
        frame_stats: FrameStats::default(),
        dragging: false,
        last_cursor: None,
    };
    event_loop.spawn_app(app);
}

/// World settings from the page URL: `?seed=42&view=10&height=4`.
/// The defaults are smaller than native because everything runs on one thread.
#[derive(Debug)]
struct Settings {
    seed: i32,
    view: i32,
    height: i32,
}

impl Settings {
    fn from_url() -> Self {
        let query = page::query_string();
        let get = |key: &str, default: i32, min: i32, max: i32| {
            query
                .get(key)
                .and_then(|v| v.parse::<i32>().ok())
                .map_or(default, |v| v.clamp(min, max))
        };
        Self {
            seed: get("seed", 42, i32::MIN, i32::MAX),
            view: get("view", 10, 1, 32),
            height: get("height", 4, 1, 12),
        }
    }
}

struct App {
    /// Sends the `RenderState` back into the event loop once WebGPU is set up.
    gpu_ready: EventLoopProxy<RenderState>,
    window: Option<Arc<Window>>,
    /// `None` until WebGPU setup finishes; nothing renders before that.
    render_state: Option<RenderState>,
    game_state: WorldState,
    last_frame_ms: f64,
    frame_stats: FrameStats,
    /// Drag-to-look fallback: true while the left button is held without pointer lock.
    dragging: bool,
    last_cursor: Option<PhysicalPosition<f64>>,
}

impl ApplicationHandler<RenderState> for App {
    /// Creates the window on the page's canvas and starts WebGPU setup.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title("Between")
            .with_canvas(page::canvas())
            .with_prevent_default(true);
        let window = match event_loop.create_window(attrs) {
            Ok(window) => Arc::new(window),
            Err(e) => return page::show_error(&format!("Couldn’t create a canvas: {e}")),
        };

        let gpu_ready = self.gpu_ready.clone();
        let for_gpu = window.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match RenderState::new(for_gpu).await {
                Ok(render_state) => {
                    let _ = gpu_ready.send_event(render_state);
                }
                Err(e) => page::show_error(&format!("Couldn’t start WebGPU: {e}")),
            }
        });
        self.window = Some(window);
    }

    /// WebGPU is ready: size the surface and start the frame loop.
    fn user_event(&mut self, _: &ActiveEventLoop, mut render_state: RenderState) {
        if let Some(window) = &self.window {
            let size = window.inner_size();
            render_state.resize(size.width, size.height);
            window.request_redraw();
        }
        self.render_state = Some(render_state);
        self.last_frame_ms = page::now_ms();
        page::show_ready();
    }

    /// Raw mouse movement. Only used for looking while the pointer is locked.
    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event
            && page::pointer_locked()
        {
            self.game_state.handle_mouse_motion(delta);
        }
    }

    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => self.frame(),
            WindowEvent::Resized(size) => {
                if let Some(render_state) = &mut self.render_state {
                    render_state.resize(size.width, size.height);
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => self.on_left_button(state),
            WindowEvent::CursorMoved { position, .. } => self.on_cursor_moved(position),
            WindowEvent::CursorLeft { .. } => self.dragging = false,
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        repeat,
                        ..
                    },
                ..
            } => self.on_key(code, state, repeat),
            _ => {}
        }
    }
}

impl App {
    /// One frame: terrain jobs, then the same update + render as native.
    fn frame(&mut self) {
        let (Some(window), Some(render_state)) = (&self.window, &mut self.render_state) else {
            return;
        };
        let now = page::now_ms();
        let dt_seconds = (((now - self.last_frame_ms) / 1000.0) as f32).min(MAX_DT_SECONDS);
        self.last_frame_ms = now;

        // Stand-in for the native worker threads: run queued terrain jobs
        // until the queue is empty or the budget is spent.
        let pipeline = &self.game_state.chunk_pipeline;
        let deadline = now + TERRAIN_BUDGET_MS;
        while page::now_ms() < deadline && pipeline.run_next_task() {}

        run_update_loop(&mut self.game_state, render_state, dt_seconds);

        // Once a second, like the native title bar.
        if let Some((p50, p95)) = self.frame_stats.record(now, dt_seconds * 1000.0) {
            let r = render_state.stats;
            let queued = self.game_state.chunk_pipeline.pending_tasks();
            page::show_stats(&format!(
                "frame p50 {p50:.1} / p95 {p95:.1} ms · draw {} / {} · culled {} · triangles {} · queued {queued}",
                r.drawn, r.resident, r.culled, r.triangles
            ));
        }

        window.request_redraw();
    }

    /// Pressing asks for pointer lock and also starts a drag, so looking works
    /// either way. Releasing ends the drag.
    fn on_left_button(&mut self, state: ElementState) {
        self.dragging = state.is_pressed();
        if let (true, Some(window)) = (state.is_pressed(), &self.window)
            && !page::pointer_locked()
        {
            let _ = window.set_cursor_grab(CursorGrabMode::Locked);
            window.focus_window();
        }
    }

    /// Drag-to-look. With pointer lock, `device_event` handles looking instead.
    fn on_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        if self.dragging
            && !page::pointer_locked()
            && let Some(last) = self.last_cursor
        {
            let delta = (position.x - last.x, position.y - last.y);
            self.game_state.handle_mouse_motion(delta);
        }
        self.last_cursor = Some(position);
    }

    /// Same order as native: movement keys first, then render toggles.
    /// Escape is left to the browser, which uses it to release pointer lock.
    fn on_key(&mut self, code: KeyCode, state: ElementState, repeat: bool) {
        if self.game_state.handle_input(code, state) {
            return;
        }
        if let Some(render_state) = &mut self.render_state {
            render_state.handle_input(code, state, repeat);
        }
    }
}

/// Collects frame intervals and reports p50/p95 once per second.
#[derive(Default)]
struct FrameStats {
    window_start_ms: f64,
    frame_times_ms: Vec<f32>,
}

impl FrameStats {
    fn record(&mut self, now_ms: f64, frame_ms: f32) -> Option<(f32, f32)> {
        if self.frame_times_ms.is_empty() {
            self.window_start_ms = now_ms;
        }
        self.frame_times_ms.push(frame_ms);
        if now_ms - self.window_start_ms < 1000.0 {
            return None;
        }
        let times = &mut self.frame_times_ms;
        times.sort_unstable_by(f32::total_cmp);
        let n = times.len();
        let percentiles = (times[n / 2], times[((n - 1) * 95) / 100]);
        times.clear();
        Some(percentiles)
    }
}

/// Everything that touches the page outside the canvas. Element ids and the
/// `data-state` values match `www/index.html`.
mod page {
    use wasm_bindgen::JsCast;

    fn window() -> web_sys::Window {
        web_sys::window().expect("no window")
    }

    fn document() -> web_sys::Document {
        window().document().expect("no document")
    }

    fn set_text(id: &str, text: &str) {
        if let Some(el) = document().get_element_by_id(id) {
            el.set_text_content(Some(text));
        }
    }

    fn set_state(state: &str) {
        if let Some(body) = document().body() {
            let _ = body.set_attribute("data-state", state);
        }
    }

    pub fn canvas() -> Option<web_sys::HtmlCanvasElement> {
        document().get_element_by_id("between")?.dyn_into().ok()
    }

    pub fn query_string() -> web_sys::UrlSearchParams {
        let search = window().location().search().unwrap_or_default();
        web_sys::UrlSearchParams::new_with_str(&search).expect("query string")
    }

    pub fn now_ms() -> f64 {
        window().performance().map_or(0.0, |p| p.now())
    }

    pub fn pointer_locked() -> bool {
        document().pointer_lock_element().is_some()
    }

    /// Hides the loading message and shows the controls hint and stats bar.
    pub fn show_ready() {
        set_state("ready");
    }

    pub fn show_error(message: &str) {
        log::error!("{message}");
        set_text("status", message);
        set_state("error");
    }

    pub fn show_stats(text: &str) {
        set_text("stats", text);
    }
}
