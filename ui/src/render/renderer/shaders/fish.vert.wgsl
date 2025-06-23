struct VSOut {
    @builtin(position) Position : vec4<f32>,
    @location(0)      uv        : vec2<f32>,
};

struct JointIn {
    @location(0) pos   : vec3<f32>,
    @location(1) normal: vec3<f32>,      // unused for now
    @location(2) uv    : vec2<f32>,

    // the 4-column model matrix supplied by InstanceRaw
    @location(3) m0 : vec4<f32>,
    @location(4) m1 : vec4<f32>,
    @location(5) m2 : vec4<f32>,
    @location(6) m3 : vec4<f32>,
};

struct BoneIn {
    @location(0) pos   : vec3<f32>,
    @location(1) normal: vec3<f32>,      // unused for now
    @location(2) uv    : vec2<f32>,
};

@vertex
fn joints_vs(v: JointIn) -> VSOut {
    let model : mat4x4<f32> = mat4x4<f32>(v.m0, v.m1, v.m2, v.m3);

    var o : VSOut;
    o.Position = model * vec4<f32>(v.pos, 1.0);
    o.uv       = v.uv;
    return o;
}

@vertex
fn bones_vs(v: BoneIn) -> VSOut {
    var o : VSOut;
    o.Position = vec4f(v.pos, 1.0);      // clip-space already
    o.uv       = v.uv;
    return o;
}
