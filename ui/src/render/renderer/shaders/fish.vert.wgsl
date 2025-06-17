struct VSIn {
    @location(0) position : vec3<f32>,
    @location(1) uv       : vec2<f32>,
};

struct VSOut {
    @builtin(position)  pos      : vec4<f32>,
    @location(0)        frag_pos : vec3<f32>,
    @location(2)        uv       : vec2<f32>,
};

@vertex
fn vs_main(v: VSIn) -> VSOut {
    var out : VSOut;
    out.pos       = vec4<f32>(v.position, 1.0);
    out.frag_pos  = v.position;
    out.uv        = v.uv;
    return out;
}
