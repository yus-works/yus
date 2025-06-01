struct Camera {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VSIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    // four columns of the model matrix
    @location(3) m0: vec4<f32>,
    @location(4) m1: vec4<f32>,
    @location(5) m2: vec4<f32>,
    @location(6) m3: vec4<f32>,
}

struct VSOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) frag_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vs_main(v: VSIn) -> VSOut {
    // Rebuild the model matrix per-instance
    let model = mat4x4<f32>(v.m0, v.m1, v.m2, v.m3);

    // Transform position and normal
    let world_pos = model * vec4<f32>(v.position, 1.0);
    var out: VSOut;
    out.pos = camera.view_proj * world_pos;
    out.frag_pos = world_pos.xyz;
    out.normal = normalize((model * vec4<f32>(v.normal, 0.0)).xyz);
    out.uv = v.uv;
    return out;
}
