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
