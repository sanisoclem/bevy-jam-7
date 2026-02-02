pub(crate) mod chunk;
pub(crate) mod procgen;
pub(crate) mod tile;

use bevy::prelude::*;
use chunk::{ChunkGenerator, despawn_chunks, spawn_chunks};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
  fn build(&self, app: &mut App) {
    app.add_message::<LevelCommand>().add_systems(
      Update,
      (spawn_chunks, despawn_chunks, process_level_commands),
    );
  }
}

#[derive(Clone, Debug)]
pub struct LevelDescriptor {
  pub tileset_name: String,
  pub chunk_size: f32,
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
  mut reader: MessageReader<LevelCommand>,
  qry_levels: Query<(Entity, &ProceduralLevel)>,
) {
  for command in reader.read() {
    match command {
      LevelCommand::StartLevel(level_id, descriptor) => {
        cmd.spawn((
          ProceduralLevel { id: *level_id },
          Transform::default(),
          Visibility::default(),
          ChunkGenerator {
            owner_id: *level_id,
            chunk_size: descriptor.chunk_size,
            seed: descriptor.seed,
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
