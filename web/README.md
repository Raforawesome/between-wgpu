# Between in the browser

A WebGPU build of the same engine. It needs the `wasm32-unknown-unknown` target,
Slang (as for the native build), and `wasm-bindgen-cli` matching the version in
`web/Cargo.lock`. `wasm-opt` is used if it's installed.

```sh
web/build.sh             # writes web/dist/
web/build.sh /some/dir   # or anywhere else
```

Serve the output with any static server. `index.html` takes `?seed=`, `?view=`
(default 10) and `?height=` (default 4).

## How it differs from native

- **No threads.** `std::thread` isn't available on `wasm32-unknown-unknown`, so
  `ChunkGenPipeline` has a wasm-only constructor (`src/world/pipeline/inline.rs`)
  that spawns nothing. Each frame the page runs queued generate/mesh jobs on the
  main thread for up to 8 ms via `run_next_task`. Hence the smaller default view.
- **Async setup.** WebGPU is awaited instead of using `pollster::block_on`.
- **Mouse look.** Click to capture the pointer; where pointer lock is refused,
  drag to look instead. Esc releases it.
- **No wireframe.** WebGPU has no line polygon mode, so P does nothing.

The only changes outside `web/` are those cfg-gated hooks in
`src/world/pipeline.rs` and `thread-priority` moving to a native-only
dependency (it doesn't compile for wasm). Native builds are unaffected.
