pub(crate) mod chunk;
pub(crate) mod procgen;
pub(crate) mod tile;
pub(crate) mod tileset;

use bevy::{self, prelude::*};
use chunk::{ChunkGenerator, despawn_chunks, spawn_chunks, update_chunk_spawner_pos};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_message::<LevelCommand>()
      .init_asset::<tileset::TilesetDefinition>()
      .init_asset_loader::<tileset::TilesetDefinitionLoader>()
      .add_systems(
        Update,
        (
          spawn_chunks,
          despawn_chunks,
          process_level_commands,
          update_chunk_spawner_pos,
        ),
      );
  }
}

#[derive(Clone, Debug)]
pub struct LevelDescriptor {
  pub tileset_name: String,
  pub chunk_size: u32,
  pub seed: i64,
}

#[derive(Debug, Message)]
pub enum LevelCommand {
  StartLevel(u32, LevelDescriptor),
  UnloadLevel(u32),
}

#[derive(Component)]
pub struct ProceduralLevel {
  pub id: u32,
}

pub fn process_level_commands(
  mut cmd: Commands,
  asset_server: Res<AssetServer>,
  mut reader: MessageReader<LevelCommand>,
  qry_levels: Query<(Entity, &ProceduralLevel)>,
) {
  for command in reader.read() {
    match command {
      LevelCommand::StartLevel(level_id, descriptor) => {
        let tileset = asset_server.load(format!("tilesets/{}.png", descriptor.tileset_name));
        cmd.spawn((
          ProceduralLevel { id: *level_id },
          Transform::default(),
          Visibility::default(),
          ChunkGenerator {
            owner_id: *level_id,
            chunk_size: descriptor.chunk_size,
            seed: descriptor.seed,
            tile_size: UVec2::new(32, 16), // TODO: get this from the tileset
            tileset,
          },
        ));
      }
      LevelCommand::UnloadLevel(level_id) => {
        for (entity, level) in qry_levels.iter() {
          if level.id != *level_id {
            continue;
          }

          cmd.entity(entity).despawn();
        }
      }
    }
  }
}
