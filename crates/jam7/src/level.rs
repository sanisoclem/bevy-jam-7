pub mod asset;
pub mod render;

use asset::LevelAsset;
use bevy::{prelude::*, sprite_render::Material2dPlugin};
use render::{ChunkMaterial, IsoTilemapChunkMeshCache, render_tile_data};
use sys_chonker::SysChonkerPlugin;
use sys_prog::death::{RequestGameRestart, ShowDeathUi};

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
      .add_observer(on_game_restart)
      .add_systems(
        Update,
        (check_level_loaded, process_level_commands, render_tile_data),
      )
      .add_observer(on_show_death_ui);
  }
}

#[derive(Debug, Message)]
pub enum LevelCommand {
  LoadLevel(String),
  UnloadLevel(String),
}

#[derive(Component, Debug)]
pub struct Level {
  pub name: String,
  pub descriptor: Handle<LevelAsset>,
}

#[derive(EntityEvent, Debug, Clone)]
pub struct LevelResourcesLoaded(pub Entity);

pub fn check_level_loaded(
  mut cmd: Commands,
  mut ev_asset: MessageReader<AssetEvent<LevelAsset>>,
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
      cmd.trigger(LevelResourcesLoaded(entity));
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
      LevelCommand::LoadLevel(level_name) => {
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

fn on_show_death_ui(_trigger: On<ShowDeathUi>, mut level_cmd: MessageWriter<LevelCommand>) {
  level_cmd.write(LevelCommand::UnloadLevel("alpha".to_owned()));
}

pub fn on_game_restart(_evt: On<RequestGameRestart>, mut level_cmd: MessageWriter<LevelCommand>) {
  info!("Restart this game!");
  level_cmd.write(LevelCommand::LoadLevel("alpha".to_owned()));
}
