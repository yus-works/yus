struct VSOut {
    @builtin(position) Position: vec4<f32>,
    @location(0)      uv: vec2<f32>,
};

struct TimeUBO {
    millis: u32,   // 0-999
    secs: u32,   // whole seconds
    dt_millis: u32,   // last-frame Δ in ms
    frame_id: u32,   // ++ every render()
};

@group(0) @binding(0)
var<uniform> g_time : TimeUBO;

// helper if you want float seconds
fn time_sec() -> f32 {
    return f32(g_time.secs) + f32(g_time.millis) * 0.001;
}

fn triangle_wave(x: f32) -> f32 {
    return abs(x * 2 - 1);
}

@fragment
fn joints_fs(i: VSOut) -> @location(0) vec4<f32> {
    let p = i.uv * 2.0 - 1.0;
    let d = length(p);

    let alpha = smoothstep(0, 0.01, 1 - d) - smoothstep(0.15, 0.2, 1 - d);

    let colour = vec3<f32>(1.0, 1.0, 1.0);
    return vec4(colour * alpha, alpha);
}

@fragment
fn bones_fs(i: VSOut) -> @location(0) vec4<f32> {
    let t = time_sec();
    let speed = 0.2;
    let phase = triangle_wave(fract(i.uv.x - t * speed));

    let colorA = vec3<f32>(0.96, 0.30, 0.10);
    let colorB = vec3<f32>(0.10, 0.30, 0.96);

    let rgb = mix(colorA, colorB, phase);

    return vec4(rgb, 1.0);
}
