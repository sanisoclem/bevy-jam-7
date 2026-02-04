#import bevy_sprite::{
    mesh2d_functions as mesh_functions,
    mesh2d_view_bindings::{view,globals},
    mesh2d_vertex_output::VertexOutput,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var tileset: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var tileset_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var tile_data: texture_2d<u32>;

const DIAMOND_BASIS_X: vec2<f32> = vec2<f32>(0.5, -0.5);
const DIAMOND_BASIS_Y: vec2<f32> = vec2<f32>(0.5, 0.5);

struct TileData {
    tileset_index: u32,
    color: vec4<f32>,
    visible: bool,
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
fn get_tile_data(coord: vec2<u32>) -> TileData {
    let data = textureLoad(tile_data, coord, 0);

    let tileset_index = data.r;

    let color_r = f32(data.g & 0xFFu) / 255.0;
    let color_g = f32((data.g >> 8u) & 0xFFu) / 255.0;
    let color_b = f32(data.b & 0xFFu) / 255.0;
    let color_a = f32((data.b >> 8u) & 0xFFu) / 255.0;

    let color = vec4<f32>(color_r, color_g, color_b, color_a);

    let visible = data.a != 0u;

    return TileData(tileset_index, color, visible);
}

fn diamond_tile_pos_to_world_pos(pos: vec2<f32>, grid_width: f32, grid_height: f32) -> vec2<f32> {
    let unscaled_pos = pos.x * DIAMOND_BASIS_X + pos.y * DIAMOND_BASIS_Y;
    return vec2<f32>(grid_width * unscaled_pos.x, grid_height * unscaled_pos.y);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let t_1 = 1+.2 * sin(globals.time * 3.);
    let chunk_size = textureDimensions(tile_data, 0);
    let uv = in.uv; // + in.uv * t_1;// vec2<f32>(in.uv.x - in.uv.y * t_1, in.uv.y);
    let gg = uv  * 2.0 - 1. ;
    let tile_uv = gg * vec2<f32>(chunk_size) * 0.5;
    let f = fract(tile_uv);


    return vec4<f32>(f, 0., 1.0);
    var tile_coord = clamp(vec2<u32>(floor(tile_uv)), vec2<u32>(0), chunk_size - 1);
    tile_coord.y = chunk_size.y - 1 - tile_coord.y;

    let grid_size = 64.;
    let pos = vec2<f32>(in.world_position.x - in.world_position.y/0.3 , in.world_position.y);
    let tile = get_tile_data(tile_coord);

    if (tile.tileset_index == 0xffffu) {
        discard;
    }

    let local_uv = fract(tile_uv);// * vec2<f32>(1.0, 0.6) +vec2<f32>(0.0,0.2);
    let tex_color = textureSample(tileset, tileset_sampler, local_uv / vec2<f32>(11.0, 1.0), tile.tileset_index);

    var color = vec4<f32>(tex_color.xyz,  1.0);
//    color = color + (grid(pos / grid_size, 0.01, 10.1) * 1.2);
    return color;
}
