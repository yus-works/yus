struct Camera {
    view_proj: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> camera: Camera;

struct VSIn {
    @location(0) position: vec3<f32>,  // <- from Vertex::ATTRIBS[0]
    @location(1) normal: vec3<f32>,    // <- from Vertex::ATTRIBS[1]
    @location(2) uv: vec2<f32>,        // <- from Vertex::ATTRIBS[2]

    @location(3) m0: vec4<f32>,        // <- from InstanceRaw (4 columns of model matrix)
    @location(4) m1: vec4<f32>,
    @location(5) m2: vec4<f32>,
    @location(6) m3: vec4<f32>,
}

struct VSOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) frag_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vs_main(v: VSIn) -> VSOut {
    // Rebuild the model matrix per-instance
    let model = mat4x4<f32>(v.m0, v.m1, v.m2, v.m3);

    // object space -> world space
    let world_pos = model * vec4<f32>(v.position, 1.0);

    var out: VSOut;

    // world space -> camera space -> screen space
    out.pos = camera.view_proj * world_pos;

    // pass world_pos of each vertex to frag shader
    out.frag_pos = world_pos.xyz;

    // transform vertex normal vector to world space then normalize
    out.normal = normalize((model * vec4<f32>(v.normal, 0.0)).xyz);

    // pass vertex uv to frag shader
    out.uv = v.uv;

    return out;
}
