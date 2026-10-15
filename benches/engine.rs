//! Headless CPU benchmarks for the terrain engine (no window, no GPU).
//!
//! Dependency-free harness: `harness = false`, a counting global allocator for memory
//! figures, and `std::time::Instant` for timing. Run with:
//!
//! ```sh
//! cargo bench --locked --bench engine
//! ```
//!
//! Results are printed as Markdown and written to `docs/benchmarks/latest.md`.
//! Set `BENCH_QUICK=1` for a fast smoke run (fewer repetitions, not for reporting).

use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::{HashSet as StdHashSet, VecDeque};
use std::fmt::Write as _;
use std::hint::black_box;
use std::mem::size_of;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::time::{Duration, Instant};

use between_wgpu::{
    HashSet,
    constants::{CHUNK_SIZEI, CHUNK_VOL},
    meshing::{ChunkMeshData, mesh_chunk},
    render::{frustum::Frustum, vertex::Vertex},
    util::lru::LruCache,
    world::{
        WorldSampler,
        block::BlockType,
        chunk_manager::ChunkManager,
        chunks::Chunk,
        loader::ChunkLoader,
        pipeline::{ChunkGenPipeline, PipelineResult},
    },
    worldgen::{interpolate::Trilerp, layered::LayeredGen},
};
use glam::{IVec3, Mat4, Vec3};

// ---------------------------------------------------------------------------------------
// Counting allocator
// ---------------------------------------------------------------------------------------

struct Counting;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static ALLOCS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(l) };
        if !p.is_null() {
            let live = LIVE.fetch_add(l.size(), Relaxed) + l.size();
            PEAK.fetch_max(live, Relaxed);
            ALLOCS.fetch_add(1, Relaxed);
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) };
        LIVE.fetch_sub(l.size(), Relaxed);
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, new: usize) -> *mut u8 {
        let np = unsafe { System.realloc(p, l, new) };
        if !np.is_null() {
            if new >= l.size() {
                let live = LIVE.fetch_add(new - l.size(), Relaxed) + (new - l.size());
                PEAK.fetch_max(live, Relaxed);
            } else {
                LIVE.fetch_sub(l.size() - new, Relaxed);
            }
            ALLOCS.fetch_add(1, Relaxed);
        }
        np
    }
}

#[global_allocator]
static ALLOC: Counting = Counting;

fn live() -> usize {
    LIVE.load(Relaxed)
}
fn reset_peak() {
    PEAK.store(live(), Relaxed);
}
fn peak() -> usize {
    PEAK.load(Relaxed)
}

// ---------------------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------------------

const SEED: i32 = 42;
/// Chunk the default player spawn (0, 90, -2) sits in.
const CENTRE: IVec3 = IVec3::new(0, 5, -1);

fn quick() -> bool {
    std::env::var_os("BENCH_QUICK").is_some()
}
fn reps() -> usize {
    if quick() { 1 } else { 5 }
}

fn sampler() -> WorldSampler {
    Trilerp::new(LayeredGen::new(SEED))
}

/// Chunk coords of a half-open `-h..h` / `-v..v` region around `CENTRE`, as `ChunkLoader` builds.
fn region(h: i32, v: i32) -> Vec<IVec3> {
    let mut out = Vec::with_capacity((4 * h * h * v) as usize);
    for z in -h..h {
        for y in -v..v {
            for x in -h..h {
                out.push(CENTRE + IVec3::new(x, y, z));
            }
        }
    }
    out
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    v[v.len() / 2]
}
fn pct(sorted: &[f64], p: f64) -> f64 {
    sorted[((sorted.len() - 1) as f64 * p).round() as usize]
}
fn fmt_us(ns: f64) -> String {
    if ns >= 1e6 {
        format!("{:.2} ms", ns / 1e6)
    } else if ns >= 1e3 {
        format!("{:.1} µs", ns / 1e3)
    } else {
        format!("{ns:.0} ns")
    }
}
fn mib(b: usize) -> String {
    format!("{:.2} MiB", b as f64 / (1024.0 * 1024.0))
}

/// Median over `reps()` runs of the wall time of `f`.
fn time_median(mut f: impl FnMut()) -> Duration {
    f(); // warm up
    let mut ts: Vec<f64> = (0..reps())
        .map(|_| {
            let t = Instant::now();
            f();
            t.elapsed().as_secs_f64()
        })
        .collect();
    Duration::from_secs_f64(median(std::mem::take(&mut ts)))
}

