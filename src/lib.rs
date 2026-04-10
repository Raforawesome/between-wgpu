use std::collections;

use rapidhash::fast::RandomState;
use winit::{event::ElementState, keyboard::KeyCode};

pub mod constants;
pub mod meshing;
pub mod render;
pub mod world;

use crate::{
    render::RenderState,
    world::{
        chunk_manager::ChunkManager,
        loader::ChunkLoader,
        player::{Player, PlayerController},
    },
};

// create rapidhash type aliases for hashed collections to increase performance
pub type HashSet<K> = collections::HashSet<K, RandomState>;
pub type HashMap<K, V> = collections::HashMap<K, V, RandomState>;
pub type DashMap<K, V> = dashmap::DashMap<K, V, RandomState>;

/// This function runs the main update loop for the game. It updates
/// both the game and render state, then renders a frame.
pub fn run_update_loop(
    game_state: &mut GameState,
    render_state: &mut RenderState,
) -> Result<(), anyhow::Error> {
    // update game state based on last frame's render state
    game_state.update(render_state);
    // update render state based on new game state
    render_state.update(game_state);

    // draw new render state
    render_state.render()
}

/// Holds state for the game world, such as chunks and the player.
/// This is kept separate from the [RenderState].
pub struct GameState {
    pub chunk_man: ChunkManager,
    pub loader: ChunkLoader,
    pub player: Player,
    pub controller: PlayerController,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            chunk_man: ChunkManager::default(),
            loader: ChunkLoader::default(),
            player: Player::new((0., 90., -2.).into(), 0.0, 0.0),
            controller: PlayerController::default(),
        }
    }
}

impl GameState {
    /// Update the game state, optionally based on the last frame's render state.
    pub fn update(&mut self, render_state: &mut RenderState) {
        self.controller.update_player(&mut self.player);
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
