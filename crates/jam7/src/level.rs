pub(crate) mod asset;
pub(crate) mod chunk;
pub(crate) mod procgen;
pub(crate) mod render;

use asset::LevelAsset;
use bevy::prelude::*;
use chunk::{ChunkGenerator, despawn_chunks, spawn_chunks};
use procgen::{ProceduralLevel, generate_tile_data};
use render::render_tile_data;

use crate::level::render::TileSpriteLevel;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_message::<LevelCommand>()
      .init_asset::<asset::LevelAsset>()
      .init_asset_loader::<asset::LevelAssetLoader>()
      .add_systems(
        Update,
        (
          load_level,
          spawn_chunks,
          despawn_chunks,
          process_level_commands,
          generate_tile_data,
          render_tile_data,
        ),
      );
  }
}

#[derive(Debug, Message)]
pub enum LevelCommand {
  StartLevel(u32, String),
  UnloadLevel(u32),
}

#[derive(Component, Debug)]
pub struct Level {
  id: u32,
  descriptor: Handle<LevelAsset>,
}

pub fn load_level(
  mut cmd: Commands,
  mut ev_asset: MessageReader<AssetEvent<LevelAsset>>,
  asset_server: Res<AssetServer>,
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
  levels: Res<Assets<LevelAsset>>,
  qry: Query<(Entity, &Level)>,
) {
  for ev in ev_asset.read() {
    let AssetEvent::LoadedWithDependencies { id } = ev else {
      continue;
    };
    for (entity, level) in qry.iter() {
      if &level.descriptor.id() != id {
        continue;
      }
      let Some(level_descriptor) = levels.get(&level.descriptor) else {
        warn!("Unable to load level asset {:?}", id);
        continue;
      };

      info!("Loading level {:?}", level.id);

      let tile_size_sprite = UVec2::new(
        level_descriptor.tileset.tile_width_sprite,
        level_descriptor.tileset.tile_height_sprite,
      );
      let tile_size_screen = UVec2::new(
        level_descriptor.tileset.tile_width_screen,
        level_descriptor.tileset.tile_height_screen,
      );
      let tile_size_world = UVec2::new(
        level_descriptor.tileset.tile_width_world,
        level_descriptor.tileset.tile_height_world,
      );
      let chunk_size_world = (level_descriptor.tiles_per_chunk * tile_size_world).as_vec2();
      let chunk_size_screen = (level_descriptor.tiles_per_chunk * tile_size_screen).as_vec2();

      let tileset = asset_server.load(format!(
        "tilesets/{}.png",
        level_descriptor.tileset.spritesheet
      ));
      let layout = TextureAtlasLayout::from_grid(
        tile_size_sprite,
        level_descriptor.tileset.layout_x,
        level_descriptor.tileset.layout_y,
        None,
        None,
      );
      let texture_atlas_layout = texture_atlas_layouts.add(layout);
      let spawned_level = cmd
        .spawn((
          ProceduralLevel {
            level_id: level.id,
            seed: level_descriptor.seed,
            tiles_per_chunk: level_descriptor.tiles_per_chunk,
            moisture_scale: level_descriptor.moisture_scale,
            biopresence_scale: level_descriptor.biopresence_scale,
          },
          TileSpriteLevel {
            tile_size_screen: tile_size_screen.as_vec2(),
            tileset,
            tile_size_world: tile_size_world.as_vec2(),
            tiles: level_descriptor.tileset.tiles.clone(),
            layout: texture_atlas_layout.clone(),
          },
          Transform::default(),
          Visibility::default(),
          ChunkGenerator {
            level_id: level.id,
            chunk_size_world,
            chunk_size_screen,
          },
        ))
        .id();
      cmd
        .entity(entity)
        .despawn_children()
        .replace_children(&[spawned_level]);
    }
  }
}

pub fn process_level_commands(
  mut cmd: Commands,
  asset_server: Res<AssetServer>,
  mut reader: MessageReader<LevelCommand>,
  qry_levels: Query<(Entity, &ProceduralLevel)>,
) {
  for command in reader.read() {
    match command {
      LevelCommand::StartLevel(level_id, level_name) => {
        let handle: Handle<LevelAsset> =
          asset_server.load(format!("levels/{}.level.ron", level_name));
        cmd.spawn((
          Level {
            id: *level_id,
            descriptor: handle,
          },
          Transform::default(),
          Visibility::default(),
        ));
      }
      LevelCommand::UnloadLevel(level_id) => {
        for (entity, level) in qry_levels.iter() {
          if level.level_id != *level_id {
            continue;
          }

          cmd.entity(entity).despawn();
        }
      }
    }
  }
}
