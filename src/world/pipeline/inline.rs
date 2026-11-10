//! Browser (wasm32) version of the pipeline. There are no threads there, so
//! nothing is spawned: the owner calls [ChunkGenPipeline::run_next_task]
//! from its frame loop for as long as its time budget allows.
//! Native builds don't compile this file.

use crossbeam_channel::{Receiver, Sender};

use super::{
    Chunk, ChunkGenPipeline, GenTask, MeshTask, PipelineResult, PipelineTask, WorldSampler,
    mesh_chunk,
};

/// The worker's half of the channels, kept here instead of in a thread.
pub(super) struct InlineRunner {
    request_rx: Receiver<PipelineTask>,
    result_tx: Sender<PipelineResult>,
    density_sampler: WorldSampler,
}

impl ChunkGenPipeline {
    /// Same signature as the native constructor; `threads` is ignored.
    pub fn new(density_sampler: WorldSampler, _threads: usize) -> Self {
        let (request_tx, request_rx) = crossbeam_channel::unbounded();
        let (result_tx, result_rx) = crossbeam_channel::unbounded();
        Self {
            request_tx,
            result_rx,
            inline: InlineRunner {
                request_rx,
                result_tx,
                density_sampler,
            },
        }
    }

    /// Runs one queued task on the calling thread, exactly as a native worker
    /// would. Returns `false` if nothing was queued.
    pub fn run_next_task(&self) -> bool {
        let runner = &self.inline;
        let Ok(task) = runner.request_rx.try_recv() else {
            return false;
        };
        let result = match task {
            PipelineTask::Generate(GenTask(coords)) => {
                let chunk = Chunk::from_sampler(coords, &runner.density_sampler);
                PipelineResult::Generate(Box::new(chunk))
            }
            PipelineTask::Mesh(mesh_task) => {
                let MeshTask { chunk, neighbours } = *mesh_task;
                let mesh_data = mesh_chunk(&chunk, &neighbours);
                PipelineResult::Mesh(chunk.coords, mesh_data)
            }
        };
        // the receiver lives in `self`, so this send can't fail
        let _ = runner.result_tx.send(result);
        true
    }

    /// Number of tasks still queued.
    pub fn pending_tasks(&self) -> usize {
        self.inline.request_rx.len()
    }
}
