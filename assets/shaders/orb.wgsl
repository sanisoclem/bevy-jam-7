#import bevy_sprite::{
    mesh2d_functions as mesh_functions,
    mesh2d_view_bindings::{view, globals},
}


@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> data: vec4<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    #ifdef VERTEX_TANGENTS
    @location(3) world_tangent: vec4<f32>,
    #endif
}

#ifdef TONEMAP_IN_SHADER
#import bevy_core_pipeline::tonemapping
#endif

struct Vertex {
    @builtin(instance_index) instance_index: u32,
#ifdef VERTEX_POSITIONS
    @location(0) position: vec3<f32>,
#endif
#ifdef VERTEX_NORMALS
    @location(1) normal: vec3<f32>,
#endif
#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(3) tangent: vec4<f32>,
#endif
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif

#ifdef VERTEX_POSITIONS
    var world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
    out.world_position = mesh_functions::mesh2d_position_local_to_world(
        world_from_local,
        vec4<f32>(vertex.position, 1.0)
    );
    out.position = mesh_functions::mesh2d_position_world_to_clip(out.world_position);
#endif

#ifdef VERTEX_NORMALS
    out.world_normal = mesh_functions::mesh2d_normal_local_to_world(vertex.normal, vertex.instance_index);
#endif

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_functions::mesh2d_tangent_local_to_world(
        world_from_local,
        vertex.tangent
    );
#endif

    return out;
}

fn hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3(p.xyx) * 0.13);
    p3 += dot(p3, p3.yzx + 3.333);
    return fract((p3.x + p3.y) * p3.z);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    
    let a = hash(i);
    let b = hash(i + vec2(1.0, 0.0));
    let c = hash(i + vec2(0.0, 1.0));
    let d = hash(i + vec2(1.0, 1.0));
    
    let u = f * f * (3.0 - 2.0 * f);
    
    return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var pos = p;
    
    for (var i = 0; i < 5; i++) {
        value += amplitude * noise(pos * frequency);
        frequency *= 2.0;
        amplitude *= 0.5;
    }
    
    return value;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let time_offset = data.x;
    let intensity = data.y;
    let time = globals.time + time_offset;
    
    let uv = (in.uv - 0.5) * 2.0;
    let dist = length(uv);
    let angle = atan2(uv.y, uv.x);
    
    let rotated_uv = vec2(
        cos(time * 0.2) * uv.x - sin(time * 0.2) * uv.y,
        sin(time * 0.2) * uv.x + cos(time * 0.2) * uv.y
    );
    
    let crystal_noise = fbm(rotated_uv * 4.0 + vec2(time * 0.1, 0.0));
    let crystals = step(0.6, crystal_noise);
    let frost = fbm(uv * 8.0 + vec2(time * 0.05, time * 0.03)) * 0.5 + 0.5;
    let core = smoothstep(0.4, 0.0, dist);
    let inner = smoothstep(0.7, 0.2, dist) * (1.0 - core);
    let outer = smoothstep(1.0, 0.5, dist);
    let shimmer = 0.9 + 0.1 * sin(time * 3.0 + dist * 10.0);
    let white = vec3(1.0, 1.0, 1.0);
    let bright_cyan = vec3(0.7, 1.0, 1.0);
    let cyan = vec3(0.3, 0.8, 1.0);
    let ice_blue = vec3(0.6, 0.85, 1.0);
    
    var color = vec3(0.0);
    color = mix(color, ice_blue, outer * frost);
    color = mix(color, cyan, inner);
    color = mix(color, bright_cyan, core);
    color += crystals * white * 0.3 * outer;
    
    color *= shimmer * intensity;
    
    let alpha = smoothstep(1.0, 0.6, dist) * outer;
    
    var final_color = vec4(color, alpha);

#ifdef TONEMAP_IN_SHADER
    final_color = tonemapping::tone_mapping(final_color, view.color_grading);
#endif

    return final_color;
}
