struct VertexOutput {
    float4 clip_position : SV_Position;
}

[shader("vertex")]
VertexOutput vs_main(uint in_vertex_idx : SV_VertexID) {
    VertexOutput out;

    float x = float(1 - int(in_vertex_idx)) * 0.5;
    float y = float(int(in_vertex_idx & 1u) * 2 - 1) * 0.5;

    out.clip_position = float4(x, y, 0.0, 1.0);
    return out;
}

[shader("fragment")]
float4 fs_main(VertexOutput in) : SV_Target {
    return float4(0.3, 0.2, 0.1, 1.0);
}
