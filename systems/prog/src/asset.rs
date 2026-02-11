use bevy::{
  asset::{AssetLoader, LoadContext, io::Reader},
  prelude::*,
};
use serde::Deserialize;

use crate::SpellBuilder;

#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct SpellBuilderConfig {
  pub spellbuilder: SpellBuilder,
}

#[derive(Default, TypePath)]
pub struct SpellBuilderConfigLoader;

impl AssetLoader for SpellBuilderConfigLoader {
  type Asset = SpellBuilderConfig;
  type Settings = ();
  type Error = Box<dyn std::error::Error + Send + Sync>;

  async fn load(
    &self,
    reader: &mut dyn Reader,
    _settings: &Self::Settings,
    _load_context: &mut LoadContext<'_>,
  ) -> Result<Self::Asset, Self::Error> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).await?;
    let config = ron::de::from_bytes(&bytes)?;
    Ok(config)
  }
}