/// A full 16³ chunk's classification.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Class {
    Empty,
    Solid,
    Mixed,
}
fn classify(c: &Chunk) -> Class {
    if c.is_empty() {
        Class::Empty
    } else if c.solid_mask.iter().flatten().all(|&r| r == 0xFFFF) {
        Class::Solid
    } else {
        Class::Mixed
    }
}

struct Report(String);
impl Report {
    fn h2(&mut self, s: &str) {
        let _ = write!(self.0, "\n## {s}\n\n");
        println!("\n## {s}\n");
    }
    fn line(&mut self, s: &str) {
        let _ = writeln!(self.0, "{s}");
        println!("{s}");
    }
}

// ---------------------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------------------

/// Generates a region through the real worker pipeline; returns chunks and wall time.
fn generate_with_pipeline(coords: &[IVec3], threads: usize) -> (Vec<Chunk>, Duration) {
    let pipeline = ChunkGenPipeline::new(sampler(), threads);
    let t = Instant::now();
    for &c in coords {
        pipeline.request_generate(c);
    }
    let mut out = Vec::with_capacity(coords.len());
    while out.len() < coords.len() {
        match pipeline.result_channel().recv().unwrap() {
            PipelineResult::Generate(c) => out.push(*c),
            PipelineResult::Mesh(..) => unreachable!(),
        }
    }
    (out, t.elapsed())
}

fn bench_gen_single(r: &mut Report) -> Vec<Chunk> {
    r.h2("1. World generation, single thread");
    let coords = region(8, 4); // 16 × 8 × 16 = 2,048 chunks spanning sky, surface and ground
    let s = sampler();
    r.line(&format!(
        "Region: {} chunks (16×8×16 around the spawn chunk), seed {SEED}, `Trilerp<LayeredGen>`.\n",
        coords.len()
    ));

    // per-chunk latency, best-of-N per chunk to reject scheduler noise
    let n = reps().max(3);
    let mut per_chunk = vec![f64::MAX; coords.len()];
    let mut chunks = Vec::new();
    for pass in 0..n {
        chunks.clear();
        for (i, &c) in coords.iter().enumerate() {
            let t = Instant::now();
            let chunk = Chunk::from_sampler(c, &s);
            let dt = t.elapsed().as_nanos() as f64;
            per_chunk[i] = per_chunk[i].min(dt);
            if pass == n - 1 {
                chunks.push(black_box(chunk));
            }
        }
    }

    r.line("| Chunk class | Count | Mean | p50 | p95 | Max |");
    r.line("| --- | ---: | ---: | ---: | ---: | ---: |");
    for (name, class) in [
        ("all", None),
        ("empty (all air)", Some(Class::Empty)),
        ("mixed (surface)", Some(Class::Mixed)),
        ("solid (underground)", Some(Class::Solid)),
    ] {
        let mut v: Vec<f64> = chunks
            .iter()
            .zip(&per_chunk)
            .filter(|(c, _)| class.is_none_or(|k| classify(c) == k))
            .map(|(_, &t)| t)
            .collect();
        if v.is_empty() {
            continue;
        }
        v.sort_by(|a, b| a.total_cmp(b));
        let mean = v.iter().sum::<f64>() / v.len() as f64;
        r.line(&format!(
            "| {name} | {} | {} | {} | {} | {} |",
            v.len(),
            fmt_us(mean),
            fmt_us(pct(&v, 0.5)),
            fmt_us(pct(&v, 0.95)),
            fmt_us(*v.last().unwrap()),
        ));
    }
    let total: f64 = per_chunk.iter().sum();
    r.line(&format!(
        "\nSingle-thread throughput: **{:.0} chunks/s** ({:.1} M voxels/s), {} per chunk on average.",
        coords.len() as f64 / (total / 1e9),
        coords.len() as f64 * CHUNK_VOL as f64 / (total / 1e9) / 1e6,
        fmt_us(total / coords.len() as f64),
    ));
    chunks
}

