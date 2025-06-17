struct TimeUBO {
    millis     : u32,   // 0-999
    secs       : u32,   // whole seconds
    dt_millis  : u32,   // last-frame Δ in ms
    frame_id   : u32,   // ++ every render()
};

@group(0) @binding(6)
var<uniform> g_time : TimeUBO;

// helper if you want float seconds
fn time_sec() -> f32 {
    return f32(g_time.secs) + f32(g_time.millis) * 0.001;
}

struct FSIn {
    @location(0) frag_pos : vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@fragment
fn fs_main(in : FSIn) -> @location(0) vec4<f32> {
    return vec4<f32>(in.frag_pos.xy, 0.0, 1.0);
}
