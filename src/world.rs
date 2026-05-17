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

use crate::{
    HashSet,
    render::RenderState,
    world::pipeline::{ChunkGenPipeline, PipelineResult},
    worldgen::{DensitySampler, interpolate::Trilerp, simplex::SimplexGen},
};

use winit::{event::ElementState, keyboard::KeyCode};

pub type WorldSampler = Trilerp<SimplexGen>;

/// Holds state for the game world, such as chunks and the player.
/// This is kept separate from the [RenderState].
pub struct WorldState {
    pub density_sampler: WorldSampler,
    pub chunk_man: ChunkManager,
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

        let density_sampler = WorldSampler::with_seed(seed);
        let chunk_man = ChunkManager::new_with_sampler(density_sampler.clone());
        let chunk_pipeline = ChunkGenPipeline::new(density_sampler.clone());

        Self {
            density_sampler,
            chunk_man,
            chunk_pipeline,
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
    pub fn update(&mut self, render_state: &mut RenderState, dt_seconds: f32) {
        // update player position based on player controller
        self.controller.update_player(&mut self.player, dt_seconds);

        // update loaded chunks based on player position
        self.loader.update(self.player.position);

        // process results first to allow dirty chunks to be re-queued in the same frame
        // and to remove empty meshes from the gpu world
        for pipeline_result in self.chunk_pipeline.result_channel().try_iter() {
            match pipeline_result {
                PipelineResult::Generate(chunk) => {
                    let coords = chunk.coords();
                    self.chunk_man.insert_chunk(*chunk);
                    self.dirty.insert(coords);
                }
                PipelineResult::Mesh(coords, mesh_data) => {
                    // if mesh is empty
                    if mesh_data.vertices.is_empty() || mesh_data.indices.is_empty() {
                        // remove the chunk's buffer if it exists and we determine it's empty
                        // this prevents chunks that are remeshed, and now have no visible faces,
                        // from sticking around
                        render_state.chunk_buffers_mut().remove(&coords);
                        continue;
                    }

                    if let Some(chunk) = self.chunk_man.get_chunk(&coords) {
                        let buffers = render_state.gpu.create_chunk_buffers(
                            coords,
                            mesh_data,
                            chunk.blocks(),
                            render_state.blocks_group_layout(),
                        );
                        render_state.chunk_buffers_mut().insert(coords, buffers);
                    }
                }
            }
        }

        // stop queueing chunks if we can no longer see them
        self.queued.retain(|pos| self.loader.contains(pos));

        // queue all dirty chunks that are able to be meshed
        let meshable = self.dirty.extract_if(|pos| {
            self.chunk_man.get_chunk(pos).is_some() && self.chunk_man.has_neighbours(pos)
        });

        for pos in meshable {
            if let (Some(mesh), Some(neighbours)) = (
                self.chunk_man.get_chunk(&pos),
                self.chunk_man.neighbour_snapshot(&pos),
            ) {
                self.chunk_pipeline.request_mesh(mesh.clone(), neighbours);
            }
        }

        // send necessary chunks through the gen & meshing pipeline
        for chunk_pos in self.loader.loaded_chunks() {
            // make sure chunk isn't already in-flight
            if !self.queued.contains(chunk_pos) {
                self.queued.insert(*chunk_pos);
                self.chunk_pipeline.request_generate(*chunk_pos);
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
