use glam::IVec3;

use crate::{
    constants::{CHUNK_SIZE, CHUNK_SIZEI, CHUNK_VOL},
    world::{block::BlockType, chunks::linearize_xyz},
};

use super::DensitySampler;

/// Newtype which enforces compile-time invariants on SAMPLE_STEP.
struct SampleStep(usize);

impl SampleStep {
    const fn new(step: usize) -> Self {
        assert!(step > 0);
        assert!(CHUNK_SIZE.is_multiple_of(step));
        Self(step)
    }
}

/// The factor of upsampling to perform.
const SAMPLE_STEP: usize = SampleStep::new(4).0;
// pre-calculate reciprocal because multiplying is faster than dividing
const SAMPLE_RECIP: f32 = 1.0 / SAMPLE_STEP as f32;

/// The total size of the 3D coarse samples array.
const GRID_SIZE: usize = CHUNK_SIZE / SAMPLE_STEP + 1;
const GRID_VOL: usize = GRID_SIZE * GRID_SIZE * GRID_SIZE;

/// Wraps an `impl DensitySampler` and performs trilinear interpolation to generate chunk
/// densities when sampled on a per-chunk basis. When sampled on a per-block basis, the
/// underlying sampler is simply used directly (meaning this wrapper has no effect in this case).
#[derive(Default, Clone)]
pub struct Trilerp<S: DensitySampler>(S);

/// Linearizes a 3D grid coordinate to a 1D array coordinate.
#[inline]
fn grid_idx(x: usize, y: usize, z: usize) -> usize {
    x + y * GRID_SIZE + z * GRID_SIZE * GRID_SIZE
}

/// Performs linear interpolation between two values (`a` and `b`) by a "time" factor `t` in [0.0, 1.0]
#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

impl<S: DensitySampler> DensitySampler for Trilerp<S> {
    fn with_seed(seed: i32) -> Self {
        Self(S::with_seed(seed))
    }

    // There's an argument to be made that this function should actually use interpolation
    // for this even though it would be 8x slower than not doing that, simply for consistency
    // between the name and behaviour of this trait. However, I (currently) think that's stupid.
    fn sample_density(&self, x: f32, y: f32, z: f32) -> f32 {
        self.0.sample_density(x, y, z)
    }

    fn density_to_block_type(&self, density: f32, y: f32) -> BlockType {
        self.0.density_to_block_type(density, y)
    }

    fn sample_chunk_densities(&self, chunk_coords: IVec3) -> [f32; CHUNK_VOL] {
        let origin = chunk_coords * CHUNK_SIZEI;
        let mut samples = [0.0; GRID_VOL];
        let mut densities = [0.0; CHUNK_VOL];

        // fill up the coarse sample grid to use for interpolation
        for gz in 0..GRID_SIZE {
            for gy in 0..GRID_SIZE {
                for gx in 0..GRID_SIZE {
                    // step through the world in increments of SAMPLE_STEP
                    let world_x = origin.x + (gx * SAMPLE_STEP) as i32;
                    let world_y = origin.y + (gy * SAMPLE_STEP) as i32;
                    let world_z = origin.z + (gz * SAMPLE_STEP) as i32;

                    samples[grid_idx(gx, gy, gz)] =
                        self.sample_density(world_x as f32, world_y as f32, world_z as f32);
                }
            }
        }

        // interpolate the coarse sample grid into per-block densities
        for z in 0..CHUNK_SIZE {
            // calculate the nearest (floored) grid index for this block
            let gz = z / SAMPLE_STEP;
            // calculate how far along grid index is to the next one
            let tz = (z % SAMPLE_STEP) as f32 * SAMPLE_RECIP;

            for y in 0..CHUNK_SIZE {
                let yz = y / SAMPLE_STEP;
                let ty = (y % SAMPLE_STEP) as f32 * SAMPLE_RECIP;

                for x in 0..CHUNK_SIZE {
                    let xz = x / SAMPLE_STEP;
                    let tx = (x % SAMPLE_STEP) as f32 * SAMPLE_RECIP;

                    // first we get the 8 closest vertices in the coarse grid
                    // numbered in counter-clockwise order (bottom face first)
                    let v0 = samples[grid_idx(xz, yz, gz)];
                    let v1 = samples[grid_idx(xz + 1, yz, gz)];
                    let v2 = samples[grid_idx(xz + 1, yz, gz + 1)];
                    let v3 = samples[grid_idx(xz, yz, gz + 1)];
                    let v4 = samples[grid_idx(xz, yz + 1, gz)];
                    let v5 = samples[grid_idx(xz + 1, yz + 1, gz)];
                    let v6 = samples[grid_idx(xz + 1, yz + 1, gz + 1)];
                    let v7 = samples[grid_idx(xz, yz + 1, gz + 1)];

                    // now we perform 4 interpolations to obtain thr 4 edges of the xy plane
                    // our target point lies within
                    let x0 = lerp(v0, v1, tx);
                    let x1 = lerp(v3, v2, tx);
                    let x2 = lerp(v4, v5, tx);
                    let x3 = lerp(v7, v6, tx);

                    // now we interpolate along the y axis to find the line along the x axis
                    // our target point lies on
                    let y0 = lerp(x0, x2, ty);
                    let y1 = lerp(x1, x3, ty);

                    // finally, we do one more interpolation on the z axis to find our target point
                    densities[linearize_xyz(x, y, z)] = lerp(y0, y1, tz);
                }
            }
        }

        densities
    }
}
