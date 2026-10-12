use glam::IVec3;

use crate::{
    constants::{CHUNK_SIZE, CHUNK_SIZEI, CHUNK_VOL},
    world::block::BlockType,
};

pub mod interpolate;
pub mod layered;

/// A trait which represents any struct that lets you sample a density value
/// for an (x, y, z) coordinate. This could be a generator or an interpolator
/// wrapping a generator. `'static` here implies that the implementing member
/// should consist of 'static references and owned data.
pub trait DensitySampler: Send + Sync + Clone + 'static {
    fn sample_density(&self, x: f32, y: f32, z: f32) -> f32;

    fn sample_chunk_densities(&self, chunk_coords: IVec3) -> [f32; CHUNK_VOL] {
        let mut densities = [0.0; CHUNK_VOL];
        let origin = chunk_coords * CHUNK_SIZEI;

        for z in 0..CHUNK_SIZE {
            let z_idx = z * CHUNK_SIZE * CHUNK_SIZE;

            for y in 0..CHUNK_SIZE {
                let y_idx = y * CHUNK_SIZE;

                for x in 0..CHUNK_SIZE {
                    let world_pos = origin + glam::IVec3::new(x as i32, y as i32, z as i32);
                    densities[x + y_idx + z_idx] = self.sample_density(
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
