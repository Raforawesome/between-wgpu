/*
 * Bind Groups
*/
// Bind group 0: diffuse bind group
public struct TexBindGroup {
    [format("rgba8")] // equivalent to wgpu's Rgba8UnormSrgb
    public Texture2DArray textures; // binding 0 contains a 2d texture array (3d texture)
    public SamplerState sampler; // binding 1 contains the sampler
}

// Bind group 1: camera bind group
public struct Camera {
    public float4x4 view_proj; // binding 0 contains camera matrix
}

// Bind group 2: per-chunk block-id texture and world position
public struct ChunkBlocks {
    public Texture3D<uint> blocks; // binding 0: chunk-local block ids
    public float3 world_pos;       // binding 1: chunk world origin
}

// declare bind groups in incrementing order
public ParameterBlock<TexBindGroup> bg; // bind group 0
public ParameterBlock<Camera> camera;   // bind group 1
public ParameterBlock<ChunkBlocks> chunk; // bind group 2

/*
 * Vertex Shader
*/
struct VertexInput {
    uint packed : PACKED;
}

public struct VertexOutput {
    float4 clip_position : SV_Position;
    public float3 local_pos : LOCAL_POS;
    public float2 face_uv : UV;
    public nointerpolation uint face_id : FACE_ID;
}

uint unpack_field(uint packed, uint shift, uint mask) {
    return (packed >> shift) & mask;
}

float2 uv_from_corner(uint uv_corner) {
    switch (uv_corner) {
        case 0: return float2(0.0, 1.0);
        case 1: return float2(1.0, 1.0);
        case 2: return float2(1.0, 0.0);
        default: return float2(0.0, 0.0);
    }
}

[shader("vertex")]
VertexOutput vs_main(VertexInput vert) {
    VertexOutput out;

    let x = unpack_field(vert.packed, 0, 0b11111);
    let y = unpack_field(vert.packed, 5, 0b11111);
    let z = unpack_field(vert.packed, 10, 0b11111);
    let uv_corner = unpack_field(vert.packed, 15, 0b11);
    let face_id = unpack_field(vert.packed, 17, 0b111);

    out.local_pos = float3(float(x), float(y), float(z));
    out.face_uv = uv_from_corner(uv_corner);
    out.face_id = face_id;

    let world_pos = chunk.world_pos + out.local_pos;
    out.clip_position = mul(camera.view_proj, float4(world_pos, 1.0));

    return out;
}

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
 * Maps a generic Block ID and a Face ID to a specific texture layer in the atlas.
 * This is the GPU-side equivalent of the old CPU-side BlockType::texture_layer logic.
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

/*
 * Fragment Shader
*/
[shader("fragment")]
float4 fs_main(VertexOutput in) : SV_Target {
    let tex_array = bg.textures;
    let sampler = bg.sampler;

    // calculate the 3D coordinate of the current block
    // apply 'face_bias' to nudge the coordinate inward, ensuring we don't
    // sample the wrong block due to floating-point error at face boundary
    let voxel_coords = in.local_pos + face_bias(in.face_id);
    let voxel = int3(clamp(floor(voxel_coords), 0.0, 15.0));

    // look up the raw Block ID from the per-chunk 3D blocktype texture
    let block_id = chunk.blocks.Load(int4(voxel, 0)).x; // .x is equivalent to .r (red)

    // resolve the specific texture layer index for this block type and face
    let layer = atlas_layer(block_id, in.face_id);

    // sample the color from the 2D Texture Array using the UVs and the resolved layer
    return tex_array.Sample(sampler, float3(in.face_uv, layer));
}