fn bench_trilerp(r: &mut Report) {
    r.h2("2. Trilinear interpolation vs per-voxel noise");
    let coords: Vec<IVec3> = region(4, 4); // 8×8×8 = 512 chunks
    let direct = LayeredGen::new(SEED);
    let interp = sampler();

    let t_direct = time_median(|| {
        for &c in &coords {
            black_box(Chunk::from_sampler(c, &direct));
        }
    });
    let t_interp = time_median(|| {
        for &c in &coords {
            black_box(Chunk::from_sampler(c, &interp));
        }
    });

    // fidelity: how many voxels differ between the two strategies
    let (mut diff, mut total) = (0usize, 0usize);
    for &c in &coords {
        let a = Chunk::from_sampler(c, &direct);
        let b = Chunk::from_sampler(c, &interp);
        diff += a
            .blocks
            .iter()
            .zip(&b.blocks)
            .filter(|(x, y)| x != y)
            .count();
        total += CHUNK_VOL;
    }
    let per = |d: Duration| d.as_nanos() as f64 / coords.len() as f64;
    r.line(&format!("{} chunks, single thread.\n", coords.len()));
    r.line("| Strategy | Time / chunk | Speedup |");
    r.line("| --- | ---: | ---: |");
    r.line(&format!(
        "| Noise at all 4,096 voxels | {} | 1.00× |",
        fmt_us(per(t_direct))
    ));
    r.line(&format!(
        "| Noise on 5³ grid + trilinear | {} | **{:.2}×** |",
        fmt_us(per(t_interp)),
        per(t_direct) / per(t_interp)
    ));
    r.line(&format!(
        "\nFidelity cost: {diff} of {total} voxels ({:.3}%) differ in block type between the two strategies.",
        diff as f64 / total as f64 * 100.0
    ));
}

fn bench_scaling(r: &mut Report) -> Vec<Chunk> {
    r.h2("3. Generation thread scaling (`ChunkGenPipeline`)");
    let coords = region(16, 6); // the default view: 32 × 12 × 32
    r.line(&format!(
        "Region: {} chunks (default view distance 16 / vertical 6). Wall time to generate every chunk, median of {}.\n",
        coords.len(),
        reps()
    ));
    r.line("| Threads | Wall time | Chunks/s | Speedup | Efficiency |");
    r.line("| ---: | ---: | ---: | ---: | ---: |");
    let mut base = 0.0;
    let mut last = Vec::new();
    for threads in [1usize, 2, 4, 8] {
        let mut times = Vec::new();
        for _ in 0..reps() {
            let (chunks, dt) = generate_with_pipeline(&coords, threads);
            times.push(dt.as_secs_f64());
            last = chunks;
        }
        let t = median(times);
        if threads == 1 {
            base = t;
        }
        r.line(&format!(
            "| {threads} | {:.0} ms | {:.0} | {:.2}× | {:.0}% |",
            t * 1e3,
            coords.len() as f64 / t,
            base / t,
            base / t / threads as f64 * 100.0
        ));
    }
    r.line("\nThe app defaults to `available_parallelism / 2` workers (4 on this 8-core machine).");
    last
}

