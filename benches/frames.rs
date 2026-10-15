//! Frame-time benchmark: drives the real `WorldState` + `RenderState` through a scripted,
//! frame-indexed camera route and records frame intervals.
//!
//! Unlike `engine.rs` this needs a display and a GPU: it opens a window (as the app does),
//! so keep it visible and unobscured while it runs. Present mode is switched to `Immediate`
//! so vsync does not cap the numbers, except in the one phase that deliberately measures it.
//!
//! ```sh
//! cargo bench --locked --bench frames [-- --view-distance N --vertical-distance N --seed N]
//! ```
//!
//! Results are printed as Markdown and written to `docs/benchmarks/frames.md`.
//!
//! A frame interval is wall time between consecutive redraws. It covers world update
//! (channel draining, buffer uploads), culling, command encoding, submission and presentation
//! back-pressure. It is not isolated GPU time.

use std::{fmt::Write as _, sync::Arc, time::Instant};

use between_wgpu::{config::Config, render::RenderState, run_update_loop, world::WorldState};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::KeyCode,
    window::{Window, WindowId},
};

const SPAWN: glam::Vec3 = glam::Vec3::new(0.0, 90.0, -2.0);
const PITCH: f32 = -10.0;
const WARMUP_FRAMES: usize = 30;

#[derive(Clone, Copy, PartialEq)]
enum Route {
    /// Stand at spawn until the resident chunk count stops changing.
    Load,
    /// Stand at spawn and turn 0.5° per frame (a full turn every 720 frames).
    Spin,
    /// Fly forward along +Z at 0.5 blocks per frame (≈ the 100 blocks/s base speed at 200 fps).
    Flight,
}

struct Phase {
    name: &'static str,
    route: Route,
    frames: usize,
    culling: bool,
    vsync: bool,
}

#[derive(Default)]
struct PhaseResult {
    name: &'static str,
    frame_ms: Vec<f64>,
    total_s: f64,
    drawn: f64,
    resident: usize,
    triangles: f64,
    cpu_chunks: usize,
}

struct App {
    render_state: Option<RenderState>,
    world: WorldState,
    phases: Vec<Phase>,
    phase: usize,
    frame: usize,
    started: bool,
    last: Instant,
    cur: PhaseResult,
    phase_start: Instant,
    culling_on: bool,
    stable: usize,
    last_resident: usize,
    last_change: Instant,
    drawn_sum: f64,
    tri_sum: f64,
    counted: usize,
    results: Vec<PhaseResult>,
    window_px: (u32, u32),
}

fn set_present(rs: &mut RenderState, mode: wgpu::PresentMode) {
    rs.gpu.config.present_mode = mode;
    rs.gpu.surface.configure(&rs.gpu.device, &rs.gpu.config);
}

impl App {
    fn begin_phase(&mut self) {
        let spec = &self.phases[self.phase];
        let rs = self.render_state.as_mut().unwrap();
        if spec.culling != self.culling_on {
            rs.handle_input(KeyCode::KeyC, ElementState::Pressed, false);
            self.culling_on = spec.culling;
        }
        set_present(
            rs,
            if spec.vsync {
                wgpu::PresentMode::AutoVsync
            } else {
                wgpu::PresentMode::Immediate
            },
        );
        self.cur = PhaseResult {
            name: spec.name,
            ..Default::default()
        };
        self.frame = 0;
        self.stable = 0;
        self.drawn_sum = 0.0;
        self.tri_sum = 0.0;
        self.counted = 0;
        self.phase_start = Instant::now();
        self.last = Instant::now();
    }

