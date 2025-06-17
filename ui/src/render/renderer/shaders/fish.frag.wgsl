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

fn palette(t: f32) -> vec3<f32> {
    let a = vec3<f32>(0.5, 0.5, 0.5);
    let b = vec3<f32>(0.5, 0.5, 0.5);
    let c = vec3<f32>(0.5, 0.5, 0.5);
    let d = vec3<f32>(0.263, 0.416, 0.557);

    return a + b * cos( 6.28318 * (c * t + d) );
}

@fragment
fn fs_main(in : FSIn) -> @location(0) vec4<f32> {
    let aspect = screen.resolution.x / screen.resolution.y;
    var uv = vec2<f32>(in.frag_pos.x * aspect, in.frag_pos.y);

    uv *= 2;
    uv = fract(uv);
    uv -= 0.5;

    var d = length(uv);
    let s = time_sec();

    var col = palette(d + s);

    d = sin(d*8 + s)/8;
    d = abs(d);

    d = 0.02 / d;

    col *= d;
    col = col;

    return vec4<f32>(col, 1.0);
}
