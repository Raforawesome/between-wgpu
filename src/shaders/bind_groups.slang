module bind_groups;

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

// declare bind groups in incrementing order
public ParameterBlock<TexBindGroup> bg; // bind group 0
public ParameterBlock<Camera> camera;   // bind group 1
