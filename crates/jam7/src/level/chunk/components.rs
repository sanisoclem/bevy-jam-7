use bevy::prelude::*;

#[derive(Debug, Clone, Component)]
pub struct ChunkGenerator {
  pub owner_id: u32,
  pub chunk_size: u32,
  pub tile_size: UVec2,
  pub seed: i64,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Reflect)]
pub struct ChunkId(i32, i32);

impl ChunkId {
  pub fn x(&self) -> i32 {
    self.0
  }
  pub fn y(&self) -> i32 {
    self.1
  }
  pub const fn offset(&self, x: i32, y: i32) -> ChunkId {
    ChunkId(self.0 + x, self.1 + y)
  }
  pub fn center_world(&self, chunk_size: u32, tile_size: UVec2) -> Vec2 {
    let effective_chunk_size = chunk_size as f32 * tile_size.as_vec2();
    let center_x = (self.0 as f32) * effective_chunk_size.x;
    let center_y = (self.1 as f32) * effective_chunk_size.y;
    Vec2::new(center_x, center_y)
  }
  pub fn from_world_pos(world: Vec2, effective_chunk_size: Vec2) -> ChunkId {
    let x = (world.x / effective_chunk_size.x).floor() as i32;
    let y = (world.y / effective_chunk_size.y).floor() as i32;
    ChunkId(x, y)
  }
  pub fn get_chunks_to_be_loaded(
    origin: Vec2,
    chunk_size: u32,
    tile_size: UVec2,
    load_radius: u32,
  ) -> Vec<ChunkId> {
    let effective_chunk_size = chunk_size as f32 * tile_size.as_vec2();
    let origin_chunk = ChunkId::from_world_pos(origin, effective_chunk_size);
    let radius = load_radius as i32;
    let radius_squared =
      (load_radius as f32 * effective_chunk_size.x.max(effective_chunk_size.y)).powi(2);

    (-radius..=radius)
      .flat_map(|dx| {
        (-radius..=radius).filter_map(move |dy| {
          let new_chunk = origin_chunk.offset(dx, dy);
          let dist_squared = new_chunk
            .center_world(chunk_size, tile_size)
            .distance_squared(origin);
          (dist_squared <= radius_squared).then_some(new_chunk)
        })
      })
      .collect()
  }
}

#[derive(Clone, Component, Reflect)]
#[reflect(Component, Clone)]
pub struct LevelChunk {
  pub id: ChunkId,
  pub size: u32,
  pub tile_size: UVec2,
  pub center: Vec2,
}

#[derive(Component, Debug)]
pub struct ChunkSpawner {
  pub owner_id: u32,
  pub load_radius: u32,
  pub unload_radius: u32,
}