fn bench_meshing(r: &mut Report, region_chunks: &[Chunk]) {
    r.h2("4. Meshing baseline (current naive per-face mesher)");
    let mut man = ChunkManager::new();
    for c in region_chunks {
        man.insert_chunk(c.clone());
    }
    let snaps: Vec<(Arc<Chunk>, _)> = region_chunks
        .iter()
        .filter_map(|c| {
            Some((
                man.get_chunk(&c.coords)?.clone(),
                man.neighbour_snapshot(&c.coords)?,
            ))
        })
        .collect();

    let mut meshes: Vec<ChunkMeshData> = Vec::new();
    let t = time_median(|| {
        meshes.clear();
        for (c, n) in &snaps {
            meshes.push(black_box(mesh_chunk(c, n)));
        }
    });
    let nonempty: Vec<&ChunkMeshData> = meshes.iter().filter(|m| !m.vertices.is_empty()).collect();
    let verts: usize = nonempty.iter().map(|m| m.vertices.len()).sum();
    let idx: usize = nonempty.iter().map(|m| m.indices.len()).sum();
    let solid_voxels: usize = snaps
        .iter()
        .map(|(c, _)| c.blocks.iter().filter(|&&b| b != BlockType::Air).count())
        .sum();
    let faces = idx / 6;
    r.line(&format!(
        "{} chunks with full six-neighbour halos, single thread, median of {}.\n",
        snaps.len(),
        reps()
    ));
    r.line("| Metric | Value |");
    r.line("| --- | ---: |");
    r.line(&format!(
        "| Mesh time / chunk (all) | {} |",
        fmt_us(t.as_nanos() as f64 / snaps.len() as f64)
    ));
    r.line(&format!(
        "| Mesh time / non-empty mesh | {} |",
        fmt_us(t.as_nanos() as f64 / nonempty.len() as f64)
    ));
    r.line(&format!(
        "| Throughput | {:.0} chunks/s |",
        snaps.len() as f64 / t.as_secs_f64()
    ));
    r.line(&format!(
        "| Non-empty meshes | {} of {} |",
        nonempty.len(),
        snaps.len()
    ));
    r.line(&format!(
        "| Visible faces emitted | {faces} ({:.1} per non-empty chunk) |",
        faces as f64 / nonempty.len() as f64
    ));
    let naive = solid_voxels * 6;
    r.line(&format!(
        "| Faces vs. 6 per non-air voxel | {faces} vs {naive} → **{:.1}% removed** by hidden-face culling |",
        (1.0 - faces as f64 / naive as f64) * 100.0
    ));
    r.line(&format!(
        "| Vertices / indices | {verts} / {idx} ({:.2} vertices per face; 4 + 6 indices per quad) |",
        verts as f64 / faces as f64
    ));
}

fn bench_pipeline_e2e(r: &mut Report) {
    r.h2("5. End-to-end load: generate → neighbour-gate → mesh");
    r.line(
        "Headless replica of `WorldState::update`'s dispatch logic (same `ChunkManager`, `ChunkGenPipeline`, \
         neighbour gating and dirty set; GPU upload omitted). Measures time until every chunk that has all six \
         neighbours inside the region is meshed.\n",
    );
    r.line(
        "| Region | Threads | Chunks gen. | Chunks meshed | Time | Chunks/s (gen) | Peak heap |",
    );
    r.line("| --- | ---: | ---: | ---: | ---: | ---: | ---: |");
    for (h, v, label) in [(8, 4, "16×8×16"), (16, 6, "32×12×32 (default)")] {
        for threads in [4usize, 8] {
            let coords = region(h, v);
            let in_region: HashSet<IVec3> = coords.iter().copied().collect();
            let expected_meshes = coords
                .iter()
                .filter(|c| {
                    Chunk::neighbours_pos_list_at(**c)
                        .iter()
                        .all(|n| in_region.contains(n))
                })
                .count();

            let mut times = Vec::new();
            let mut peaks = Vec::new();
            for _ in 0..reps() {
                let base = live();
                reset_peak();
                let pipeline = ChunkGenPipeline::new(sampler(), threads);
                let mut man = ChunkManager::new();
                let mut dirty: HashSet<IVec3> = HashSet::default();
                let t = Instant::now();
                for &c in &coords {
                    pipeline.request_generate(c);
                }
                let (mut generated, mut meshed) = (0, 0);
                while meshed < expected_meshes || generated < coords.len() {
                    match pipeline.result_channel().recv().unwrap() {
                        PipelineResult::Generate(chunk) => {
                            let at = chunk.coords();
                            man.insert_chunk(*chunk);
                            dirty.insert(at);
                            generated += 1;
                            for pos in [at].into_iter().chain(Chunk::neighbours_pos_list_at(at)) {
                                if !dirty.contains(&pos) {
                                    continue;
                                }
                                if let (Some(ch), Some(nb)) =
                                    (man.get_chunk(&pos), man.neighbour_snapshot(&pos))
                                {
                                    pipeline.request_mesh(ch.clone(), nb);
                                    dirty.remove(&pos);
                                }
                            }
                        }
                        PipelineResult::Mesh(_, mesh) => {
                            meshed += 1;
                            black_box(mesh);
                        }
                    }
                }
                times.push(t.elapsed().as_secs_f64());
                peaks.push((peak() - base) as f64);
            }
            let t = median(times);
            r.line(&format!(
                "| {label} | {threads} | {} | {expected_meshes} | {:.0} ms | {:.0} | {} |",
                coords.len(),
                t * 1e3,
                coords.len() as f64 / t,
                mib(median(peaks) as usize),
            ));
        }
    }
    r.line("\nPeak heap includes in-flight meshes the main thread has not yet drained, and worker stacks are not counted.");
}

