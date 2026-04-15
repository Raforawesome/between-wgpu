pub mod block;
pub mod chunk_manager;
pub mod chunks;
pub mod loader;
pub mod player;
pub mod worldgen;

use chunk_manager::ChunkManager;
use loader::ChunkLoader;
use player::{Player, PlayerController};

use crate::render::RenderState;

use winit::{event::ElementState, keyboard::KeyCode};

/// Holds state for the game world, such as chunks and the player.
/// This is kept separate from the [RenderState].
pub struct WorldState {
    pub chunk_man: ChunkManager,
    pub loader: ChunkLoader,
    pub player: Player,
    pub controller: PlayerController,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            chunk_man: ChunkManager::default(),
            loader: ChunkLoader::default(),
            player: Player::new((0., 90., -2.).into(), 0.0, 0.0),
            controller: PlayerController::default(),
        }
    }
}

impl WorldState {
    /// Update the game state, optionally based on the last frame's render state.
    pub fn update(&mut self, _render_state: &mut RenderState) {
        // update player position based on player controller
        self.controller.update_player(&mut self.player);
        // update loaded chunks based on player position
        self.loader.update(self.player.position);
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
