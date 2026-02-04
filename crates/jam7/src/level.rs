pub(crate) mod chunk;
pub(crate) mod procgen;
pub(crate) mod tile;

use bevy::{
  image::{ImageArrayLayout, ImageLoaderSettings},
  prelude::*,
  sprite_render::Material2dPlugin,
};
use chunk::{
  ChunkGenerator, ChunkMaterial, IsoTilemapChunkMeshCache, despawn_chunks,
  generate_level_chunk_mesh, spawn_chunks, update_chunk_spawner_pos,
};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<IsoTilemapChunkMeshCache>()
      .add_plugins(Material2dPlugin::<ChunkMaterial>::default())
      .add_message::<LevelCommand>()
      .add_systems(
        Update,
        (
          spawn_chunks,
          despawn_chunks,
          process_level_commands,
          generate_level_chunk_mesh,
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
        let tileset = asset_server.load_with_settings(
          format!("tilesets/{}.png", descriptor.tileset_name),
          |settings: &mut ImageLoaderSettings| {
            // The tileset texture is expected to be an array of tile textures, so we tell the
            // `ImageLoader` that our texture is composed of 4 stacked tile images.
            settings.array_layout = Some(ImageArrayLayout::RowCount { rows: 11 });
          },
        );
        cmd.spawn((
          ProceduralLevel { id: *level_id },
          Transform::default(),
          Visibility::default(),
          ChunkGenerator {
            owner_id: *level_id,
            chunk_size: descriptor.chunk_size,
            seed: descriptor.seed,
            tile_size: UVec2::new(64, 32), // TODO: get this from the tileset
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
