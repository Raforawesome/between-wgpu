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
    worldgen::{interpolate::Trilerp, layered::LayeredGen},
};

use winit::{event::ElementState, keyboard::KeyCode};

pub type WorldSampler = Trilerp<LayeredGen>;

/// Holds state for the game world, such as chunks and the player.
/// This is kept separate from the [RenderState].
pub struct WorldState {
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
        Self::new(
            42,
            crate::constants::VIEW_DISTANCE,
            crate::constants::VIEW_DISTANCE_Y,
        )
    }
}

impl WorldState {
    /// Creates a world with reproducible generation and configurable view distances.
    pub fn new(seed: i32, horizontal: i32, vertical: i32) -> Self {
        let density_sampler = Trilerp::new(LayeredGen::new(seed));
        // Use half the available cores, but always start at least one worker.
        let gen_threads = std::thread::available_parallelism()
            .map(|n| (n.get() / 2).max(1))
            .unwrap_or(1);
        let chunk_pipeline = ChunkGenPipeline::new(density_sampler, gen_threads);

        Self {
            chunk_man: ChunkManager::new(),
            chunk_pipeline,
            loader: ChunkLoader::new(horizontal, vertical),
            player: Player::new((0., 90., -2.).into(), 0.0, 0.0),
            controller: PlayerController::default(),
            queued: HashSet::default(),
            dirty: HashSet::default(),
        }
    }

    /// Update the game state, optionally based on the last frame's render state.
    pub fn update(&mut self, render_state: &mut RenderState, dt_seconds: f32) {
        // update player position based on player controller
        self.controller.update_player(&mut self.player, dt_seconds);

        // update loaded chunks based on player position
        if self.loader.update(self.player.eye) {
            // Membership only changes when the player crosses a chunk boundary.
            render_state.cache_chunks_outside_view(&self.loader);
            self.queued.retain(|pos| self.loader.contains(pos));
        }

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
                        render_state.remove_chunk_buffer(&coords);
                        continue;
                    }

                    if let Some(chunk) = self.chunk_man.get_chunk(&coords) {
                        let buffers = render_state.gpu.create_chunk_buffers(
                            coords,
                            mesh_data,
                            chunk.blocks(),
                            render_state.blocks_group_layout(),
                        );

                        if self.loader.contains(&coords) {
                            render_state.insert_chunk_buffer(coords, buffers);
                        } else {
                            render_state.cache_chunk_buffer(coords, buffers);
                        }
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
            if render_state.has_chunk_buffer(chunk_pos) {
                continue;
            }

            if render_state.restore_chunk_buffer_from_cache(chunk_pos) {
                continue;
            }

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
