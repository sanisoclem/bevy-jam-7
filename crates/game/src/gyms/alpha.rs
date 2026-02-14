use bevy::{prelude::*, sprite::Anchor, time::Stopwatch};
use jam7::{
  level::{
    Level, LevelCommand, LevelResourcesLoaded, asset::LevelAsset, render::ChunkMeshGenerator,
  },
  player::{Player, PlayerAnimationState, create_player_animations, create_player_controls},
};
use sys_cam::CameraTarget;
use sys_candy::Shadow;
use sys_chonker::ChunkGenerator;
use sys_combat::{Combatant, DeathBehavior, HitTestableShape, KillCounter};
use sys_enemy::{EnemySpawner, EnemySpawnerState};
use sys_magic::{SpellBook, SpellBookState};
use sys_move::{IsoMovementStage, IsoWorldCoords, Moveable, Placeable};
use sys_procgen::ProceduralLevel;
use sys_prog::levelup::LevelUp;
use utils::diff::TEAM_PLAYER;

pub struct AlphaGymPlugin;

impl Plugin for AlphaGymPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Startup, setup)
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
        Shadow { radius: 150. },
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
  cmd
    .entity(evt.0)
    .despawn_children()
    .replace_children(&[spawned_level]);

  cmd.trigger(LevelUp { target: player });
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
      KillCounter { kills: 0 },
      CameraTarget,
      Transform::default().with_scale(Vec3::splat(0.3)),
      Visibility::default(),
      Moveable::default(),
      Placeable::mid(IsoWorldCoords::default()),
    ),
    (SpellBook::default(), SpellBookState::default()),
    (
      EnemySpawner {
        spawn_parent,
        despawn_radius: 1000,
        no_spawn_radius: 400,
        spawn_radius: 700,
        initial_cooldown: 1.,
        cooldown_decay_rate: 1.5,
      },
      EnemySpawnerState {
        stopwatch: Stopwatch::new(),
        cooldown: Timer::from_seconds(0.5, TimerMode::Once),
      },
    ),
    (Combatant {
      max_hp: 100000,
      hitbox: HitTestableShape::Circle { radius: 21.0 },
      team: TEAM_PLAYER,
      regen: 0,
      regen_delay: 0,
      death_behavior: DeathBehavior::Respawn(
        Timer::from_seconds(5.0, TimerMode::Once),
        Timer::from_seconds(2.0, TimerMode::Once),
      ),
    }),
    (
      create_player_animations(asset_server, layouts),
      Anchor(Vec2::new(0., -0.42)),
      PlayerAnimationState::default(),
    ),
    create_player_controls(),
  )
}
