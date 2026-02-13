#import bevy_sprite::{
    mesh2d_functions as mesh_functions,
    mesh2d_view_bindings::{view, globals},
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> data: vec4<f32>; // (time_offset, intensity, inner_radius, outer_radius)

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

// Simple noise function
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
    let inner_radius = 50.;
    let outer_radius = 100.;
    
    let time = globals.time + time_offset;
    
    let uv = (in.uv - 0.5) * 2.0;
    let dist = length(uv);
    
    let noise_val = fbm(uv * 3.0 + vec2(time * 0.5, time * 0.3));
    let distortion = fbm(uv * 5.0 - vec2(time * 0.8, time * 0.6)) * 0.1;
    let distorted_dist = dist + distortion;
    let core = smoothstep(0.3, 0.0, distorted_dist);
    let inner = smoothstep(0.6, 0.2, distorted_dist) * (1.0 - core);
    let outer = smoothstep(1.0, 0.4, distorted_dist + noise_val * 0.2);
    let flicker = 0.9 + 0.1 * sin(time * 10.0 + noise_val * 6.28);
    
    let white = vec3(1.0, 1.0, 1.0);
    let yellow = vec3(1.0, 0.9, 0.3);
    let orange = vec3(1.0, 0.5, 0.1);
    let red = vec3(0.8, 0.1, 0.0);
    
    var color = vec3(0.0);
    color = mix(color, red, outer);
    color = mix(color, orange, outer * 0.5);
    color = mix(color, yellow, inner);
    color = mix(color, white, core);
    
    color *= flicker * intensity;
    
    let alpha = smoothstep(1.0, 0.5, dist) * outer;
    
    var final_color = vec4(color, alpha);

#ifdef TONEMAP_IN_SHADER
    final_color = tonemapping::tone_mapping(final_color, view.color_grading);
#endif

    return final_color;
}
