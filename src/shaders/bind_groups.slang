/*
 * Bind Groups
 */

// Bind group 0: texture atlas and sampler.
public struct TexBindGroup {
    [format("rgba8")] // matches wgpu's Rgba8UnormSrgb texture format
    public Texture2DArray textures;
    public SamplerState sampler;
}

// Bind group 1: camera uniforms.
public struct Camera {
    public float4x4 view_proj;
}

// Bind group 2: per-chunk uniforms and block-id texture.
// Slang emits the non-resource field as binding 0 and the texture as binding 1,
// matching Texture::blocks_layout_desc in Rust.
public struct ChunkBlocks {
    public float3 world_pos;
    public Texture3D<uint> blocks;
}

// Declare bind groups in pipeline-layout order.
public ParameterBlock<TexBindGroup> bg;
public ParameterBlock<Camera> camera;
public ParameterBlock<ChunkBlocks> chunk;
