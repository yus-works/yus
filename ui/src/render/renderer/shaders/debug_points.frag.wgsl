struct VSOut {
    @builtin(position) Position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@fragment
fn fs_main(i: VSOut) -> @location(0) vec4<f32> {
    let p = i.uv * 2.0 - 1.0;
    let d = length(p);

    let alpha = smoothstep(0, 0.01, 1 - d);

    let colour = vec3<f32>(1.0, 1.0, 1.0);
    return vec4(colour * alpha, alpha);
}
