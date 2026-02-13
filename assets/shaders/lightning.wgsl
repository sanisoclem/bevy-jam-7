#import bevy_sprite::{
    mesh2d_functions as mesh_functions,
    mesh2d_view_bindings::{view, globals},
}

// (time_offset, intensity, length, width)
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

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let time_offset = data.x;
    let intensity = data.y;
    
    let time = globals.time + time_offset;
    
    // center UV to -1 to 1
    let uv = (in.uv - 0.5) * 2.0;
    
    // jagged lightning bolt along x-axis
    let bolt_noise = noise(vec2(uv.x * 8.0, time * 20.0)) * 0.3;
    let bolt_dist = abs(uv.y - bolt_noise);
    
    // core bolt (bright white)
    let core = smoothstep(0.05, 0.0, bolt_dist);
    
    // inner glow (cyan-white)
    let inner = smoothstep(0.15, 0.02, bolt_dist);
    
    // outer glow (blue)
    let outer = smoothstep(0.35, 0.1, bolt_dist);
    
    // arcs/branches
    let arc_noise = noise(vec2(uv.x * 12.0 + time * 15.0, uv.y * 8.0));
    let arc = step(0.85, arc_noise) * smoothstep(0.5, 0.0, abs(uv.y));
    
    // rapid flickering
    let flicker = 0.8 + 0.2 * sin(time * 50.0 + noise(uv * 10.0) * 6.28);
    
    // color composition
    let white = vec3(1.0, 1.0, 1.0);
    let cyan = vec3(0.5, 0.9, 1.0);
    let blue = vec3(0.2, 0.5, 1.0);
    
    var color = vec3(0.0);
    color = mix(color, blue, outer);
    color = mix(color, cyan, inner);
    color = mix(color, white, core);
    color += arc * cyan * 0.5;
    
    color *= flicker * intensity;
    
    // alpha
    let alpha = max(outer, arc * 0.8);
    
    var final_color = vec4(color, alpha);

#ifdef TONEMAP_IN_SHADER
    final_color = tonemapping::tone_mapping(final_color, view.color_grading);
#endif

    return final_color;
}
