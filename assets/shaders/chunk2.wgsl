#import bevy_sprite::{
    mesh2d_functions as mesh_functions,
    mesh2d_view_bindings::{view, globals},
    mesh2d_vertex_output::VertexOutput,
}

#ifdef TONEMAP_IN_SHADER
#import bevy_core_pipeline::tonemapping
#endif

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var tile_data: texture_2d<u32>;

struct TileData {
    data1: vec4<f32>,
    data2: vec4<f32>,
}

fn get_tile_data(coord: vec2<u32>) -> TileData {
    let data = textureLoad(tile_data, coord, 0);

    let d1 = f32(data.r & 0xFFu) / 255.0;
    let d2  = f32((data.r >> 8u) & 0xFFu) / 255.0;
    let d3 = f32(data.g & 0xFFu) / 255.0;
    let d4 = f32((data.g >> 8u) & 0xFFu) / 255.0;
    let d5 = f32(data.b & 0xFFu) / 255.0;
    let d6 = f32((data.b >> 8u) & 0xFFu) / 255.0;
    let d7 = f32(data.a & 0xFFu) / 255.0;
    let d8 = f32((data.a >> 8u) & 0xFFu) / 255.0;


    return TileData(vec4<f32>(d1, d2, d3,d4), vec4<f32>(d5,d6,d7,d8));
}

fn grid(uv: vec2f, w: f32, vs: f32) -> f32 {
    var lineWidth: vec2f = vec2<f32>(w, w);
    var ddx: vec2f = dpdx(uv);
    var ddy: vec2f = dpdy(uv);
    var uvDeriv: vec2f = vec2(length(vec2(ddx.x, ddy.x)), length(vec2(ddx.y, ddy.y)));
    let invertLine: vec2<bool> = vec2<bool>(lineWidth.x > 0.5, lineWidth.y > 0.5);
    var targetWidth: vec2<f32>;
    if invertLine.x {
        targetWidth.x = 1.0 - lineWidth.x;
    } else {
        targetWidth.x = lineWidth.x;
    };
    if invertLine.y {
        targetWidth.y = 1.0 - lineWidth.y;
    } else {
        targetWidth.y = lineWidth.y;
    };
    let drawWidth: vec2f = clamp(targetWidth, uvDeriv, vec2(0.5));
    let lineAA: vec2f = uvDeriv * 1.5;
    var gridUV: vec2f = abs(fract(uv) * 2.0 - 1.0);
    if invertLine.x { gridUV.x = gridUV.x; } else { gridUV.x = 1.0 - gridUV.x; };
    if invertLine.y { gridUV.y = gridUV.y; } else { gridUV.y = 1.0 - gridUV.y; };
    var grid2: vec2f = smoothstep(drawWidth + lineAA, drawWidth - lineAA, gridUV);

    grid2 *= clamp(targetWidth / drawWidth, vec2(0.0), vec2(1.0));
    grid2 = mix(grid2, targetWidth, clamp(uvDeriv * 2.0 - 1.0, vec2(0.0), vec2(1.0)));
    if invertLine.x {
        grid2.x = 1.0 - grid2.x;
    };// else { grid2.x = grid2.x };
    if invertLine.y {
        grid2.y = 1.0 - grid2.y;
    }; // else { grid2.y = grid2.y };
    return mix(grid2.x, 1.0, grid2.y);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let chunk_size = textureDimensions(tile_data, 0);
    let tile_uv = in.uv * vec2<f32>(chunk_size);
    var tile_coord = clamp(vec2<u32>(floor(tile_uv)), vec2<u32>(0), chunk_size - 1);
    let tile = get_tile_data(tile_coord);
    let g = (grid(tile_uv, 0.01, 10.1) * 1.);

    let t_1 = 1+.2 * sin(globals.time * 3.);
    let t_2 = 0.5 + sin( (globals.time * 1.1) + (tile.data1.y + tile.data1.x) ) * 0.5;
    let t_3 = 0.5 + cos( (globals.time * 5.1) + f32(tile_coord.x) ) * 0.1;
    let t_4 = 0.5 + sin( (globals.time * 5.1) + f32(tile_coord.y) ) * 0.1;

    var color = vec4<f32>(tile.data1.y * t_3, tile.data1.y * t_2,tile.data1.y * t_4, 1.0) * (1. - g);

    if (color.a < 0.001) {
        discard;
    }


    let pos = vec2<f32>(in.world_position.x + in.world_position.y / 0.5 , in.world_position.y / 0.5 - in.world_position.x);
    if (distance(pos, vec2<f32>(0.0)) <= 50.) {
//        return vec4<f32>(1.0);
    } else {
 //       return vec4<f32>(0.0);
    }

#ifdef TONEMAP_IN_SHADER
    color = tonemapping::tone_mapping(color, view.color_grading);
#endif

    return color;
}
