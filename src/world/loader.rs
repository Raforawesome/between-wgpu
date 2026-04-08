use glam::IVec3;

use crate::HashSet;

const RENDER_DISTANCE: usize = 16;

#[derive(Default)]
pub struct ChunkLoader {
    idx: i32,
    center: IVec3,
    loaded: HashSet<IVec3>,
}