fn bench_memory(r: &mut Report, region_chunks: &[Chunk]) {
    r.h2("6. Memory footprint");
    r.line("Static sizes:\n");
    r.line("| Item | Size |");
    r.line("| --- | ---: |");
    r.line(&format!("| `BlockType` | {} B |", size_of::<BlockType>()));
    r.line(&format!(
        "| `Chunk` (blocks + coords + occupancy mask) | {} B ({} B blocks + {} B mask) |",
        size_of::<Chunk>(),
        CHUNK_VOL * size_of::<BlockType>(),
        size_of::<between_wgpu::constants::ChunkMask>()
    ));
    r.line(&format!(
        "| `Vertex` (packed) | {} B |",
        size_of::<Vertex>()
    ));
    r.line("| Hypothetical unpacked vertex (pos `[f32;3]` + uv `[f32;2]` + normal `[f32;3]`) | 32 B → packing is **8×** smaller |");

    // measured heap for the 2,048-chunk region held in a ChunkManager
    let before = live();
    let mut man = ChunkManager::new();
    for c in region_chunks {
        man.insert_chunk(c.clone());
    }
    let chunk_heap = live() - before;
    let n = region_chunks.len();
    r.line(&format!(
        "| `ChunkManager` holding {n} chunks (measured heap) | {} → {:.0} B per chunk |",
        mib(chunk_heap),
        chunk_heap as f64 / n as f64
    ));

    // CPU mesh buffers: len vs capacity
    let snaps: Vec<_> = region_chunks
        .iter()
        .filter_map(|c| {
            Some((
                man.get_chunk(&c.coords)?.clone(),
                man.neighbour_snapshot(&c.coords)?,
            ))
        })
        .collect();
    let (mut len_b, mut cap_b, mut nonempty) = (0usize, 0usize, 0usize);
    let meshes: Vec<ChunkMeshData> = snaps.iter().map(|(c, nb)| mesh_chunk(c, nb)).collect();
    for m in &meshes {
        if m.vertices.is_empty() {
            continue;
        }
        nonempty += 1;
        len_b += m.vertices.len() * 4 + m.indices.len() * 4;
        cap_b += m.vertices.capacity() * 4 + m.indices.capacity() * 4;
    }
    r.line(&format!(
        "| Mesh data, {nonempty} non-empty chunks | {} used, {} allocated ({:.0}% slack) → {:.0} B per chunk |",
        mib(len_b),
        mib(cap_b),
        (cap_b - len_b) as f64 / cap_b as f64 * 100.0,
        len_b as f64 / nonempty as f64
    ));
    // GPU side: vertex + index + 16³ R8 block-volume texture (4096 B)
    let gpu_total = len_b + nonempty * CHUNK_VOL;
    r.line(&format!(
        "| GPU data for those chunks (vertex + index + 4 KiB block texture) | {} → {:.1} KiB per chunk |",
        mib(gpu_total),
        gpu_total as f64 / nonempty as f64 / 1024.0
    ));
    let mean_gpu = gpu_total as f64 / nonempty as f64;
    r.line(&format!(
        "| Chunks fitting the 64 MiB GPU cache budget at that mean | ≈ {:.0} (entry cap is 2,048) |",
        64.0 * 1024.0 * 1024.0 / mean_gpu
    ));
    drop(meshes);

    // bytes per rendered quad
    let quads: usize = snaps
        .iter()
        .map(|(c, nb)| mesh_chunk(c, nb).indices.len() / 6)
        .sum();
    r.line(&format!(
        "| Mesh bytes per visible face (4 vertices × 4 B + 6 indices × 4 B) | 40 B ({quads} faces in region) |"
    ));
}

