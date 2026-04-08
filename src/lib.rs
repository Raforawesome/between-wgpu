use rapidhash::fast::RandomState;

pub mod constants;
pub mod meshing;
pub mod render;
pub mod world;

use crate::world::chunk_manager::ChunkManager;

pub type HashSet<K> = collections::HashSet<K, RandomState>;
pub type HashMap<K, V> = collections::HashMap<K, V, RandomState>;
pub type DashMap<K, V> = dashmap::DashMap<K, V, RandomState>;

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
    pub fn update(&mut self) {
        self.controller.update_player(&mut self.player);
    }

    pub fn handle_mouse_motion(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.controller
            .process_mouse(mouse_dx, mouse_dy, &mut self.player);
    }
}
