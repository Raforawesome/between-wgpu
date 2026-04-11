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
