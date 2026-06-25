use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use glam::IVec3;

use crate::{
    meshing::{ChunkMeshData, mesh_chunk},
    world::{WorldSampler, chunk_manager::NeighbourSnapshot, chunks::Chunk},
};

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
}

impl ChunkGenPipeline {
    pub fn new(density_sampler: WorldSampler, threads: usize) -> Self {
        let (request_tx, request_rx) = crossbeam_channel::unbounded();
        let (result_tx, result_rx) = crossbeam_channel::unbounded();

        for _ in 0..threads {
            let request_rx = request_rx.clone();
            let result_tx = result_tx.clone();
            let density_sampler = density_sampler.clone();

            std::thread::spawn(move || {
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

                    result_tx.send(result).expect("Chunk channel shut down!");
                }
                println!("Chunk thread shut down!");
            });
        }

        Self {
            request_tx,
            result_rx,
        }
    }

    pub fn request(&self, task: PipelineTask) {
        self.request_tx
            .send(task)
            .expect("Chunk channel shut down!");
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
