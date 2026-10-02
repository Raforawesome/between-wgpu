//! Conservative AABB culling for wgpu's 0..w clip depth and infinite reversed Z.
use crate::constants::CHUNK_SIZEI;
use glam::{IVec3, Mat4, Vec3, Vec4};

pub struct Frustum {
    planes: [Vec4; 5],
}
impl Frustum {
    pub fn from_view_projection(matrix: Mat4) -> Self {
        let x = matrix.row(0);
        let y = matrix.row(1);
        let z = matrix.row(2);
        let w = matrix.row(3);
        // x/y lie in [-w,w]. Near is z <= w. With infinite reversed Z,
        // z >= 0 has no finite far plane; normalizing it would divide by zero.
        Self {
            planes: [w + x, w - x, w + y, w - y, w - z],
        }
    }

    pub fn intersects_aabb(&self, min: Vec3, max: Vec3) -> bool {
        let center = (min + max) * 0.5;
        let extent = (max - min) * 0.5;
        self.planes
            .iter()
            .all(|p| p.truncate().dot(center) + p.w + p.truncate().abs().dot(extent) >= -0.001)
    }

    pub fn contains_chunk(&self, coords: IVec3) -> bool {
        let min = coords.as_vec3() * CHUNK_SIZEI as f32;
        self.intersects_aabb(min, min + Vec3::splat(CHUNK_SIZEI as f32))
    }
}