fn bench_culling(r: &mut Report, region_chunks: &[Chunk]) {
    r.h2("7. Frustum culling");
    let nonempty: Vec<IVec3> = region_chunks
        .iter()
        .filter(|c| !c.is_empty())
        .map(|c| c.coords)
        .collect();
    let eye = Vec3::new(0.0, 90.0, -2.0);
    let proj = Mat4::perspective_infinite_reverse_lh(50f32.to_radians(), 16.0 / 9.0, 0.1);

    let mut kept_total = 0usize;
    let mut rows = Vec::new();
    for yaw_deg in (0..360).step_by(45) {
        let yaw = (yaw_deg as f32).to_radians();
        let dir = Vec3::new(yaw.sin(), -0.1, yaw.cos());
        let vp = proj * Mat4::look_to_lh(eye, dir, Vec3::Y);
        let f = Frustum::from_view_projection(vp);
        let kept = nonempty.iter().filter(|&&c| f.contains_chunk(c)).count();
        kept_total += kept;
        rows.push((yaw_deg, kept));
    }
    r.line(&format!(
        "{} non-empty chunks in the 32×12×32 default region ({} chunks total), camera at the spawn point, FOV 50° at 16:9, 8 headings.\n",
        nonempty.len(),
        region_chunks.len()
    ));
    r.line("| Heading | Chunks drawn | Culled |");
    r.line("| ---: | ---: | ---: |");
    for (yaw, kept) in rows {
        r.line(&format!(
            "| {yaw}° | {kept} | {:.1}% |",
            (1.0 - kept as f64 / nonempty.len() as f64) * 100.0
        ));
    }
    let avg_kept = kept_total as f64 / 8.0;
    r.line(&format!(
        "\nAverage: **{:.1}% of non-empty chunks culled** ({avg_kept:.0} of {} drawn).",
        (1.0 - avg_kept / nonempty.len() as f64) * 100.0,
        nonempty.len()
    ));

    let vp = proj * Mat4::look_to_lh(eye, Vec3::new(0.0, -0.1, 1.0), Vec3::Y);
    let iters = if quick() { 20 } else { 400 };
    let t = Instant::now();
    for _ in 0..iters {
        let f = Frustum::from_view_projection(black_box(vp));
        let mut k = 0;
        for &c in &nonempty {
            k += f.contains_chunk(c) as usize;
        }
        black_box(k);
    }
    let per_frame = t.elapsed().as_nanos() as f64 / iters as f64;
    r.line(&format!(
        "Cost: {} per frame to test all {} chunks ({:.1} ns per chunk).",
        fmt_us(per_frame),
        nonempty.len(),
        per_frame / nonempty.len() as f64
    ));
    r.line("\nThis counts chunks submitted for drawing. It is not an FPS measurement.");
}

fn bench_lru(r: &mut Report) {
    r.h2("8. GPU mesh cache LRU");
    const CAP: usize = 2048;
    let ops = if quick() { 200_000 } else { 2_000_000 };
    // LCG-driven key stream over a working set 1.5× the capacity, so the cache evicts constantly
    let universe = (CAP * 3 / 2) as u32;
    let keys: Vec<IVec3> = {
        let mut s = 0x9E3779B97F4A7C15u64;
        (0..ops)
            .map(|_| {
                s = s
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let k = ((s >> 33) as u32) % universe;
                IVec3::new((k % 32) as i32, (k / 32 % 16) as i32, (k / 512) as i32)
            })
            .collect()
    };

    let run_arena = || {
        let mut c: LruCache<IVec3, u32, CAP> = LruCache::default();
        let mut hits = 0usize;
        for (i, k) in keys.iter().enumerate() {
            if c.get(k).is_some() {
                hits += 1;
            } else {
                c.put(*k, i as u32);
            }
        }
        hits
    };
    let mut hits_a = 0;
    let ta = time_median(|| hits_a = black_box(run_arena()));
    // naive baseline: VecDeque with linear search for move-to-front. It is O(n) per op,
    // so it runs on a shorter prefix of the same key stream; the figures are per-op.
    let naive_ops = (ops / 20).max(10_000);
    let keys_prefix = &keys[..naive_ops];
    let run_naive_prefix = || {
        let mut q: VecDeque<(IVec3, u32)> = VecDeque::with_capacity(CAP);
        for (i, k) in keys_prefix.iter().enumerate() {
            if let Some(pos) = q.iter().position(|(kk, _)| kk == k) {
                let e = q.remove(pos).unwrap();
                q.push_front(e);
            } else {
                if q.len() == CAP {
                    q.pop_back();
                }
                q.push_front((*k, i as u32));
            }
        }
    };
    let tn = time_median(run_naive_prefix);
    let ns_a = ta.as_nanos() as f64 / ops as f64;
    let ns_n = tn.as_nanos() as f64 / naive_ops as f64;
    r.line(&format!(
        "Capacity {CAP}, {ops} get-or-put operations over {universe} distinct chunk keys (hit rate {:.1}%).\n",
        hits_a as f64 / ops as f64 * 100.0
    ));
    r.line("| Implementation | Time / op | Speedup |");
    r.line("| --- | ---: | ---: |");
    r.line(&format!(
        "| `VecDeque` + linear search (naive) | {} | 1× |",
        fmt_us(ns_n)
    ));
    r.line(&format!(
        "| `LruCache` (arena-linked list + rapidhash map) | {} | **{:.0}×** |",
        fmt_us(ns_a),
        ns_n / ns_a
    ));
}

