# Between

[![CI](https://github.com/Raforawesome/between-wgpu/actions/workflows/ci.yml/badge.svg)](https://github.com/Raforawesome/between-wgpu/actions/workflows/ci.yml)

A voxel terrain engine built in Rust with wgpu and Slang shaders. It explores how procedural terrain moves from a density field through worker threads into compact GPU geometry.

![Seed 42 terrain rendered by Between on Apple M2 / Metal](assets/showcase/terrain.png)

A walkthrough of the main optimizations, with animations, is in [this thread on X](https://x.com/raforawesome/status/2114935770566062370).

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

### Lock-free terrain workers

`WorldState` owns chunk storage, loading, and the queued/dirty sets. Four persistent worker threads pull generate or mesh jobs off a shared crossbeam channel and send results back. Meshing jobs carry immutable snapshots of the chunk's neighbours, so workers never need locks. Generation scales 4.2× on 4 threads; 8 threads reach 5.9× (74% efficiency), likely held back by the M2's 4 performance + 4 efficiency cores. The full default view (12,288 chunks) generates and meshes in 78 ms, so flying into new terrain costs about the same as standing still.

### Trilinear density interpolation

<img src="assets/showcase/concepts/trilinear.gif" width="720" alt="Animation comparing noise sampled at every voxel with a coarse grid filled by trilinear interpolation">

Instead of evaluating noise at all 4,096 voxels of a 16³ chunk, the generator samples a 5³ grid and fills the rest with trilinear interpolation: 186 → 20 µs per chunk (9.4×). Only 0.11% of voxels change block type, all of them at the surface.

### Cache-friendly iteration

<img src="assets/showcase/concepts/cache_iteration.gif" width="720" alt="Animation of z-innermost versus x-innermost loops walking a chunk array">

Chunks are flat 4 KB arrays laid out x → y → z. With z innermost, every step jumped 256 elements; making z the outer loop and x the inner one walks memory in a straight line, so the hardware prefetcher stays ahead and the compiler can auto-vectorize.

### 32-bit packed vertices

<img src="assets/showcase/concepts/vertex_packing.gif" width="720" alt="Animation of the bit layout of a packed 32-bit vertex">

Each vertex is a single `u32` instead of a typical 32-byte float vertex, 8× less vertex memory and bandwidth: 5 bits each for x, y, z within the chunk (the vertex shader adds the world offset), 2 bits of UV corner, 3 bits of face, and 12 spare. There is no normal (the face implies it) and no block type: the fragment shader looks that up in a per-chunk 16³ block-volume texture.

### Hidden-face removal and culling

<img src="assets/showcase/concepts/face_culling.gif" width="720" alt="Animation of hidden faces being removed from a block of voxels">

Workers check each block's neighbours, across chunk borders via the immutable snapshots, and skip faces that can't be seen: 78k faces emitted versus 15.8M at 6 per solid voxel, 99.5% removed. Conservative chunk-AABB frustum culling, compatible with infinite reversed Z, then cuts submitted triangles about 4×.

### Also

- **Frame rate:** about 240 FPS median uncapped on an Apple M2 at 3000×1800 with the default 32×12×32 view (4.0 ms p50, 7.6 ms p95); 60 FPS under the shipped vsync. Frustum culling is worth about 16% in frame rate.
- **GPU cache:** recently visible meshes are retained in an LRU with both a 2,048-entry cap and a 64 MiB payload budget.
- **Compact data:** flat 4,096-byte voxel arrays, non-air occupancy masks, indexed triangles, and texture arrays.
- **Procedural terrain:** seeded layered noise (continentalness, erosion, peaks and valleys, near-surface 3D detail).
- **Validation:** unit tests for the chunk loader and the LRU cache, repeatable CPU benchmarks, and CI running fmt, clippy, and tests.

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

The `engine` benchmark needs no window or GPU and rewrites [docs/benchmarks/latest.md](docs/benchmarks/latest.md); `frames` needs a display, rewrites [docs/benchmarks/frames.md](docs/benchmarks/frames.md), and must stay visible while it runs. Slang is still required by the build. CI runs the fmt, clippy, and test steps on every push.

## Scope and next experiments

This is an engine prototype with free flight. Meshing emits visible unit faces; it is not greedy meshing. Face lighting is a fixed artistic approximation. Water uses the opaque pass. Collision, block editing, persistence, transparent water, LOD, and shadows are not implemented. Generation is reproducible within the tested build; cross-platform bit-identical terrain is not guaranteed.

CPU chunks currently remain stored during travel, and generation covers the render region without a neighbour halo, so stationary boundary chunks can remain unmeshed.

Next: optimize chunk meshing (greedy meshing or material IDs in vertices) and compare against the baseline in the [performance report](docs/performance.md).

Runtime images are embedded from `assets/game`; `assets/raw` holds editable source artwork. Log and leaf layers currently reuse placeholder textures.

## License

The source code is licensed under the [GNU GPL v3.0](LICENSE). The artwork in `assets/` is licensed under [CC BY-SA 4.0](LICENSE-ASSETS), except the third-party Colorful-42 palette (`assets/raw/colorful-42.gpl`).
