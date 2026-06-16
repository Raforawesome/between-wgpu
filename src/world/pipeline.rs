use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use glam::IVec3;

use crate::{
    constants::GEN_THREADS,
    meshing::{ChunkMeshData, mesh_chunk},
    world::{
        WorldSampler,
        chunk_manager::NeighbourSnapshot,
        chunks::Chunk,
    },
};

pub const BATCH_SIZE: i32 = 8;

/// Sum type for pipeline tasks.
pub enum PipelineTask {
    Generate(GenTask),
    Mesh(Box<MeshTask>),
}

/// Wrapper struct which represents a generation task for a given chunk coordinate.
pub struct GenTask(pub IVec3);

pub struct MeshTask {
    chunk: Arc<Chunk>,
    neighbours: NeighbourSnapshot,
}

/// Represents the outcome of a pipeline task. Pipeline tasks can be one of:
/// - Generate
/// - Mesh
pub enum PipelineResult {
    Generate(Box<Chunk>),
    Mesh(IVec3, ChunkMeshData),
}

pub struct ChunkGenPipeline {
    // takes a chunk position and whether it's already generated
    // if its already generated it just needs remeshing
    request_tx: Sender<PipelineTask>,
    result_rx: Receiver<PipelineResult>,
}

impl ChunkGenPipeline {
    pub fn new(density_sampler: WorldSampler) -> Self {
        let (request_tx, request_rx) = crossbeam_channel::unbounded();
        let (result_tx, result_rx) = crossbeam_channel::unbounded();

        for _ in 0..GEN_THREADS {
            let request_rx = request_rx.clone();
            let result_tx = result_tx.clone();
            let density_sampler = density_sampler.clone();

            std::thread::spawn(move || {
                while let Ok(task) = request_rx.recv() {
                    let mut batch = Vec::with_capacity(BATCH_SIZE as usize);
                    let mut results = Vec::with_capacity(BATCH_SIZE as usize);

                    batch.push(task);
                    for _ in 0..BATCH_SIZE {
                        if let Ok(task) = request_rx.try_recv() {
                            batch.push(task);
                        } else {
                            break;
                        }
                    }

                    for task in batch {
                        let result = match task {
                            PipelineTask::Generate(GenTask(coords)) => {
                                let chunk = Chunk::from_sampler(
                                    coords,
                                    &density_sampler,
                                );
                                PipelineResult::Generate(Box::new(chunk))
                            }
                            PipelineTask::Mesh(mesh_task) => {
                                let MeshTask { chunk, neighbours } = *mesh_task;
                                let mesh_data = mesh_chunk(&chunk, &neighbours);
                                PipelineResult::Mesh(chunk.coords, mesh_data)
                            }
                        };

                        results.push(result);
                    }

                    for result in results {
                        result_tx.send(result).expect("Chunk channel shut down!");
                    }
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
