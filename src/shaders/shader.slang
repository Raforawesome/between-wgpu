/*
 * Bind Groups
*/
// Bind group 0: diffuse bind group
struct TexBindGroup {
    [format("rgba8")] // equivalent to wgpu's Rgba8UnormSrgb
    Texture2DArray textures; // binding 0 contains a 2d texture array (3d texture)
    SamplerState sampler; // binding 1 contains the sampler
}

// Bind group 1: camera bind group
struct Camera {
    float4x4 view_proj; // binding 0 contains camera matrix
}

// declare bind groups in incrementing order
ParameterBlock<TexBindGroup> bg; // bind group 0
ParameterBlock<Camera> camera;   // bind group 1

/*
 * Vertex Shader
*/
struct VertexInput {
    float3 position : POSITION; // user defined semantics are given an index in order of declaration
    float3 tex_coords : UV;
}

struct VertexOutput {
    float4 clip_position : SV_Position;
    float3 tex_coords : UV;
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
