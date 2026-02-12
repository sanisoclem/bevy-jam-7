pub(crate) mod asset;
pub(crate) mod render;

use crate::player::create_player;
use asset::LevelAsset;
use bevy::{prelude::*, sprite_render::Material2dPlugin, time::Stopwatch};
use render::{ChunkMaterial, ChunkMeshGenerator, IsoTilemapChunkMeshCache, render_tile_data};
use sys_chonker::{ChunkGenerator, SysChonkerPlugin};
use sys_move::IsoMovementStage;
use sys_procgen::ProceduralLevel;
use sys_prog::levelup::LevelUp;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins((
        Material2dPlugin::<ChunkMaterial>::default(),
        SysChonkerPlugin,
      ))
      .add_message::<LevelCommand>()
      .init_asset::<asset::LevelAsset>()
      .init_resource::<IsoTilemapChunkMeshCache>()
      .init_asset_loader::<asset::LevelAssetLoader>()
      .add_systems(
        Update,
        (load_level, process_level_commands, render_tile_data),
      );
  }
}

#[derive(Debug, Message)]
pub enum LevelCommand {
  StartLevel(String),
  UnloadLevel(String),
}

#[derive(Component, Debug)]
pub struct Level {
  name: String,
  descriptor: Handle<LevelAsset>,
}

pub fn load_level(
  mut cmd: Commands,
  mut ev_asset: MessageReader<AssetEvent<LevelAsset>>,
  mut layouts: ResMut<Assets<TextureAtlasLayout>>,
  asset_server: Res<AssetServer>,
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
      let level_descriptor = levels
        .get(&level.descriptor)
        .expect("level asset should be loaded");

      let tile_size_screen = UVec2::new(
        level_descriptor.tileset.tile_width_screen,
        level_descriptor.tileset.tile_height_screen,
      );
      let tile_size_world = UVec2::new(
        level_descriptor.tileset.tile_width_world,
        level_descriptor.tileset.tile_height_world,
      );
      let chunk_size_world = (level_descriptor.tiles_per_chunk * tile_size_world).as_vec2();

      let spawned_level = cmd
        .spawn((
          IsoMovementStage {
            aspect_ratio: tile_size_screen.y as f32 / tile_size_screen.x as f32,
            stopwatch: Stopwatch::new(),
          },
          ProceduralLevel {
            seed: level_descriptor.seed,
            tile_size: tile_size_world,
            noisegen: level_descriptor
              .noisegen_settings
              .clone()
              .map(|s| s.create_generator(level_descriptor.seed + s.seed_offset)),
          },
          Transform::default(),
          Visibility::default(),
        ))
        .id();

      let player = cmd
        .spawn(create_player(&asset_server, &mut layouts, spawned_level))
        .id();
      cmd.trigger(LevelUp { target: player });
      cmd.spawn((
        ChunkGenerator {
          chunk_size_world,
          load_around: player,
          load_radius: 3,
          unload_radius: 7,
        },
        ChunkMeshGenerator {
          tile_size_screen: tile_size_screen.as_vec2(),
          tile_size_world: tile_size_world.as_vec2(),
          tiles_per_chunk: level_descriptor.tiles_per_chunk,
        },
        Transform::default(),
        Visibility::default(),
        ChildOf(spawned_level),
      ));
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
  qry_levels: Query<(Entity, &Level)>,
) {
  for command in reader.read() {
    match command {
      LevelCommand::StartLevel(level_name) => {
        let handle: Handle<LevelAsset> =
          asset_server.load(format!("levels/{}.level.ron", level_name));
        cmd.spawn((
          Level {
            descriptor: handle,
            name: level_name.clone(),
          },
          Transform::default(),
          Visibility::default(),
        ));
      }
      LevelCommand::UnloadLevel(name) => {
        for (entity, level) in qry_levels.iter() {
          if &level.name != name {
            continue;
          }

          cmd.entity(entity).despawn();
        }
      }
    }
  }
}
