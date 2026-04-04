/*
 * Vertex Shader
*/
struct VertexInput {
    float3 position : POSITION; // user defined semantics are given
    float2 tex_coords : UV;  // an index in order of declaration
}

struct VertexOutput {
    float4 clip_position : SV_Position;
    float2 tex_coords: UV;
}

[shader("vertex")]
VertexOutput vs_main(VertexInput vert) {
    VertexOutput out;

    out.tex_coords = vert.tex_coords;
    out.clip_position = float4(vert.position, 1.0);

    return out;
}

/*
 * Fragment Shader
*/
struct TexBindGroup {
    [format("rgba8")] // equivalent to wgpu's Rgba8UnormSrgb
    Texture2DArray textures;
    SamplerState sampler;
}

ParameterBlock<TexBindGroup> bg;

[shader("fragment")]
float4 fs_main(VertexOutput in) : SV_Target {
    let tex_idx = 0;
    let tex_array = bg.textures;
    let sampler = bg.sampler;

    return tex_array.Sample(sampler, float3(in.tex_coords, tex_idx));
}
