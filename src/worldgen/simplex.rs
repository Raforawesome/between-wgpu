use std::sync::LazyLock;

use noise_functions::Noise;

pub const BASE_HEIGHT: f32 = 80.0;
pub const SEA_LEVEL: f32 = 72.0;
pub const FREQ: f32 = 1.0 / 150.0;
pub const AMPLITUDE: f32 = 1.0;
pub const HEIGHT_GRADIENT: f32 = 30.0;

use crate::world::block::BlockType;

static SEED: LazyLock<i32> = LazyLock::new(|| fastrand::i32(..));

#[derive(Default)]
pub struct SimplexGen {
    seed: i32,
}

impl super::DensitySampler for SimplexGen {
    fn with_seed(seed: i32) -> Self {
        Self { seed }
    }

    fn sample_density(&self, x: f32, y: f32, z: f32) -> f32 {
        let noise = noise_functions::OpenSimplex2s
            .fbm(3, 0.5, 2.0)
            .seed(*SEED)
            .frequency(FREQ)
            .sample3([x, y, z]);
        (noise * AMPLITUDE) - (y - BASE_HEIGHT) / HEIGHT_GRADIENT
    }

    fn density_to_block_type(&self, density: f32, y: f32) -> BlockType {
        match density {
            0.00..0.06 => BlockType::Grass,
            0.06..0.20 => BlockType::Dirt,
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
}
