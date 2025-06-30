struct VSOut {
    @builtin(position) Position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(2) @binding(0)
var<uniform> u_colour : vec4<f32>;

@fragment
fn fs_main(i: VSOut) -> @location(0) vec4<f32> {
    let p = i.uv * 2.0 - 1.0;
    let d = length(p);

    let alpha = smoothstep(0, 0.01, 1 - d);

    return vec4(u_colour.rgb * alpha, alpha);
}
