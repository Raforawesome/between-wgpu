use std::collections;

use rapidhash::fast::RandomState;

pub mod constants;
pub mod meshing;
pub mod render;
pub mod world;
pub mod worldgen;

use world::WorldState;

use crate::render::RenderState;

// create rapidhash type aliases for hashed collections to increase performance
pub type HashSet<K> = collections::HashSet<K, RandomState>;
pub type HashMap<K, V> = collections::HashMap<K, V, RandomState>;
pub type DashMap<K, V> = dashmap::DashMap<K, V, RandomState>;

/// This function runs the main update loop for the game. It updates
/// both the game and render state, then renders a frame.
pub fn run_update_loop(
    game_state: &mut WorldState,
    render_state: &mut RenderState,
) -> Result<(), anyhow::Error> {
    // update game state based on last frame's render state
    game_state.update(render_state);
    // update render state based on new game state
    render_state.update(game_state);

    // draw new render state
    render_state.render(game_state)
}
