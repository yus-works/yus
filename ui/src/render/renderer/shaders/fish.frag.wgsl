struct TimeUBO {
    millis     : u32,   // 0-999
    secs       : u32,   // whole seconds
    dt_millis  : u32,   // last-frame Δ in ms
    frame_id   : u32,   // ++ every render()
};

@group(0) @binding(0)
var<uniform> g_time : TimeUBO;

struct Screen {
    resolution : vec2<f32>, // (width, height)
    _pad       : vec2<f32>, // alignment padding
};
@group(0) @binding(1)
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

    let uv0 = uv;
    var finCol = vec3<f32>(0.0);

    for (var i = 0.0; i < 4.0; i = i + 1.0) {
        uv = fract(uv * 1.5) - 0.5;

        var d = length(uv) * exp(-length(uv0));
        let s = time_sec();

        var col = palette(length(uv0) + s + i*.4);

        d = sin(d*8 + s)/8;
        d = abs(d);

        d = pow(0.01 / d, 2);

        finCol = finCol + col * d;
    }

    return vec4<f32>(finCol, 1.0);
}
