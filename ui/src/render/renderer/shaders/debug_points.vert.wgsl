struct VSOut {
    @builtin(position) Position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct VSIn {
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,

    // the 4-column model matrix supplied by InstanceRaw
    @location(3) m0: vec4<f32>,
    @location(4) m1: vec4<f32>,
    @location(5) m2: vec4<f32>,
    @location(6) m3: vec4<f32>,
};

struct Camera { view_proj : mat4x4<f32>, };
@group(1) @binding(0)
var<uniform> camera : Camera;

@vertex
fn vs_main(v: VSIn) -> VSOut {
    let model: mat4x4<f32> = mat4x4<f32>(v.m0, v.m1, v.m2, v.m3);

    var o: VSOut;
    o.Position = camera.view_proj * model * vec4<f32>(v.pos, 1.0);
    o.uv = v.uv;
    return o;
}
