struct VSOut {
    @builtin(position) Position : vec4<f32>,
    @location(0)      uv        : vec2<f32>,
};

struct TimeUBO {
    millis     : u32,   // 0-999
    secs       : u32,   // whole seconds
    dt_millis  : u32,   // last-frame Δ in ms
    frame_id   : u32,   // ++ every render()
};

@group(0) @binding(0)
var<uniform> g_time : TimeUBO;

// helper if you want float seconds
fn time_sec() -> f32 {
    return f32(g_time.secs) + f32(g_time.millis) * 0.001;
}

@fragment
fn fs_main(i: VSOut) -> @location(0) vec4<f32> {
    // warm gradient just so we see something
    var x = sin(i.uv.x + time_sec() * 5);
    var y = sin(i.uv.y + time_sec() * 5);
    var z = sin(i.uv.x + time_sec() * 5);

    return vec4f(x * 0.8, y * 0.8, 0.6, 1.0);
}
