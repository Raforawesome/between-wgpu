pub mod constants;
pub mod meshing;
pub mod render;
pub mod world;

pub type HashMap<K, V> = std::collections::HashMap<K, V, rapidhash::fast::RandomState>;
pub type Dashmap<K, V> = dashmap::DashMap<K, V, rapidhash::fast::RandomState>;
