use noise_functions::Noise;

use crate::constants::{AMPLITUDE, BASE_HEIGHT, FREQ, HEIGHT_GRADIENT, SEA_LEVEL};
use crate::world::block::BlockType;

pub fn get_y(x: f32, z: f32) -> f32 {
    let noise = noise_functions::Simplex
        .fbm(3, 0.5, 2.0)
        .seed(42)
        .frequency(FREQ)
        .sample2([x, z]);
    (noise * AMPLITUDE + BASE_HEIGHT).floor()
}

pub fn sample_density(x: f32, y: f32, z: f32) -> f32 {
    let noise = noise_functions::OpenSimplex2s
        .fbm(3, 0.5, 2.0)
        .seed(42)
        .frequency(FREQ)
        .sample3([x, y, z]);
    (noise * AMPLITUDE) - (y - BASE_HEIGHT) / HEIGHT_GRADIENT
}

pub fn density_to_block_type(density: f32, y: f32) -> BlockType {
    match density {
        0.00..0.05 => BlockType::Grass,
        0.05..0.20 => BlockType::Dirt,
        0.20.. => BlockType::Stone,
        _ => {
            if y < SEA_LEVEL {
                BlockType::Water
            } else {
                BlockType::Air
            }
        }
    }
}

pub fn get_block_type(x: f32, y: f32, z: f32) -> BlockType {
    let density = sample_density(x, y, z);
    density_to_block_type(density, y)
}
