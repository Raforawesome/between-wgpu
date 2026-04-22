module fragment;

import bind_groups;
import vertex;

/*
 * Fragment Shader
*/
[shader("fragment")]
public float4 fragment_shader(VertexOutput in) : SV_Target {
    let tex_array = bg.textures;
    let sampler = bg.sampler;

    return tex_array.Sample(sampler, in.tex_coords);
}