    fn finish_phase(&mut self) {
        let rs = self.render_state.as_ref().unwrap();
        let mut r = std::mem::take(&mut self.cur);
        r.total_s = self
            .last_change
            .duration_since(self.phase_start)
            .as_secs_f64();
        let n = self.counted.max(1) as f64;
        r.drawn = self.drawn_sum / n;
        r.triangles = self.tri_sum / n;
        r.resident = rs.stats.resident;
        r.cpu_chunks = self.world.chunk_man.len();
        self.results.push(r);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // same window attributes as the app
        let attrs = Window::default_attributes()
            .with_title("Between frame benchmark (leave visible)")
            .with_inner_size(LogicalSize::new(1500.0, 900.0));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        let rs = pollster::block_on(RenderState::new(window)).unwrap();
        self.window_px = (rs.gpu.config.width, rs.gpu.config.height);
        self.render_state = Some(rs);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(rs) = self.render_state.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => rs.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                // a redraw can still arrive after the final `exit()`
                if self.phase >= self.phases.len() {
                    return;
                }
                if !self.started {
                    self.started = true;
                    self.begin_phase();
                }
                let now = Instant::now();
                let dt = now.duration_since(self.last).as_secs_f64();
                self.last = now;

                let spec = &self.phases[self.phase];
                let f = self.frame as f32;
                let player = &mut self.world.player;
                match spec.route {
                    Route::Load => {
                        player.eye = SPAWN;
                        player.yaw = 0.0;
                    }
                    Route::Spin => {
                        player.eye = SPAWN;
                        player.yaw = f * 0.5;
                    }
                    Route::Flight => {
                        player.eye = SPAWN + glam::Vec3::new(0.0, 0.0, f * 0.5);
                        player.yaw = 0.0;
                    }
                }
                player.pitch = PITCH;

                let rs = self.render_state.as_mut().unwrap();
                run_update_loop(&mut self.world, rs, dt as f32);

                // the first frames of a phase include pipeline/surface reconfiguration
                // startup frames before the surface is ready return immediately; they are not frames
                let presented = dt >= 0.0005;
                if (spec.route == Route::Load && presented) || self.frame >= WARMUP_FRAMES {
                    self.cur.frame_ms.push(dt * 1000.0);
                    self.drawn_sum += rs.stats.drawn as f64;
                    self.tri_sum += rs.stats.triangles as f64;
                    self.counted += 1;
                }
                self.frame += 1;

                let done = match spec.route {
                    Route::Load => {
                        if rs.stats.resident == self.last_resident {
                            self.stable += 1;
                        } else {
                            self.stable = 0;
                            self.last_resident = rs.stats.resident;
                            self.last_change = Instant::now();
                        }
                        (self.stable >= 120 && self.frame >= 200) || self.frame >= spec.frames
                    }
                    _ => self.frame >= spec.frames,
                };
                if done {
                    self.finish_phase();
                    self.phase += 1;
                    if self.phase == self.phases.len() {
                        event_loop.exit();
                    } else {
                        self.begin_phase();
                    }
                }
            }
            _ => {}
        }
    }
}

fn pct(sorted: &[f64], p: f64) -> f64 {
    sorted[((sorted.len() - 1) as f64 * p).round() as usize]
}

fn adapter_name() -> String {
    let instance = wgpu::Instance::default();
    match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())) {
        Ok(a) => {
            let i = a.get_info();
            format!("{} ({:?})", i.name, i.backend)
        }
        Err(_) => "unknown".into(),
    }
}

