struct VSIn {
    @location(0) pos   : vec3<f32>,
    @location(1) normal: vec3<f32>,      // ignored
    @location(2) uv    : vec2<f32>,
};

struct VSOut {
    @builtin(position) Position : vec4<f32>,
    @location(0)      uv        : vec2<f32>,
};

@vertex
fn vs_main(v: VSIn) -> VSOut {
    var o : VSOut;
    o.Position = vec4f(v.pos, 1.0);      // clip-space already
    o.uv       = v.uv;
    return o;
}
