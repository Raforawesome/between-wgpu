use glam::IVec3;

use crate::{
    constants::{CHUNK_SIZE, CHUNK_SIZEI, CHUNK_VOL},
    world::block::BlockType,
};

pub mod interpolate;
pub mod simplex;

/// A trait which represents any struct that lets you sample a density value
/// for an (x, y, z) coordinate. This could be a generator or an interpolator
/// wrapping a generator. `'static` here implies that the implementing member
/// should consist of 'static references and owned data.
pub trait DensitySampler: Default + Send + Sync + Clone + 'static {
    fn with_seed(seed: i32) -> Self;

    fn sample_density(&self, x: f32, y: f32, z: f32) -> f32;

    fn sample_chunk_densities(&self, chunk_coords: IVec3) -> [f32; CHUNK_VOL] {
        let mut densities = [0.0; CHUNK_VOL];
        let origin = chunk_coords * CHUNK_SIZEI;

        for x in 0..CHUNK_SIZEI {
            for y in 0..CHUNK_SIZEI {
                for z in 0..CHUNK_SIZEI {
                    let idx =
                        x as usize + y as usize * CHUNK_SIZE + z as usize * CHUNK_SIZE * CHUNK_SIZE;
                    let world_pos = origin + IVec3::new(x, y, z);
                    densities[idx] = self.sample_density(
                        world_pos.x as f32,
                        world_pos.y as f32,
                        world_pos.z as f32,
                    );
                }
            }
        }

        densities
    }

    fn density_to_block_type(&self, density: f32, y: f32) -> BlockType;

    fn get_block_type(&self, x: f32, y: f32, z: f32) -> BlockType {
        let density = self.sample_density(x, y, z);
        self.density_to_block_type(density, y)
    }
}
