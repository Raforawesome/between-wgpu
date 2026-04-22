import bind_groups;
import vertex;
import fragment;

[shader("vertex")]
public VertexOutput vs_main(VertexInput vert) {
    return vertex_shader(vert);
}

[shader("fragment")]
public float4 fs_main(VertexOutput in) : SV_Target {
    return fragment_shader(in);
}
