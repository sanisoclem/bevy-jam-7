use bevy::{prelude::*, sprite_render::TilemapChunk};

use super::chunk::LevelChunk;

pub struct Tile {
  moisture: f32,
  macro_elevation: f32,
  elevation: f32,
  destruction: f32,
}

pub struct Tileset {}

pub fn generate_chunk_tiles(
  mut commands: Commands,
  assets: Res<AssetServer>,
  qry_chunk: Query<(Entity, &LevelChunk), Without<TilemapChunk>>,
) {

  // procedurally generate tile data and spawn tile entities
}

pub fn on_insert_tilemap_chunk() {
  // generate the mesh or get from mesh cache
  // set material/shader and set initial value for tile data
}

pub fn update_tilemap_chunk_indices() {
  // compute and update tile data based on Tile component changes
  // build and update texture
}
