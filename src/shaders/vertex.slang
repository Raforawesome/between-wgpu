module vertex;

import bind_groups;

/*
 * Vertex Shader
*/
struct VertexInput {
    float3 position : POSITION; // user defined semantics are given an index in order of declaration
    float3 tex_coords : UV;
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
