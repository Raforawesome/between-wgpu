use rapidhash::fast::RandomState;

pub mod constants;
pub mod meshing;
pub mod render;
pub mod world;

use crate::world::chunk_manager::ChunkManager;

pub type HashMap<K, V> = std::collections::HashMap<K, V, RandomState>;
pub type DashMap<K, V> = dashmap::DashMap<K, V, RandomState>;

pub struct GameState {
    pub chunk_man: ChunkManager,
}
