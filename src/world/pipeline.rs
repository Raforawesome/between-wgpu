use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use glam::IVec3;

use crate::{
    meshing::{ChunkMeshData, mesh_chunk},
    world::{WorldSampler, chunk_manager::NeighbourSnapshot, chunks::Chunk},
};

#[cfg(target_arch = "wasm32")]
mod inline;

/// Wrapper struct which represents a generation task for a given chunk coordinate.
pub struct GenTask(pub IVec3);

/// Represents a meshing task for an already-generated chunk.
pub struct MeshTask {
    chunk: Arc<Chunk>,
    neighbours: NeighbourSnapshot,
}

// Sum type for pipeline tasks.
pub enum PipelineTask {
    Generate(GenTask),
    Mesh(Box<MeshTask>),
}

/// Represents the outcome of a pipeline task. Pipeline tasks can be one of:
/// - Generate
/// - Mesh
pub enum PipelineResult {
    Generate(Box<Chunk>),
    Mesh(IVec3, ChunkMeshData),
}

pub struct ChunkGenPipeline {
    request_tx: Sender<PipelineTask>,
    result_rx: Receiver<PipelineResult>,
    #[cfg(target_arch = "wasm32")]
    inline: inline::InlineRunner,
}

impl ChunkGenPipeline {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(density_sampler: WorldSampler, threads: usize) -> Self {
        assert!(threads > 0, "at least one terrain worker is required");
        let (request_tx, request_rx) = crossbeam_channel::unbounded();
        let (result_tx, result_rx) = crossbeam_channel::unbounded();

        for id in 0..threads {
            let request_rx = request_rx.clone();
            let result_tx = result_tx.clone();
            let density_sampler = density_sampler.clone();

            std::thread::Builder::new()
                .name(format!("terrain-{id}"))
                .spawn(move || {
                    while let Ok(task) = request_rx.recv() {
                        let result = match task {
                            PipelineTask::Generate(GenTask(coords)) => {
                                let chunk = Chunk::from_sampler(coords, &density_sampler);
                                PipelineResult::Generate(Box::new(chunk))
                            }
                            PipelineTask::Mesh(mesh_task) => {
                                let MeshTask { chunk, neighbours } = *mesh_task;
                                let mesh_data = mesh_chunk(&chunk, &neighbours);
                                PipelineResult::Mesh(chunk.coords, mesh_data)
                            }
                        };

                        // Dropping the world disconnects the receiver and releases workers.
                        if result_tx.send(result).is_err() {
                            break;
                        }
                    }
                })
                .expect("failed to start terrain worker");
        }

        Self {
            request_tx,
            result_rx,
        }
    }

    pub fn request_generate(&self, coords: IVec3) {
        self.request_tx
            .send(PipelineTask::Generate(GenTask(coords)))
            .expect("Chunk channel shut down!");
    }

    pub fn request_mesh(&self, chunk: Arc<Chunk>, neighbours: NeighbourSnapshot) {
        self.request_tx
            .send(PipelineTask::Mesh(Box::new(MeshTask { chunk, neighbours })))
            .expect("Chunk channel shut down!");
    }

    pub fn result_channel(&self) -> &Receiver<PipelineResult> {
        &self.result_rx
    }
}
