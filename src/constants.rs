/*
 * Configurable Constants
 */

// player
pub const BASE_SPEED: f32 = 100.0;
pub const SPRINT_MULTI: f32 = 3.0;
pub const PLAYER_RADIUS: f32 = 0.3;
pub const PLAYER_EYE_HEIGHT: f32 = 1.62;
pub const PLAYER_FOREHEAD: f32 = 0.18;

// chunks
pub const CHUNK_SIZE: usize = 16;
// The u16 occupancy rows and fragment shader's 0..15 voxel clamp assume this size.
const _: () = assert!(CHUNK_SIZE == 16);
pub type ChunkMask = [[u16; CHUNK_SIZE]; CHUNK_SIZE];

// chunk rendering pipeline settings
pub const VIEW_DISTANCE: i32 = 8;
pub const VIEW_DISTANCE_Y: i32 = 4;

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
