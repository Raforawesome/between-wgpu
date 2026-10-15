# Performance report

All figures come from `cargo bench --locked --bench engine` (source: [`benches/engine.rs`](../benches/engine.rs), raw output: [`benchmarks/latest.md`](benchmarks/latest.md)).

- **Machine:** Apple M2, 8 logical cores, macOS 26.5.1, rustc 1.101.0-nightly (the project requires nightly), release profile.
- **Scenario:** seed 42, camera at the default spawn (0, 90, -2), default view 16 / vertical 6 (a 32×12×32 = 12,288-chunk region) unless noted.
- **Scope:** sections 1–6 are CPU only and need no window or GPU. Section 7 renders through the real wgpu/Metal renderer and reports frame times; it is wall-clock frame intervals, not isolated GPU time.
- **Method:** std-only harness with a counting global allocator. Timings are the median of 5 runs after a warm-up. Per-chunk latencies are best-of-5 per chunk. Repeat runs agreed within a few percent; the exception is hasher timings, which depend on set pre-sizing (noted below).

## Headline numbers

| Area | Result |
| --- | --- |
| Terrain generation, 1 thread | **~54,000 chunks/s** (≈ 220 M voxels/s), 18.5 µs per 16³ chunk |
| Generation scaling | **4.2× at 4 threads, 5.9× at 8 threads** (the app's default is 4) |
| Default view fully generated | **34 ms on 8 threads**, 48 ms on 4 (12,288 chunks) |
| Generate + mesh end to end | **78 ms on 4 threads** for the default view, 9,000 chunks meshed (GPU upload excluded) |
| Trilinear interpolation of density | **9.4× faster** than noise at every voxel, at a cost of 0.11% differing voxels |
| Hidden-face removal | **99.5% of faces removed** (78 k emitted vs 15.8 M for 6 per solid voxel) |
| Frustum culling | **78% of non-empty chunks culled** on average, at 8.5 ns per chunk (59 µs per frame for 6,940 chunks) |
| Frame rate (3000×1800, M2, uncapped) | **~240 FPS median** (4.0 ms p50, 7.6 ms p95) with culling; ~200 FPS without |
| Vertex size | **4 B** packed, 8× smaller than a 32 B unpacked layout |
| LRU mesh cache | **20 ns/op**, 47× faster than a linear-search LRU at capacity 2,048 |
| Chunk-coordinate hashing | rapidhash **2.7× faster** than std SipHash on pre-sized sets (11 → 4 ns/op) |

## 1. Terrain generation

`Chunk::from_sampler` over a 16×8×16 slice of the world around spawn (2,048 chunks), single thread:

| Chunk class | Count | Mean | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: | ---: |
| all | 2,048 | 18.5 µs | 17.1 µs | 25.2 µs | 29.6 µs |
| empty (all air) | 791 | 17.2 µs | 16.8 µs | 20.1 µs | 24.5 µs |
| mixed (surface) | 338 | 24.0 µs | 24.1 µs | 27.1 µs | 29.6 µs |
| solid (underground) | 919 | 17.6 µs | 17.1 µs | 20.5 µs | 23.8 µs |

Cost is almost flat across chunk types. The 2D continentalness/erosion/weirdness noise runs for every column, and 3D detail noise only runs within ±8 blocks of the surface, so surface chunks cost about 40% more than empty or solid ones. The worst chunk is under 30 µs, so no single task stalls a worker.

### Trilinear interpolation

`Trilerp` samples density on a 5³ grid and interpolates to the 16³ voxels. Against evaluating `LayeredGen` at all 4,096 voxels (512 chunks, single thread):

| Strategy | Time / chunk | Speedup |
| --- | ---: | ---: |
| Noise at every voxel | 185.6 µs | 1× |
| 5³ grid + trilinear | 19.7 µs | **9.4×** |

The cost is fidelity: 2,304 of 2,097,152 voxels (0.11%) change block type. These are boundary voxels at the surface, so the terrain is visually the same.

### Thread scaling

`ChunkGenPipeline` generating the whole default region (12,288 chunks), wall time:

| Threads | Wall time | Chunks/s | Speedup | Efficiency |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 199 ms | 61,825 | 1.00× | 100% |
| 2 | 99 ms | 124,454 | 2.01× | 101% |
| 4 | 48 ms | 258,180 | 4.18× | 104% |
| 8 | 34 ms | 363,589 | 5.88× | 74% |

Scaling is close to linear up to 4 threads (the slightly-over-100% efficiency is run-to-run variation, not a real superlinear effect). The drop at 8 threads is consistent with the M2's 4 performance + 4 efficiency core layout, though I did not isolate that cause.

## 2. Meshing (baseline)

This is the current per-face mesher, recorded as the baseline for the planned optimization. 1,176 chunks with full six-neighbour halos, single thread:

| Metric | Value |
| --- | ---: |
| Time per chunk (all) | 12.3 µs |
| Time per non-empty mesh | 50.9 µs |
| Throughput | 81,000 chunks/s |
| Non-empty meshes | 284 of 1,176 |
| Faces per non-empty chunk | 275.6 |
| Faces vs 6 per non-air voxel | 78,268 vs 15,843,312 (**99.5% removed**) |

The 99.5% figure counts every non-air voxel as 6 faces, including buried ones, so it overstates the saving against a mesher that only skips obviously empty chunks. It is a fair measure of neighbour-aware culling against drawing every cube.

## 3. End-to-end load

A headless replica of the `WorldState::update` dispatch (same `ChunkManager`, pipeline, neighbour gating and dirty set; the GPU upload is omitted). It measures the time until every chunk with all six neighbours in the region is meshed.

| Region | Threads | Generated | Meshed | Time | Peak heap |
| --- | ---: | ---: | ---: | ---: | ---: |
| 16×8×16 | 4 | 2,048 | 1,176 | 12 ms | 9.8 MiB |
| 16×8×16 | 8 | 2,048 | 1,176 | 9 ms | 14.4 MiB |
| 32×12×32 (default) | 4 | 12,288 | 9,000 | 78 ms | 59.4 MiB |
| 32×12×32 (default) | 8 | 12,288 | 9,000 | 52 ms | 88.8 MiB |

The CPU side of a full default-radius load takes under 0.1 s on the default 4 workers. Peak heap includes meshes the main thread has not yet drained, so it is an upper bound for this headless drain loop.

## 4. Memory

| Item | Size |
| --- | ---: |
| `Chunk` | 4,620 B (4,096 B blocks + 512 B occupancy mask + coords) |
| `ChunkManager`, 2,048 chunks (measured heap) | 9.16 MiB → 4,689 B/chunk |
| `Vertex` (packed into one `u32`) | 4 B vs 32 B unpacked (8×) |
| Mesh data, 284 non-empty chunks | 2.99 MiB used → 11.0 KB per non-empty chunk |
| Mesh `Vec` slack before upload | 35% of the allocated bytes |
| GPU data per non-empty chunk (vertex + index + 4 KiB block texture) | ≈ 14.8 KiB |
| 64 MiB GPU cache budget at that mean | ≈ 4,400 chunks (the 2,048-entry cap binds first) |

At the default radius a fully resident world is 12,288 × 4.7 KB ≈ 56 MiB of block data, since the CPU keeps all generated chunks. The 35% mesh slack is the up-front `Vec::with_capacity(512 / 768)`, which over-reserves for small meshes. Sizing it from the final count would be an easy follow-up.

The unpacked-vertex row is a hypothetical comparison, not something the engine ever stored.

## 5. Visibility

6,940 of the 12,288 default-region chunks are non-empty. With the camera at spawn, FOV 50° at 16:9, over 8 headings:

| Heading | Chunks drawn | Culled |
| ---: | ---: | ---: |
| 0° | 1,162 | 83.3% |
| 45° | 1,324 | 80.9% |
| 90° | 1,461 | 78.9% |
| 135° | 1,796 | 74.1% |
| 180° | 1,813 | 73.9% |
| 225° | 1,704 | 75.4% |
| 270° | 1,453 | 79.1% |
| 315° | 1,321 | 81.0% |

Average **78.3%** culled (1,504 of 6,940 drawn). Testing every chunk takes 59 µs per frame (8.5 ns/chunk), so culling costs about 0.4% of a 16.7 ms frame. These are submitted-chunk counts, not FPS.

## 6. Data structures

**GPU mesh cache.** `LruCache` (arena-linked list + rapidhash map), capacity 2,048, 2 M get-or-put operations over 3,072 keys (67% hit rate):

| Implementation | Time / op | Speedup |
| --- | ---: | ---: |
| `VecDeque` + linear search | 949 ns | 1× |
| `LruCache` | 20 ns | **47×** |

The baseline is a deliberately simple LRU, not `lru` or another crate. The point is O(1) versus O(n), which matters when a 2,048-entry list is scanned on every cache probe.

**Hashing.** Insert + lookup of the 12,288 default-region coordinates into pre-sized sets: SipHash 11 ns/op, rapidhash 4 ns/op (**2.7×**). An earlier un-sized run showed 15× because it included map growth, so pre-sizing is required for a fair comparison.

**Loader.** `ChunkLoader::update` rebuilds the 12,288-entry set in **≈ 50 µs** when the player crosses a chunk boundary and costs ~0 otherwise.

## 7. Frame time

[`benches/frames.rs`](../benches/frames.rs) drives the real `WorldState` + `RenderState` through a scripted, frame-indexed camera route in a real window (3000×1800 px framebuffer, Apple M2 / Metal, default 32×12×32 view, seed 42). Present mode is switched to `Immediate` so vsync does not cap the result, except in one phase that measures the shipped behaviour. A frame interval is the wall time between consecutive redraws: world update (channel draining, mesh uploads), culling, encoding, submission and presentation back-pressure.

Median of 5 runs, with the min–max range across runs in brackets:

| Phase | Avg FPS | p50 | p95 | p99 | 1% low FPS | Chunks drawn | Triangles |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Orbit at spawn, culling on | **235** [219–253] | 4.03 ms | 7.6 ms | 9.6 ms | 37 | 303 | 0.18 M |
| Orbit at spawn, culling off | **202** [186–211] | 4.75 ms | 7.7 ms | 9.3 ms | 33 | 1,272 | 0.77 M |
| Flight, 600 blocks, culling on | **243** [233–252] | 3.88 ms | 7.3 ms | 9.7 ms | 41 | 266 | 0.16 M |
| Orbit at spawn, vsync (as shipped) | **60** | 16.67 ms | 17.9 ms | 18.4 ms | 54 | 303 | 0.18 M |

- **Orbit** stands at spawn and turns 0.5° per frame (720 frames per revolution). It isolates rendering from world updates.
- **Flight** moves 0.5 blocks per frame for 1,200 frames (600 blocks, about 37 chunks), so it crosses chunk boundaries and streams chunks in and out. The route is frame-indexed, so every run takes the same path regardless of frame rate.
- **Culling:** frustum culling cuts the submitted work from 1,272 to ~303 chunks and from 0.77 M to 0.18 M triangles, a 4.3× reduction in triangles. That is about a **16% median frame-rate gain** (202 → 235 FPS). The gain is much smaller than the triangle reduction because at roughly 4 ms per frame the renderer is not triangle-bound at this scene size (see caveats).
- **Streaming:** flight over fresh terrain is no slower than standing still (243 vs 235 FPS median, within run-to-run noise), so chunk generation, meshing and upload on worker threads do not show up in steady frame time.
- **Initial load:** the 1,272 non-empty meshes (12,288 CPU chunks) were all resident about **0.14 s** (0.12–0.15 s) after the first frame. That window includes the load hitches below: p95 about 16.9 ms and worst frame about 78 ms, because mesh uploads are not budgeted per frame.
- **vsync:** at the shipped present mode the renderer holds 60 FPS with p99 18.4 ms. The display is capping the frame rate, so there is large headroom (about 4 ms of work in a 16.7 ms frame).

### Caveats for this section

- **Single machine and one scene.** At 3000×1800 with ~300 draw calls, the 4 ms floor suggests the frame is bounded by something other than triangle count (per-draw encoding and driver overhead, or presentation), but I have not profiled which. Don't present these as GPU-bound or CPU-bound figures.
- **~45 ms spike in every uncapped phase.** The worst frame is 35–52 ms in every `Immediate` phase and never appears in the vsync phase. It pulls the 1% low down to 30–48 FPS. I did not find the cause; it looks like a macOS presentation artefact specific to the uncapped mode rather than engine work, but that is unverified. Quote p50/p95/p99 rather than the maximum.
- **Occlusion and sleep invalidate runs.** A hidden or occluded window stops presenting and frames return in microseconds. The bench detects this and refuses to write results. Keep the window visible and the Mac awake. Wrapping the run in `caffeinate -d` made macOS cap every phase at 60 FPS in my tests, so don't.
- **Run-to-run variance is real** (one stray earlier run had culling-off at 116 FPS). Hence the medians of 5 runs; repeat the bench several times before quoting a number.
- It needs a display and GPU, so unlike `engine` it cannot run in CI or over SSH.

## Reproducing

```sh
cargo bench --locked --bench engine    # ~5 s; rewrites docs/benchmarks/latest.md
BENCH_QUICK=1 cargo bench --locked --bench engine   # smoke run, single repetition, writes nothing
cargo bench --locked --bench frames    # opens a window; ~40 s; rewrites docs/benchmarks/frames.md
```

Slang is still required by `build.rs`. Close other work first, because the 8-thread rows are sensitive to background load. Numbers will differ on other hardware, and the 8-thread efficiency in particular depends on the core mix.

## Limits of these numbers

- Sections 1–6 are CPU-only; section 7 measures whole-frame wall time, not isolated GPU time or upload cost. Everything is single-machine.
- The meshing numbers describe the current naive mesher and will change when it is replaced. Re-run the bench for a before/after.
- The LRU and hasher comparisons are against simple baselines, which is what makes them large. Quote them as "vs. a linear-scan LRU" and "vs. SipHash", not as absolute claims.
- Peak heap excludes thread stacks and anything the allocator caches.
