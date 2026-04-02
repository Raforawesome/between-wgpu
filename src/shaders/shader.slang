struct VertexInput {
    float3 position : POSITION; // user defined semantics are given
    float3 color : COLOR;  // an index in order of declaration
}

struct VertexOutput {
    float4 clip_position : SV_Position;
    float3 color: COLOR;
}

[shader("vertex")]
VertexOutput vs_main(VertexInput vert) {
    VertexOutput out;

    out.color = vert.color;
    out.clip_position = float4(vert.position, 1.0);

    return out;
}

[shader("fragment")]
float4 fs_main(VertexOutput in) : SV_Target {
    return float4(in.color, 1.0);
}
