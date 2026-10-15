# Between

A voxel terrain engine built in Rust with wgpu and Slang shaders. It explores how procedural terrain moves from a density field through worker threads into compact GPU geometry.

![Seed 42 terrain rendered by Between on Apple M2 / Metal](assets/showcase/terrain.png)

## Run

Requires a **nightly Rust toolchain (1.94+)** and **Slang 2026.5.2** (`slangc` on PATH). Download Slang for your platform from its [official release](https://github.com/shader-slang/slang/releases/tag/v2026.5.2); alternatively set `SLANGC=/absolute/path/to/slangc`. The build compiles the Slang sources to WGSL. Nightly is required because `rapidhash` is built with its `nightly` feature.

```sh
cargo run --release --locked -- --seed 42 --view-distance 8 --vertical-distance 4
```

Chunk offsets use half-open ranges: `-distance..distance`. These settings select a 16 × 8 × 16 chunk region around the camera, with the vertical range following camera height. Smaller distances reduce the initial terrain region; `--view-distance 4 --vertical-distance 2` is a quick smoke run. Run with `RUST_LOG=info` to print the seed, settings, and GPU adapter.

| Control | Action |
| --- | --- |
| WASD / arrows, mouse | Free flight and look |
| Space / E, Ctrl / Q | Up, down |
| Left Shift | Sprint |
| P | Toggle wireframe (when supported by the GPU) |
| C | Toggle frustum culling for comparison |
| Escape | Quit |

The title bar reports frame-interval p50/p95 over the last second, submitted/resident chunk counts, culled chunks, triangles, and CPU chunks. Frame intervals include vsync and scheduling; they are not GPU timings.

## Engineering highlights

- **Terrain workers:** `WorldState` owns chunk storage, loading, and the queued/dirty sets; worker threads generate chunks and mesh immutable snapshots. Requests and results use unbounded channels.
- **Generation speed:** noise is sampled on a 5³ grid and trilinearly interpolated into each 16³ chunk, about 9× faster than per-voxel noise. A chunk generates in about 18 µs, scaling about 4× on 4 threads.
- **Frame rate:** about 240 FPS median uncapped on an Apple M2 at 3000×1800 with the default 32×12×32 view (4.0 ms p50, 7.6 ms p95); 60 FPS under the shipped vsync. Frustum culling cuts triangles about 4× (a ~16% frame-rate gain).
- **GPU cache:** recently visible meshes are retained in an LRU with both a 2,048-entry cap and a 64 MiB payload budget.
- **Visibility:** neighbour-aware hidden-face removal plus conservative chunk-AABB frustum culling compatible with infinite reversed Z. Only resident nonempty GPU meshes are considered for drawing.
- **Compact data:** flat 4,096-byte voxel arrays, non-air occupancy masks, four-byte packed vertices, indexed triangles, texture arrays, and per-chunk block-volume textures.
- **Procedural terrain:** seeded layered noise (continentalness, erosion, peaks and valleys, near-surface 3D detail).
- **Validation:** unit tests for the chunk loader and the LRU cache, plus repeatable CPU benchmarks.

The numbers above are measured on an Apple M2; see the [performance report](docs/performance.md) for methodology, results, and caveats. Figures other than the frame rate are CPU-only.

[Wireframe screenshot](assets/showcase/wireframe.png)

## Verify

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo bench --locked --bench engine   # CPU benchmarks
cargo bench --locked --bench frames   # frame times; opens a window
```

The `engine` benchmark needs no window or GPU and rewrites [docs/benchmarks/latest.md](docs/benchmarks/latest.md); `frames` needs a display, rewrites [docs/benchmarks/frames.md](docs/benchmarks/frames.md), and must stay visible while it runs. Slang is still required by the build. There is no CI configured.

## Scope and next experiments

This is an engine prototype with free flight. Meshing emits visible unit faces; it is not greedy meshing. Face lighting is a fixed artistic approximation. Water uses the opaque pass. Collision, block editing, persistence, transparent water, LOD, and shadows are not implemented. Generation is reproducible within the tested build; cross-platform bit-identical terrain is not guaranteed.

CPU chunks currently remain stored during travel, and generation covers the render region without a neighbour halo, so stationary boundary chunks can remain unmeshed.

Next: optimize chunk meshing (greedy meshing or material IDs in vertices) and compare against the baseline in the [performance report](docs/performance.md).

Runtime images are embedded from `assets/game`; `assets/raw` holds editable source artwork. Log and leaf layers currently reuse placeholder textures.
