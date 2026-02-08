use bevy::{
  asset::{AssetLoader, LoadContext, io::Reader},
  prelude::*,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::level::procgen::NoiseSettings;

#[derive(Asset, TypePath, Debug, Deserialize, Serialize)]
pub struct LevelAsset {
  pub seed: u64,
  pub tiles_per_chunk: u32,
  pub moisture_scale: f32,
  pub biopresence_scale: f32,
  pub bio_noise_settings: NoiseSettings,
  pub moisture_noise_settings: NoiseSettings,
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

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum LevelAssetLoaderError {
  #[error("Could not load asset: {0}")]
  Io(#[from] std::io::Error),
  #[error("Could not parse RON: {0}")]
  RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for LevelAssetLoader {
  type Asset = LevelAsset;
  type Settings = ();
  type Error = LevelAssetLoaderError;
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
    &[".level.ron"]
  }
}
