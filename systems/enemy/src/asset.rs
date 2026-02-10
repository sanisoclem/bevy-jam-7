use bevy::{
  asset::{AssetLoader, LoadContext, io::Reader},
  platform::collections::HashMap,
  prelude::*,
  sprite::Anchor,
};
use serde::Deserialize;
use sys_animation::{AnimationDefinition, AnimationPlaybackSpeed};
use sys_combat::HitTestableShape;
use sys_move::MoveDirection;
use utils::assets::CustomRonAssetLoaderError;

use crate::EnemyAnimationState;

#[derive(Asset, TypePath, Clone, Debug)]
pub struct EnemyDescriptor {
  pub prevalence: f32,
  pub rangeness: f32,
  pub anchor: Anchor,
  pub scale: Vec2,
  pub hitbox: HitTestableShape,
  pub spritesheets: Vec<Handle<Image>>,
  pub animations: HashMap<EnemyAnimationState, AnimationDefinition>,
}

#[derive(Deserialize)]
pub struct EnemyDescriptorRon {
  pub prevalence: f32,
  pub hitbox: HitTestableShape,
  pub rangeness: f32,
  pub anchor: Vec2,
  pub scale: Vec2,
  pub spritesheets: Vec<String>,
  pub animations: Vec<EnemyAnimationRon>,
}

#[derive(Deserialize)]
pub struct EnemyAnimationRon {
  pub facing: MoveDirection,
  pub is_moving: bool,
  pub spritesheet_index: usize,
  pub frames: Vec<usize>,
  pub playback_speed: AnimationPlaybackSpeed,
  pub should_loop: bool,
  pub should_flip: bool,
}

#[derive(Default, TypePath)]
pub struct EnemyDescriptorAssetLoader;

impl AssetLoader for EnemyDescriptorAssetLoader {
  type Asset = EnemyDescriptor;
  type Settings = ();
  type Error = CustomRonAssetLoaderError;

  async fn load(
    &self,
    reader: &mut dyn Reader,
    _settings: &Self::Settings,
    load_context: &mut LoadContext<'_>,
  ) -> Result<Self::Asset, Self::Error> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).await?;
    let ebr = ron::de::from_bytes::<EnemyDescriptorRon>(&bytes)?;

    let spritesheets: Vec<Handle<Image>> = ebr
      .spritesheets
      .iter()
      .map(|s| load_context.load(format!("char/{}.png", s)))
      .collect();
    let layouts: Vec<Handle<TextureAtlasLayout>> = ebr
      .spritesheets
      .iter()
      .map(|s| load_context.load(format!("char/{}.layout.ron", s)))
      .collect();
    let animations = ebr
      .animations
      .into_iter()
      .filter_map(|x| {
        let s = spritesheets.get(x.spritesheet_index)?;
        let l = layouts.get(x.spritesheet_index)?;

        Some((
          EnemyAnimationState {
            facing: x.facing,
            moving: x.is_moving,
          },
          AnimationDefinition {
            spritesheet: s.clone(),
            layout: l.clone(),
            frames: x.frames,
            playback_speed: x.playback_speed,
            playback_loop: x.should_loop,
            flip_vertical: x.should_flip,
          },
        ))
      })
      .collect();

    Ok(EnemyDescriptor {
      scale: ebr.scale,
      prevalence: ebr.prevalence,
      hitbox: ebr.hitbox,
      rangeness: ebr.rangeness,
      anchor: Anchor(ebr.anchor),
      spritesheets,
      animations,
    })
  }

  fn extensions(&self) -> &[&str] {
    &["enemy.ron"]
  }
}

#[derive(Deserialize)]
pub struct TextureAtlasLayoutGridRon {
  pub cols: u32,
  pub rows: u32,
  pub tile_width: u32,
  pub tile_height: u32,
}

#[derive(Default, TypePath)]
pub struct TextureAtlasLayoutAssetLoader;

impl AssetLoader for TextureAtlasLayoutAssetLoader {
  type Asset = TextureAtlasLayout;
  type Settings = ();
  type Error = CustomRonAssetLoaderError;

  async fn load(
    &self,
    reader: &mut dyn Reader,
    _settings: &Self::Settings,
    _load_context: &mut LoadContext<'_>,
  ) -> Result<Self::Asset, Self::Error> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).await?;
    let talgr = ron::de::from_bytes::<TextureAtlasLayoutGridRon>(&bytes)?;
    Ok(TextureAtlasLayout::from_grid(
      UVec2::new(talgr.tile_width, talgr.tile_height),
      talgr.cols,
      talgr.rows,
      None,
      None,
    ))
  }

  fn extensions(&self) -> &[&str] {
    &["layout.ron"]
  }
}
