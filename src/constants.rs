/*
 * Configurable Constants
 */

// player
pub const BASE_SPEED: f32 = 1.5;
pub const SPRINT_MULTI: f32 = 5.0;

// chunks
pub const CHUNK_SIZE: usize = 16;
pub type ChunkMask = [[u16; CHUNK_SIZE]; CHUNK_SIZE];

// world gen
pub const BASE_HEIGHT: f32 = 80.0;
pub const SEA_LEVEL: f32 = 72.0;
pub const FREQ: f32 = 1.0 / 150.0;
pub const AMPLITUDE: f32 = 1.0;
pub const HEIGHT_GRADIENT: f32 = 30.0;

// chunk rendering pipeline settings
pub const VIEW_DISTANCE: i32 = 16;
pub const VIEW_DISTANCE_Y: i32 = 4;
pub const MAX_SPAWNS_PER_FRAME: usize = 64;
pub const GEN_THREADS: usize = 2;

/*
 * Derived Constants
 * Calculated from configurable constants
 */

// chunk system
pub const CHUNK_SIZEI: i32 = CHUNK_SIZE as i32;
pub const CHUNK_SIZEU: u32 = CHUNK_SIZE as u32;
pub const CHUNK_VOL: usize = CHUNK_SIZE.pow(3);
pub const CHUNK_IDX: u8 = CHUNK_SIZE as u8 - 1;

// chunk cursor offsets for array traversal
pub const Y_INC: usize = CHUNK_SIZE * CHUNK_SIZE;
pub const Z_INC: usize = Y_INC * CHUNK_SIZE;
pub const Y_INC_IDX: usize = Y_INC - 1;
pub const Z_INC_IDX: usize = Z_INC - 1;
