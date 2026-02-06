use bevy::prelude::*;
use libnoise::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{level::chunk::ChunkId, prelude::LevelChunk};

#[derive(Component, Debug)]
pub struct ProceduralLevel {
  pub level_id: u32,
  pub seed: u64,
  pub tiles_per_chunk: u32,
  pub moisture_scale: f32,
  pub biopresence_scale: f32,
  pub moisture_noise_settings: NoiseSettings,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NoiseSettings {
  pub worley_seed_offset1: u64,
  pub worley_seed_offset2: u64,
  pub worley_scale1: [f64; 2],
  pub worley_scale2: [f64; 2],
  pub fbm_octaves: u32,
  pub fbm_freq: f64,
  pub fbm_lacunarity: f64,
  pub fbm_persistence: f64,
}

impl ProceduralLevel {
  pub fn get_moisture(&self, tile_coords: IVec2) -> f32 {
    let s = &self.moisture_noise_settings;
    let generator = Source::simplex(self.seed)
      .fbm(
        s.fbm_octaves,
        s.fbm_freq,
        s.fbm_lacunarity,
        s.fbm_persistence,
      )
      .blend(
        Source::worley(self.seed + s.worley_seed_offset1).scale(s.worley_scale1),
        Source::worley(self.seed + s.worley_seed_offset2).scale(s.worley_scale2),
      );

    let coords = tile_coords.as_vec2() * self.moisture_scale;
    generator.sample([coords.x as f64, coords.y as f64]) as f32
  }
  pub fn get_biopresence(&self, tile_coords: IVec2) -> f32 {
    let generator = Source::simplex(self.seed).fbm(7, 0.023, 2.0, 0.5).blend(
      Source::worley(self.seed + 1).scale([0.05, 0.05]), // ...with scaled worley noise
      Source::worley(self.seed + 2).scale([0.02, 0.02]),
    );

    let coords = tile_coords.as_vec2() * self.biopresence_scale;
    generator.sample([coords.x as f64, coords.y as f64]) as f32
  }

  pub fn generate_chunk_tile_data(&self, chunk: ChunkId) -> Vec<TileData> {
    let range = 0..self.tiles_per_chunk;
    range
      .clone()
      .flat_map(|x| range.clone().map(move |y| (x, y)))
      .map(|(x, y)| {
        let coords =
          chunk.as_ivec2() * self.tiles_per_chunk as i32 + IVec2::new(x as i32, y as i32);
        TileData {
          coords,
          moisture: self.get_moisture(coords),
          biopresence: self.get_biopresence(coords),
        }
      })
      .collect()
  }
}

#[derive(Component, Debug)]
pub struct ChunkTileData {
  pub data: Vec<TileData>,
  pub loaded: bool,
}

#[derive(Component, Debug)]
pub struct TileData {
  pub coords: IVec2,
  pub moisture: f32,
  pub biopresence: f32,
}

pub fn generate_tile_data(
  mut cmd: Commands,
  qry_level: Query<(Entity, &ProceduralLevel)>,
  qry_chunk: Query<&LevelChunk, Without<ChunkTileData>>,
  qry_children: Query<&Children>,
) {
  for (level_entity, level) in qry_level {
    let Some(children) = qry_children.get(level_entity).ok() else {
      continue;
    };

    let chunks: Vec<_> = children
      .into_iter()
      .filter_map(|child| qry_chunk.get(*child).ok().map(|c| (child, c)))
      .collect();

    for (entity, chunk) in chunks {
      cmd.entity(*entity).insert(ChunkTileData {
        data: level.generate_chunk_tile_data(chunk.id),
        loaded: false,
      });
    }
  }
}
