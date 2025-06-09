struct Light { dir : vec3<f32>, color : vec3<f32> };

@group(0) @binding(2) var<uniform> light            : Light; // light data (direction and color)
@group(0) @binding(3) var texture_data    : texture_2d<f32>; // the actual 2d texture image
@group(0) @binding(4) var texture_sampler : sampler;         // the thing that tells the gpu how to read the texture

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

    let tex  = textureSample(texture_data, texture_sampler, in.uv);

    let ambient = 0.5;
    let lit     = tex.rgb * (ambient + diff * light.color);
    return vec4<f32>(lit, tex.a);
}
