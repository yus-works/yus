struct VSOut {
    @builtin(position) Position : vec4<f32>,
    @location(0)      uv        : vec2<f32>,
};

@fragment
fn fs_main(i: VSOut) -> @location(0) vec4<f32> {
    // warm gradient just so we see something
    return vec4f(i.uv.x, i.uv.y, 1.0 - i.uv.x, 1.0);
}
