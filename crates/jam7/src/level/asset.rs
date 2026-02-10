use bevy::{
  asset::{AssetLoader, LoadContext, io::Reader},
  prelude::*,
};
use serde::{Deserialize, Serialize};
use sys_procgen::NoiseGenSettings;
use utils::assets::CustomRonAssetLoaderError;

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
