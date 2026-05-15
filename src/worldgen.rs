use crate::world::block::BlockType;

pub mod interpolate;
pub mod simplex;

/// A trait which represents any struct that lets you sample a density value
/// for an (x, y, z) coordinate. This could be a generator or an interpolator
/// wrapping a generator. `'static` here implies that the implementing member
/// should consist of 'static references and owned data.
pub trait DensitySampler: Default + Send + Sync + Clone + 'static {
    fn with_seed(seed: i32) -> Self;

    fn sample_density(&self, x: f32, y: f32, z: f32) -> f32;
    fn density_to_block_type(&self, density: f32, y: f32) -> BlockType;

    fn get_block_type(&self, x: f32, y: f32, z: f32) -> BlockType {
        let density = self.sample_density(x, y, z);
        self.density_to_block_type(density, y)
    }
}
