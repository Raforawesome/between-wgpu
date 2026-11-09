use crate::world::block::BlockType;
use noise_functions::Noise;

pub const SEA_LEVEL: f32 = 71.0;

#[derive(Clone)]
pub struct LayeredGen {
    seed: i32,
}

impl LayeredGen {
    pub fn new(seed: i32) -> Self {
        Self { seed }
    }
}

// 2D terrain mapping helper functions
fn get_base_height(cont: f32) -> f32 {
    if cont < -0.15 {
        let t = (cont - (-1.0)) / 0.85;
        35.0 + t * 33.0
    } else if cont < 0.15 {
        let t = (cont - (-0.15)) / 0.30;
        68.0 + t * 8.0
    } else {
        let t = (cont - 0.15) / 0.85;
        76.0 + t * 44.0
    }
}

fn get_terrain_scale(ero: f32) -> f32 {
    let t = (ero + 1.0) / 2.0;
    35.0 * (1.0 - t) * (1.0 - t) + 3.0
}

fn get_peaks_valleys(weird: f32) -> f32 {
    1.0 - weird.abs()
}

impl super::DensitySampler for LayeredGen {
    fn sample_density(&self, x: f32, y: f32, z: f32) -> f32 {
        // continentalness (determines overall shape/water)
        let cont_f = 1.0 / 800.0;
        let c1 = noise_functions::OpenSimplex2
            .seed(self.seed)
            .frequency(cont_f)
            .sample2([x, z]);
        let c2 = noise_functions::OpenSimplex2
            .seed(self.seed.wrapping_add(50))
            .frequency(cont_f * 2.0)
            .sample2([x, z]);
        let cont = c1 + 0.5 * c2;

        // erosion (finer sculpting)
        let ero = noise_functions::OpenSimplex2
            .seed(self.seed.wrapping_add(100))
            .frequency(1.0 / 400.0)
            .sample2([x, z]);

        // weirdness / peaks & valleys (creates more dramatic variation)
        let weird = noise_functions::OpenSimplex2
            .seed(self.seed.wrapping_add(200))
            .frequency(1.0 / 250.0)
            .sample2([x, z]);

        let base_h = get_base_height(cont);
        let scale = get_terrain_scale(ero);
        let pv = get_peaks_valleys(weird);
        let target_h = base_h + pv * scale;

        // Base terrain density (proportional to distance from the surface)
        let base_density = (target_h - y) / 12.0;

        // if we are high above the detail zone, skip all 3D noise evaluations
        if y >= target_h + 8.0 {
            return base_density;
        }

        // if we are deep below the surface detail zone, skip all 3D noise evaluations
        if y <= target_h - 8.0 {
            return base_density;
        }

        // sample detail 3D noise only near the surface to add overhangs/arches
        let det_f = 1.0 / 80.0;
        let d1 = noise_functions::OpenSimplex2
            .seed(self.seed.wrapping_add(300))
            .frequency(det_f)
            .sample3([x, y, z]);
        let d2 = noise_functions::OpenSimplex2
            .seed(self.seed.wrapping_add(310))
            .frequency(det_f * 2.0)
            .sample3([x, y, z]);
        let detail = d1 + 0.5 * d2;

        base_density + detail * 0.4
    }

    fn density_to_block_type(&self, density: f32, y: f32) -> BlockType {
        const BEACH_MIN: f32 = SEA_LEVEL - 4.0;
        const BEACH_MAX: f32 = SEA_LEVEL + 3.0;

        if density < 0.0 {
            if y < SEA_LEVEL {
                BlockType::Water
            } else {
                BlockType::Air
            }
        } else if density < 0.08 {
            if (BEACH_MIN..=BEACH_MAX).contains(&y) {
                BlockType::Sand
            } else {
                BlockType::Grass
            }
        } else if density < 0.22 {
            if (BEACH_MIN..=BEACH_MAX).contains(&y) {
                BlockType::Sand
            } else {
                BlockType::Dirt
            }
        } else {
            BlockType::Stone
        }
    }
}
