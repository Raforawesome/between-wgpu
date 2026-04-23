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

// Bind group 2: per-chunk block-id texture
public struct ChunkBlocks {
    public Texture3D<uint> blocks; // binding 0 contains chunk-local block ids
}

// declare bind groups in incrementing order
public ParameterBlock<TexBindGroup> bg; // bind group 0
public ParameterBlock<Camera> camera;   // bind group 1
public ParameterBlock<ChunkBlocks> chunk; // bind group 2

/*
 * Vertex Shader
*/
struct VertexInput {
    float3 position : POSITION; // user defined semantics are given an index in order of declaration
    float3 local_pos : LOCAL_POS;
    float2 face_uv : UV;
    uint face_id : FACE_ID;
}

public struct VertexOutput {
    float4 clip_position : SV_Position;
    public float3 tex_coords : UV;
}

[shader("vertex")]
VertexOutput vs_main(VertexInput vert) {
    VertexOutput out;

    out.tex_coords = vert.tex_coords;
    out.clip_position = mul(camera.view_proj, float4(vert.position, 1.0));

    return out;
}

/*
 * Fragment Shader
*/
[shader("fragment")]
float4 fs_main(VertexOutput in) : SV_Target {
    let tex_array = bg.textures;
    let sampler = bg.sampler;

    return tex_array.Sample(sampler, in.tex_coords);
}
