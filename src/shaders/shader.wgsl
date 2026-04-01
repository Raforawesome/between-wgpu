struct VertexOutput_0
{
    @builtin(position) clip_position_0 : vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_idx_0 : u32) -> VertexOutput_0
{
    var out_0 : VertexOutput_0;
    out_0.clip_position_0 = vec4<f32>(f32(i32(1) - i32(in_vertex_idx_0)) * 0.5f, f32(i32((in_vertex_idx_0 & (u32(1)))) * i32(2) - i32(1)) * 0.5f, 0.0f, 1.0f);
    return out_0;
}

struct pixelOutput_0
{
    @location(0) output_0 : vec4<f32>,
};

@fragment
fn fs_main(@builtin(position) clip_position_1 : vec4<f32>) -> pixelOutput_0
{
    var _S1 : pixelOutput_0 = pixelOutput_0( vec4<f32>(0.30000001192092896f, 0.20000000298023224f, 0.10000000149011612f, 1.0f) );
    return _S1;
}

