pub mod block;
pub mod chunk_manager;
pub mod chunks;
pub mod loader;
pub mod pipeline;
pub mod player;
pub mod worldgen;

use chunk_manager::ChunkManager;
use glam::IVec3;
use loader::ChunkLoader;
use player::{Player, PlayerController};
use std::sync::Arc;

use crate::{
    HashSet, meshing::ChunkMeshData, render::RenderState, world::pipeline::ChunkGenPipeline,
};

use winit::{event::ElementState, keyboard::KeyCode};

/// Holds state for the game world, such as chunks and the player.
/// This is kept separate from the [RenderState].
pub struct WorldState {
    pub chunk_man: Arc<ChunkManager>,
    pub loader: ChunkLoader,
    pub player: Player,
    pub controller: PlayerController,
    pub chunk_pipeline: ChunkGenPipeline,
    pub queued: HashSet<IVec3>,
    pub dirty: HashSet<IVec3>,
}

impl Default for WorldState {
    fn default() -> Self {
        let chunk_man = Arc::new(ChunkManager::default());
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

        // stop queueing chunks if we can no longer see them
        self.queued.retain(|pos| self.loader.contains(pos));

        // send necessary chunks through the gen & meshing pipeline
        for chunk_pos in self.loader.loaded_chunks() {
            // if the chunk isn't already queued, and it either doesn't exist or needs remeshing
            if !self.queued.contains(chunk_pos)
                && (self.chunk_man.get_chunk(chunk_pos).is_none() || self.dirty.contains(chunk_pos))
            {
                self.queued.insert(*chunk_pos);
                self.chunk_pipeline.request(*chunk_pos);
            }
        }

        // update chunk buffers in the gpu/render world
        for (pos, mesh_data) in self.chunk_pipeline.result().try_iter() {
            let ChunkMeshData { vertices, indices } = mesh_data;

            // skip empty data to avoid gpu crashes (gpu buffers can't be empty)
            if vertices.is_empty() || indices.is_empty() {
                continue;
            }

            let buffers = render_state.gpu.create_chunk_buffers(vertices, indices);
            let chunk_buffers = render_state.chunk_buffers_mut();
            chunk_buffers.insert(pos, buffers);
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
