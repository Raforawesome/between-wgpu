pub mod block;
pub mod chunk_manager;
pub mod chunks;
pub mod loader;
pub mod pipeline;
pub mod player;

use chunk_manager::ChunkManager;
use glam::IVec3;
use loader::ChunkLoader;
use player::{Player, PlayerController};
use std::sync::Arc;

use crate::{
    HashSet,
    render::RenderState,
    world::{chunks::Chunk, pipeline::ChunkGenPipeline},
    worldgen::simplex::SimplexGen,
};

use winit::{event::ElementState, keyboard::KeyCode};

/// Holds state for the game world, such as chunks and the player.
/// This is kept separate from the [RenderState].
pub struct WorldState {
    pub chunk_man: Arc<ChunkManager<SimplexGen>>,
    pub loader: ChunkLoader,
    pub player: Player,
    pub controller: PlayerController,
    pub chunk_pipeline: ChunkGenPipeline,
    pub queued: HashSet<IVec3>,
    pub dirty: HashSet<IVec3>,
}

impl Default for WorldState {
    fn default() -> Self {
        let seed = fastrand::i32(..);
        let chunk_man = Arc::new(ChunkManager::new_with_seed(seed));

        Self {
            chunk_pipeline: ChunkGenPipeline::create(chunk_man.clone()),
            chunk_man,
            loader: ChunkLoader::default(),
            player: Player::new((0., 90., -2.).into(), 0.0, 0.0),
            controller: PlayerController::default(),
            queued: HashSet::default(),
            dirty: HashSet::default(),
        }
    }
}

impl WorldState {
    /// Update the game state, optionally based on the last frame's render state.
    pub fn update(&mut self, render_state: &mut RenderState) {
        // update player position based on player controller
        self.controller.update_player(&mut self.player);

        // update loaded chunks based on player position
        self.loader.update(self.player.position);

        // process results first to allow dirty chunks to be re-queued in the same frame
        // and to remove empty meshes from the gpu world
        for (pos, mesh_data) in self.chunk_pipeline.result().try_iter() {
            self.queued.remove(&pos);

            // if mesh is empty
            if mesh_data.vertices.is_empty() || mesh_data.indices.is_empty() {
                // remove the chunk's buffer if it exists and we determine it's empty
                // this prevents chunks that are remeshed, and now have no visible faces,
                // from sticking around
                render_state.chunk_buffers_mut().remove(&pos);
                continue;
            }

            if let Some(chunk) = self.chunk_man.get_chunk(&pos) {
                let buffers = render_state.gpu.create_chunk_buffers(
                    pos,
                    mesh_data,
                    chunk.blocks(),
                    render_state.blocks_group_layout(),
                );
                render_state.chunk_buffers_mut().insert(pos, buffers);
            }
        }

        // stop queueing chunks if we can no longer see them
        self.queued.retain(|pos| self.loader.contains(pos));
        // clean up dirty flags for chunks that are out of range
        self.dirty.retain(|pos| self.loader.contains(pos));

        // send necessary chunks through the gen & meshing pipeline
        for chunk_pos in self.loader.loaded_chunks() {
            // make sure chunk isn't already in-flight
            if !self.queued.contains(chunk_pos) {
                // if this is the first time generating the chunk, mark neighbours as dirty
                if self.chunk_man.get_chunk(chunk_pos).is_none() {
                    self.queued.insert(*chunk_pos);
                    self.dirty.extend(Chunk::neighbours_pos_list_at(*chunk_pos));
                    // don't remove self from dirty yet, ensure we remesh after neighbors load
                    self.chunk_pipeline.request(*chunk_pos);
                }
                // otherwise if the chunk is being remeshed
                else if self.dirty.contains(chunk_pos) {
                    self.queued.insert(*chunk_pos);
                    self.dirty.remove(chunk_pos);
                    self.chunk_pipeline.request(*chunk_pos);
                }
            }
        }
    }

    /// Handle mouse movement for camera rotation.
    pub fn handle_mouse_motion(&mut self, (mouse_dx, mouse_dy): (f64, f64)) {
        self.controller
            .process_mouse(mouse_dx, mouse_dy, &mut self.player);
    }

    /// Handles all key press inputs relating to the game state, such as movement.
    pub fn handle_input(&mut self, code: KeyCode, key_state: ElementState) -> bool {
        let is_pressed = key_state.is_pressed();
        self.controller.process_events(code, is_pressed)
    }
}
