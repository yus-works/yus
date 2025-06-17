struct TimeUBO {
    millis     : u32,   // 0-999
    secs       : u32,   // whole seconds
    dt_millis  : u32,   // last-frame Δ in ms
    frame_id   : u32,   // ++ every render()
};

@group(0) @binding(6)
var<uniform> g_time : TimeUBO;

struct Screen {
    resolution : vec2<f32>, // (width, height)
    _pad       : vec2<f32>, // alignment padding
};
@group(0) @binding(7)
var<uniform> screen : Screen;

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
    let aspect = screen.resolution.x / screen.resolution.y;
    let uv = vec2<f32>(in.frag_pos.x * aspect, in.frag_pos.y);

    let d = length(uv);
    return vec4<f32>(d, d, 0.0, 1.0);
}
