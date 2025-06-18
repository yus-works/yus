struct Light { dir : vec3<f32>, color : vec3<f32> };

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

@group(1) @binding(2) var<uniform> light  : Light;

@group(2) @binding(0) var texture_data    : texture_2d<f32>;
@group(2) @binding(1) var texture_sampler : sampler;

struct FSIn {
    @location(0) frag_pos : vec3<f32>,
    @location(1) normal   : vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@fragment
fn fs_main(in : FSIn) -> @location(0) vec4<f32> {
    // basic lambert lighting
    let N    = normalize(in.normal);
    let L    = normalize(-light.dir);
    let diff = max(dot(N, L), 0.0);

    let s = time_sec();

    let tex  = textureSample(
        texture_data, texture_sampler, in.uv * sin(s)
    );

    let ambient = 0.5;
    let lit     = tex.rgb * (ambient + diff * light.color);
    return vec4<f32>(lit, tex.a);
}
