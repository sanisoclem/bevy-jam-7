use bevy::{
  color::palettes::css::{GREEN, PURPLE},
  prelude::*,
  sprite::Text2dShadow,
};
use jam7::{
  level::{ChunkSpawner, LevelChunk, LevelCommand, LevelDescriptor, LevelId},
  player::{Player, PlayerCommand, PlayerId},
};

pub struct TestingPlugin;

impl Plugin for TestingPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Update, (generate_level_chunk_mesh, generate_player_mesh))
      .add_systems(Startup, setup);
  }
}

pub fn setup(
  mut player_cmd: MessageWriter<PlayerCommand>,
  mut level_cmd: MessageWriter<LevelCommand>,
) {
  level_cmd.write(LevelCommand::StartLevel(
    LevelId(0),
    LevelDescriptor {
      chunk_size: 230.,
      seed: 0,
    },
  ));

  player_cmd.write(PlayerCommand::SpawnPlayer(PlayerId(0), Vec2::splat(0.)));
}

pub fn generate_level_chunk_mesh(
  asset_server: Res<AssetServer>,
  mut cmd: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
  qry: Query<(&LevelChunk, Entity), Without<Mesh2d>>,
) {
  let font = asset_server.load("fonts/FiraSans-Bold.ttf");
  let text_font = TextFont {
    font: font.clone(),
    font_size: 50.0,
    ..default()
  };
  for (chunk, entity) in qry {
    // TODO: generate create resource to track meshes and materials used by chunks and
    // unload them when no longer needed
    cmd.entity(entity).insert((
      Text2d::new(format!("xx{},{}", chunk.id.x(), chunk.id.y())),
      text_font.clone(),
      TextLayout::new_with_justify(Justify::Center),
      TextBackgroundColor(Color::BLACK.with_alpha(0.5)),
      Text2dShadow::default(),
      Mesh2d(meshes.add(Rectangle::from_size(Vec2::splat(chunk.size)))),
      MeshMaterial2d(materials.add(Color::from(PURPLE))),
    ));
  }
}

pub fn generate_player_mesh(
  mut cmd: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
  qry: Query<Entity, (Without<Mesh2d>, With<Player>)>,
) {
  for entity in qry {
    cmd.entity(entity).insert((
      Mesh2d(meshes.add(Rectangle::from_size(Vec2::splat(30.0)))),
      MeshMaterial2d(materials.add(Color::from(GREEN))),
      ChunkSpawner {
        level: LevelId(0),
        load_radius: 1,
        unload_radius: 4,
      },
    ));
  }
}
