#import bevy_sprite::{
    mesh2d_functions as mesh_functions,
    mesh2d_view_bindings::{view, globals},
}

// (time_offset, intensity, radius, unused)
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

fn hash_color(v:u32) -> vec3<f32> {
    let hash_x = u32(max(v,1) * 374761393 + max(v,1) * 668265263);
    let hash_y = u32(v * 1274126177 + v * 1664525);
    let hash_z = u32((v + v) * 2147483647);
    return vec3<f32>(
        f32(hash_x % 1000u) / 1000.0,
        f32(hash_y % 1000u) / 1000.0,
        f32(hash_z % 1000u) / 1000.0
    );
}
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let time_offset = data.x;
    let time = globals.time + time_offset;
    let intensity = data.y;
    let radius = 0.8; // use uvs, ignore radius uniform

    let num_slices = 20.;
    let uv = in.uv;

    // normalize uvs 
    let cuv = (in.uv * 2.0 - 1.0) * radius;

    // accumulate stuff
    var brightness = 0.0;
    var color = vec3(0.0);
    var debug = 0.0;
    var debug2 = vec3(0.0);

    // take vertical slices of the z axis so 
    // we can determine the brightness contribution of each slice
    for (var i = 0.0; i < num_slices; i += 1.0) {
      let z = i/num_slices;

      // screen to world coord transformation
      // world coords are uv.x, uv.y, and z 
      let cy = cuv.y + (z * 0.4);
      let frag_world = vec2<f32>(
          cuv.x + cy / 0.5,
          cy / 0.5 - cuv.x
      );

      // find dist to center when z is included
      // some distances will be greater than 1.0
      // and those would be intersecting the ball surface at 1.0
      var dist_2d = length(vec3(frag_world, z));

      // calculate how much more vertical dome is left in this silce and x,y coord
      // we can tweak the dome height by scaling this 
      let dome_height = sqrt(max(0.0, 1.0 - dist_2d * dist_2d)) * 0.5;

      // tracks how much dome we have in this fragment
      brightness = brightness + dome_height; 

      let xy_length_sq = dot(cuv, cuv);
      let normal = vec3(cuv, dome_height);
      let light_dir = normalize(vec3(0.3, 0.3, 1.0));
      let diffuse = max(0.0, dot(normal, light_dir));
      let rim = pow(1.0 - dome_height, 2.0);

    // no idea how this 
      let noise_val = fbm(uv * 3.0 + vec2(time * 0.8, time * 0.5));
      let distortion = fbm(uv * 5.0 - vec2(time * 1.2, time * 0.8)) * 0.1;
      let distorted_dist = dist_2d + distortion * (1.0 - dist_2d);
      let core = smoothstep(0.3, 0.0, distorted_dist) * pow(dome_height, 3.0);
      let inner = smoothstep(0.7, 0.2, distorted_dist) * diffuse;
      let outer = smoothstep(1.0, 0.3, distorted_dist + noise_val * 0.2) * (0.5 + rim * 0.5);
      let flicker = 0.85 + 0.15 * sin(time * 12.0 + noise_val * 6.28);
      
      let white = vec3(1.0, 1.0, 1.0);
      let yellow = vec3(1.0, 0.9, 0.3);
      let orange = vec3(1.0, 0.5, 0.1);
      var halo = vec3(0.9, 0.1, 0.2);
      if data.w > 0.0 {
          halo = vec3(0.0, 0.4, 0.7);
      } else if data.w > 5.0 {
          halo = vec3(0.9, 0.8, 0.9);
      }
      
      color = mix(color, halo, outer);
      color = mix(color, orange, inner);
      color = mix(color, yellow, inner * diffuse);
      color = mix(color, white, core);
    }

    var final_color = vec4(clamp(color * intensity, vec3(0.), vec3(1.0)), brightness);

#ifdef TONEMAP_IN_SHADER
    final_color = tonemapping::tone_mapping(final_color, view.color_grading);
#endif

    return final_color;
}

