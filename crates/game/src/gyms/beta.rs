use bevy::{color::palettes::css::RED, prelude::*, sprite::Anchor};
use jam7::{
  level::{
    Level, LevelCommand, LevelResourcesLoaded, asset::LevelAsset, render::ChunkMeshGenerator,
  },
  player::{Player, PlayerAnimationState, create_player_animations, create_player_controls},
};
use sys_cam::CameraTarget;
use sys_candy::{FireballBody, FireballExplosionBody, Shadow};
use sys_chonker::ChunkGenerator;
use sys_move::{IsoMovementStage, IsoWorldCoords, Moveable, Placeable};
use sys_procgen::ProceduralLevel;

pub struct BetaGymPlugin;

impl Plugin for BetaGymPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Startup, setup)
      .add_systems(Update, draw_gizmo)
      .add_observer(on_level_loaded);
  }
}
#[derive(Component, Reflect)]
pub struct EnemyPlaceholder {
  pub fire_timer: Timer,
}

pub fn setup(mut level_cmd: MessageWriter<LevelCommand>) {
  level_cmd.write(LevelCommand::LoadLevel("alpha".to_owned()));
}

fn on_level_loaded(
  evt: On<LevelResourcesLoaded>,
  mut cmd: Commands,
  qry: Query<&Level>,
  mut layouts: ResMut<Assets<TextureAtlasLayout>>,
  asset_server: Res<AssetServer>,
  levels: Res<Assets<LevelAsset>>,
) {
  let Some(level) = qry.get(evt.0).ok() else {
    return;
  };
  let level_descriptor = levels
    .get(&level.descriptor)
    .expect("level asset should be loaded");

  let tile_size_world = UVec2::new(
    level_descriptor.tileset.tile_width_world,
    level_descriptor.tileset.tile_height_world,
  );
  let chunk_size_world = (level_descriptor.tiles_per_chunk * tile_size_world).as_vec2();
  let spawned_level = cmd
    .spawn((
      Transform::default(),
      Visibility::default(),
      ProceduralLevel::from(level_descriptor),
      IsoMovementStage::from(level_descriptor),
    ))
    .id();

  let player = cmd
    .spawn(create_player(&asset_server, &mut layouts, spawned_level))
    .with_children(|x| {
      x.spawn((
        Shadow { radius: 100. },
        Transform::default().with_translation(-Vec3::Z),
        Visibility::default(),
      ));
    })
    .id();
  cmd.spawn((
    ChunkGenerator::from_player(player, chunk_size_world),
    ChunkMeshGenerator::from(level_descriptor),
    Transform::default(),
    Visibility::default(),
    ChildOf(spawned_level),
  ));

  cmd.spawn((
    FireballExplosionBody {
      radius: 50.,
      intensity: 1.0,
      lifetime: Timer::from_seconds(2.0, TimerMode::Repeating),
    },
    ChildOf(spawned_level),
    Transform::default().with_translation(Vec3::new(-500., 0.0, 1.0)),
    Visibility::default(),
  ));
  cmd.spawn((
    FireballExplosionBody {
      radius: 150.,
      intensity: 1.0,
      lifetime: Timer::from_seconds(2.0, TimerMode::Repeating),
    },
    ChildOf(spawned_level),
    Transform::default().with_translation(Vec3::new(-250., 0.0, 1.0)),
    Visibility::default(),
  ));
  cmd.spawn((
    FireballBody {
      radius: 10.,
      intensity: 1.0,
    },
    ChildOf(spawned_level),
    Transform::default().with_translation(Vec3::new(250., 0.0, 1.0)),
    Visibility::default(),
  ));
  cmd.spawn((
    FireballBody {
      radius: 50.,
      intensity: 1.0,
    },
    ChildOf(spawned_level),
    Transform::default().with_translation(Vec3::new(500., 0.0, 1.0)),
    Visibility::default(),
  ));

  cmd
    .entity(evt.0)
    .despawn_children()
    .replace_children(&[spawned_level]);
}

pub fn draw_gizmo(mut gizmo: Gizmos) {
  gizmo.ellipse_2d(
    Isometry2d::from_translation(Vec2::new(500., 0.)),
    Vec2::new(50. * 0.7, 50. * 0.35),
    Color::from(RED),
  );
  gizmo.ellipse_2d(
    Isometry2d::from_translation(Vec2::new(250., 0.)),
    Vec2::new(10. * 0.7, 10. * 0.35),
    Color::from(RED),
  );
  gizmo.ellipse_2d(
    Isometry2d::from_translation(Vec2::new(-500., 0.)),
    Vec2::new(50. * 0.7, 50. * 0.35),
    Color::from(RED),
  );
  gizmo.ellipse_2d(
    Isometry2d::from_translation(Vec2::new(-250., 0.)),
    Vec2::new(150. * 0.7, 150. * 0.35),
    Color::from(RED),
  );
}

pub fn create_player(
  asset_server: &AssetServer,
  layouts: &mut Assets<TextureAtlasLayout>,
  spawn_parent: Entity,
) -> impl Bundle {
  (
    (
      Player,
      ChildOf(spawn_parent),
      CameraTarget,
      Transform::default().with_scale(Vec3::splat(0.1)),
      Visibility::default(),
      Moveable::default(),
      Placeable::mid(IsoWorldCoords::default()),
    ),
    (
      create_player_animations(asset_server, layouts),
      Anchor(Vec2::new(0., -0.42)),
      PlayerAnimationState::default(),
    ),
    create_player_controls(),
  )
}