fn bench_hashing_and_loader(r: &mut Report) {
    r.h2("9. Chunk-coordinate hashing and loader rebuild");
    let coords = region(16, 6);
    let iters = if quick() { 5 } else { 100 };

    let t_rapid = time_median(|| {
        for _ in 0..iters {
            let mut s: HashSet<IVec3> =
                HashSet::with_capacity_and_hasher(coords.len(), Default::default());
            for &c in &coords {
                s.insert(c);
            }
            let mut hit = 0;
            for &c in &coords {
                hit += s.contains(&c) as usize;
            }
            black_box(hit);
        }
    });
    let t_sip = time_median(|| {
        for _ in 0..iters {
            let mut s: StdHashSet<IVec3> = StdHashSet::with_capacity(coords.len());
            for &c in &coords {
                s.insert(c);
            }
            let mut hit = 0;
            for &c in &coords {
                hit += s.contains(&c) as usize;
            }
            black_box(hit);
        }
    });
    let ops = (iters * coords.len() * 2) as f64;
    r.line(&format!(
        "Insert + lookup of {} chunk coordinates into pre-sized sets ({}-op batches).\n",
        coords.len(),
        coords.len() * 2
    ));
    r.line("| Hasher | Time / op | Speedup |");
    r.line("| --- | ---: | ---: |");
    let (a, b) = (
        t_sip.as_nanos() as f64 / ops,
        t_rapid.as_nanos() as f64 / ops,
    );
    r.line(&format!("| `std` SipHash-1-3 | {} | 1× |", fmt_us(a)));
    r.line(&format!(
        "| rapidhash (`crate::HashSet`) | {} | **{:.2}×** |",
        fmt_us(b),
        a / b
    ));

    // loader: cost of a chunk-boundary crossing (full 12,288-entry rebuild)
    let mut loader = ChunkLoader::new(16, 6);
    let mut x = 0.0f32;
    let t = time_median(|| {
        for _ in 0..iters {
            x += CHUNK_SIZEI as f32; // cross a boundary every call
            black_box(loader.update(Vec3::new(x, 90.0, -2.0)));
        }
    });
    r.line(&format!(
        "\n`ChunkLoader::update` on a chunk-boundary crossing (default region, 12,288 chunks): **{}** per rebuild, \
         and ~0 on frames that stay inside a chunk.",
        fmt_us(t.as_nanos() as f64 / iters as f64)
    ));
}

fn main() {
    // `cargo bench` passes `--bench`; ignore all args
    let t0 = Instant::now();
    let mut r = Report(String::new());
    r.line("# Between engine benchmarks");
    r.line(&format!(
        "\nGenerated by `cargo bench --bench engine`{}. {} logical cores, release profile, seed {SEED}.",
        if quick() { " (QUICK mode: not for reporting)" } else { "" },
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1),
    ));

    let region_2048 = bench_gen_single(&mut r);
    bench_trilerp(&mut r);
    let region_default = bench_scaling(&mut r);
    bench_meshing(&mut r, &region_2048);
    bench_pipeline_e2e(&mut r);
    bench_memory(&mut r, &region_2048);
    bench_culling(&mut r, &region_default);
    bench_lru(&mut r);
    bench_hashing_and_loader(&mut r);

    r.line(&format!(
        "\n_Total benchmark time: {:.0} s._",
        t0.elapsed().as_secs_f64()
    ));
    if !quick() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/docs/benchmarks/latest.md");
        std::fs::create_dir_all(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/benchmarks")).unwrap();
        std::fs::write(path, &r.0).unwrap();
        println!("\nWrote {path}");
    }
}
