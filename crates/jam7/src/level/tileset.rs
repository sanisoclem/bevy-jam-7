use bevy::{
  asset::{AssetLoader, LoadContext, io::Reader},
  prelude::*,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Asset, TypePath, Debug, Deserialize, Serialize)]
pub struct TilesetDefinition {
  pub spritesheet: String,
  pub layout: UVec2,
  pub tiles: Vec<TileDefinition>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TileDefinition {
  pub index: u32,
  pub surface_height: u32,
}

#[derive(Default, TypePath)]
pub struct TilesetDefinitionLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum TilesetDefinitionLoaderError {
  #[error("Could not load asset: {0}")]
  Io(#[from] std::io::Error),
  #[error("Could not parse RON: {0}")]
  RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for TilesetDefinitionLoader {
  type Asset = TilesetDefinition;
  type Settings = ();
  type Error = TilesetDefinitionLoaderError;
  async fn load(
    &self,
    reader: &mut dyn Reader,
    _settings: &(),
    _load_context: &mut LoadContext<'_>,
  ) -> Result<Self::Asset, Self::Error> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).await?;
    let custom_asset = ron::de::from_bytes::<TilesetDefinition>(&bytes)?;
    Ok(custom_asset)
  }

  fn extensions(&self) -> &[&str] {
    &[".tileset"]
  }
}