fn main() -> anyhow::Result<()> {
    let args = std::env::args().skip(1).filter(|a| a != "--bench");
    let Some(config) = Config::parse(args)? else {
        println!("frames bench: [--seed I32] [--view-distance 1..32] [--vertical-distance 1..16]");
        return Ok(());
    };
    let adapter = adapter_name();

    let phase = |name, route, frames, culling, vsync| Phase {
        name,
        route,
        frames,
        culling,
        vsync,
    };
    let phases = vec![
        phase(
            "Initial load (stand at spawn until stable)",
            Route::Load,
            4000,
            true,
            false,
        ),
        phase("Orbit at spawn, culling on", Route::Spin, 720, true, false),
        phase(
            "Orbit at spawn, culling off",
            Route::Spin,
            720,
            false,
            false,
        ),
        phase(
            "Orbit at spawn, culling on, vsync (as shipped)",
            Route::Spin,
            720,
            true,
            true,
        ),
        phase(
            "Flight, 600 blocks, culling on (streams chunks)",
            Route::Flight,
            1200,
            true,
            false,
        ),
    ];

    let mut app = App {
        render_state: None,
        world: WorldState::new(config.seed, config.view, config.height),
        phases,
        phase: 0,
        frame: 0,
        started: false,
        last: Instant::now(),
        cur: PhaseResult::default(),
        phase_start: Instant::now(),
        culling_on: true,
        stable: 0,
        last_resident: usize::MAX,
        last_change: Instant::now(),
        drawn_sum: 0.0,
        tri_sum: 0.0,
        counted: 0,
        results: Vec::new(),
        window_px: (0, 0),
    };
    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut app)?;

    anyhow::ensure!(
        app.results.len() == app.phases.len(),
        "window closed before the benchmark finished"
    );

    // A hidden or occluded window stops presenting: `render()` then returns early and frames take
    // microseconds with nothing drawn. Refuse to report numbers from such a run.
    for r in app
        .results
        .iter()
        .filter(|r| !r.name.starts_with("Initial"))
    {
        let skipped = r.frame_ms.iter().filter(|&&ms| ms < 0.5).count();
        anyhow::ensure!(
            skipped * 100 <= r.frame_ms.len() && r.resident > 0,
            "phase '{}' is invalid: {skipped} of {} frames were skipped (window hidden or not presenting?). Keep the window visible and rerun.",
            r.name,
            r.frame_ms.len()
        );
    }

    let mut out = String::new();
    let _ = writeln!(out, "# Between frame-time benchmark\n");
    let _ = writeln!(
        out,
        "Generated by `cargo bench --bench frames`. GPU: {adapter}. Framebuffer: {}×{} px. Seed {}, view distance {} / vertical {} ({} × {} × {} chunks), release profile, nightly rustc.\n",
        app.window_px.0,
        app.window_px.1,
        config.seed,
        config.view,
        config.height,
        config.view * 2,
        config.height * 2,
        config.view * 2,
    );
    let _ = writeln!(
        out,
        "| Phase | Frames | Avg FPS | Mean | p50 | p95 | p99 | Max | 1% low FPS | Drawn chunks | Triangles |"
    );
    let _ = writeln!(
        out,
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"
    );
    for r in &app.results {
        let mut v = r.frame_ms.clone();
        v.sort_by(|a, b| a.total_cmp(b));
        let n = v.len();
        let sum: f64 = v.iter().sum();
        let mean = sum / n as f64;
        let worst = &v[n - (n / 100).max(1)..];
        let low = 1000.0 / (worst.iter().sum::<f64>() / worst.len() as f64);
        let _ = writeln!(
            out,
            "| {} | {} | **{:.0}** | {:.2} ms | {:.2} ms | {:.2} ms | {:.2} ms | {:.1} ms | {:.0} | {:.0} | {:.2} M |",
            r.name,
            n,
            1000.0 / mean,
            mean,
            pct(&v, 0.5),
            pct(&v, 0.95),
            pct(&v, 0.99),
            v[n - 1],
            low,
            r.drawn,
            r.triangles / 1e6,
        );
    }
    let load = &app.results[0];
    let _ = writeln!(
        out,
        "\nInitial load: {:.2} s until the resident chunk count last changed (final: {} GPU meshes, {} CPU chunks).",
        load.total_s, load.resident, load.cpu_chunks
    );
    println!("{out}");
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/docs/benchmarks");
    std::fs::create_dir_all(dir)?;
    std::fs::write(format!("{dir}/frames.md"), &out)?;
    println!("Wrote {dir}/frames.md");
    Ok(())
}
