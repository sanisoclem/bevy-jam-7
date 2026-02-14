use bevy::{
  asset::{AssetLoader, LoadContext, io::Reader},
  prelude::*,
  time::Stopwatch,
};
use serde::{Deserialize, Serialize};
use sys_move::IsoMovementStage;
use sys_procgen::{NoiseGenSettings, ProceduralLevel};
use utils::assets::CustomRonAssetLoaderError;

use crate::level::render::ChunkMeshGenerator;

#[derive(Asset, TypePath, Debug, Deserialize, Serialize)]
pub struct LevelAsset {
  pub seed: u64,
  pub tiles_per_chunk: u32,
  pub noisegen_settings: [NoiseGenSettings; 8],
  pub tileset: TilesetDefinition,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TilesetDefinition {
  pub tile_height_screen: u32,
  pub tile_width_screen: u32,
  pub tile_height_world: u32,
  pub tile_width_world: u32,
  pub tile_height_sprite: u32,
  pub tile_width_sprite: u32,
}

#[derive(Default, TypePath)]
pub struct LevelAssetLoader;

impl AssetLoader for LevelAssetLoader {
  type Asset = LevelAsset;
  type Settings = ();
  type Error = CustomRonAssetLoaderError;
  async fn load(
    &self,
    reader: &mut dyn Reader,
    _settings: &(),
    _load_context: &mut LoadContext<'_>,
  ) -> Result<Self::Asset, Self::Error> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).await?;
    let custom_asset = ron::de::from_bytes::<LevelAsset>(&bytes)?;
    Ok(custom_asset)
  }

  fn extensions(&self) -> &[&str] {
    &["level.ron"]
  }
}

impl From<&LevelAsset> for IsoMovementStage {
  fn from(val: &LevelAsset) -> Self {
    let tile_size_screen = UVec2::new(
      val.tileset.tile_width_screen,
      val.tileset.tile_height_screen,
    );
    IsoMovementStage {
      aspect_ratio: tile_size_screen.y as f32 / tile_size_screen.x as f32,
      stopwatch: Stopwatch::new(),
    }
  }
}

impl From<&LevelAsset> for ProceduralLevel {
  fn from(value: &LevelAsset) -> Self {
    let tile_size_world = UVec2::new(
      value.tileset.tile_width_world,
      value.tileset.tile_height_world,
    );
    ProceduralLevel {
      seed: value.seed,
      tile_size: tile_size_world,
      noisegen: value
        .noisegen_settings
        .clone()
        .map(|s| s.create_generator(value.seed + s.seed_offset)),
    }
  }
}

impl From<&LevelAsset> for ChunkMeshGenerator {
  fn from(value: &LevelAsset) -> Self {
    let tile_size_screen = UVec2::new(
      value.tileset.tile_width_screen,
      value.tileset.tile_height_screen,
    );
    let tile_size_world = UVec2::new(
      value.tileset.tile_width_world,
      value.tileset.tile_height_world,
    );
    ChunkMeshGenerator {
      tile_size_screen: tile_size_screen.as_vec2(),
      tile_size_world: tile_size_world.as_vec2(),
      tiles_per_chunk: value.tiles_per_chunk,
    }
  }
}
