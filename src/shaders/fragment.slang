/*
 * Fragment Shader
 */

float3 face_bias(uint face_id) {
    switch (face_id) {
        case 0: return float3(0.0, -0.001, 0.0);
        case 1: return float3(0.0, 0.001, 0.0);
        case 2: return float3(0.001, 0.0, 0.0);
        case 3: return float3(-0.001, 0.0, 0.0);
        case 4: return float3(0.0, 0.0, 0.001);
        default: return float3(0.0, 0.0, -0.001);
    }
}

/**
 * Maps a generic Block ID and Face ID to a specific texture layer in the atlas.
 */
int atlas_layer(uint block_id, uint face_id) {
    switch (block_id) {
        case 0: // Grass Block
            switch (face_id) {
                case 0: return 0;  // Top Face (Grass Texture)
                case 1: return 2;  // Bottom Face (Dirt Texture)
                default: return 1; // Side Faces (Grass-Side Texture)
            }
        case 1: return 2; // Dirt Block
        case 2: return 3; // Stone Block
        case 3: return 7; // Water Block
        case 5: return 4; // Sand Block
        case 6: return 5; // Log Block
        case 7: return 6; // Leaf Block
        default: return 0; // Fallback to Layer 0
    }
}

[shader("fragment")]
float4 fs_main(VertexOutput in) : SV_Target {
    let tex_array = bg.textures;
    let sampler = bg.sampler;

    // Nudge inward to avoid sampling the neighboring block at face boundaries.
    let voxel_coords = in.local_pos + face_bias(in.face_id);
    let voxel = int3(clamp(floor(voxel_coords), 0.0, 15.0));

    let block_id = chunk.blocks.Load(int4(voxel, 0)).x;
    let layer = atlas_layer(block_id, in.face_id);

    return tex_array.Sample(sampler, float3(in.face_uv, layer));
}
