/*
 * Configurable Constants
 */

// chunks
pub const CHUNK_SIZE: usize = 16;

// world gen
pub const BASE_HEIGHT: f32 = 80.0;
pub const SEA_LEVEL: f32 = 72.0;
pub const FREQ: f32 = 1.0 / 150.0;
pub const AMPLITUDE: f32 = 1.0;
pub const HEIGHT_GRADIENT: f32 = 25.0;

// texture atlas (set to actual values of atlas)
//
// TODO: replace assets directory with a list of individual textures and generate
// atlas at runtime
pub const ATLAS_SIZE: usize = 4;
pub const TILE_SIZE: usize = 16;

// chunk rendering pipeline settings
pub const VIEW_DISTANCE: i32 = 12;
pub const VIEW_DISTANCE_Y: i32 = 4;
pub const DESPAWN_DISTANCE: i32 = 14;
pub const DESPAWN_DISTANCE_Y: i32 = 6;
pub const MAX_SPAWNS_PER_FRAME: usize = 256;

/*
 * Derived Constants
 * Calculated from configurable constants
 */

// chunk system
pub const CHUNK_SIZEI: i32 = CHUNK_SIZE as i32;
pub const CHUNK_VOL: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
pub const CHUNK_IDX: u8 = CHUNK_SIZE as u8 - 1;

// chunk cursor offsets for array traversal
pub const Y_INC: usize = CHUNK_SIZE * CHUNK_SIZE;
pub const Z_INC: usize = Y_INC * CHUNK_SIZE;
pub const Y_INC_IDX: usize = Y_INC - 1;
pub const Z_INC_IDX: usize = Z_INC - 1;

// texture atlas
pub const TILE_STEP: f32 = 1.0 / ATLAS_SIZE as f32;
pub const ATLAS_DIM: usize = TILE_SIZE * ATLAS_SIZE;
