use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use glam::IVec3;

use crate::{constants::GEN_THREADS, meshing::ChunkMeshData, world::chunk_manager::ChunkManager};

pub struct ChunkGenPipeline {
    // takes a chunk position and whether it's already generated
    // if its already generated it just needs remeshing
    request_tx: Sender<IVec3>,
    result_rx: Receiver<(IVec3, ChunkMeshData)>,
}

impl ChunkGenPipeline {
    pub fn create(chunk_man: Arc<ChunkManager>) -> Self {
        let (request_tx, request_rx) = crossbeam_channel::unbounded();
        let (result_tx, result_rx) = crossbeam_channel::unbounded();

        for _ in 0..GEN_THREADS {
            let (request_rx, result_tx) = (request_rx.clone(), result_tx.clone());
            let chunk_man = chunk_man.clone();

            std::thread::spawn(move || {
                while let Ok(chunk_pos) = request_rx.recv() {
                    let chunk = chunk_man.get_or_generate_chunk(&chunk_pos);
                    let mesh = chunk_man.mesh_chunk(&chunk);
                    result_tx.send((chunk_pos, mesh)).ok();
                }
                println!("Chunk thread shut down!");
            });
        }

        Self {
            request_tx,
            result_rx,
        }
    }

    pub fn request(&self, chunk_pos: IVec3) {
        self.request_tx
            .send(chunk_pos)
            .expect("Chunk channel shut down!");
    }

    pub fn result(&self) -> &Receiver<(IVec3, ChunkMeshData)> {
        &self.result_rx
    }
}
