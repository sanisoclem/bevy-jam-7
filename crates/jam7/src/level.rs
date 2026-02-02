mod chunk;
mod procgen;

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
  pub chunk_generator: ChunkGenerator,
}

#[derive(Debug, Message)]
pub enum LevelCommand {
  StartLevel(LevelId, LevelDescriptor),
  UnloadLevel(LevelId),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Reflect)]
pub struct LevelId(pub i32);

#[derive(Component)]
pub struct ProceduralLevel {
  pub id: LevelId,
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
          descriptor.chunk_generator.clone(),
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
