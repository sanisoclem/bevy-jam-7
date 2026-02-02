use bevy::{
  camera::visibility::NoFrustumCulling,
  prelude::*,
  render::render_resource::AsBindGroup,
  shader::ShaderRef,
  sprite_render::{AlphaMode2d, Material2d, Material2dPlugin},
};
use bevy_enhanced_input::prelude::*;
use jam7::{
  level::{LevelCommand, LevelDescriptor},
  player::{ActionMovePlayer, Player, PlayerCommand, PlayerId},
  prelude::*,
};

use crate::{
  audio::{GameAudioChannels, GameAudioCommand, GameAudioLibrary},
  audio_engine::AudioCommand,
};

const LEVEL_ID: u32 = 0;
pub struct AlphaGymPlugin;

impl Plugin for AlphaGymPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(Material2dPlugin::<ChunkMaterial>::default())
      .add_systems(
        Update,
        (
          generate_level_chunk_mesh,
          update_chunk_player_pos,
          generate_player_mesh,
          update_camera,
        ),
      )
      .add_systems(Startup, setup);
  }
}

pub fn setup(
  mut cmds: MessageWriter<GameAudioCommand>,
  mut player_cmd: MessageWriter<PlayerCommand>,
  mut level_cmd: MessageWriter<LevelCommand>,
) {
  cmds.write(AudioCommand::ReplaceAllAndFadeInto(
    GameAudioLibrary::Menu,
    GameAudioChannels::Music,
  ));
  level_cmd.write(LevelCommand::StartLevel(
    LEVEL_ID,
    LevelDescriptor {
      tileset_name: "alpha".to_owned(),
      chunk_size: 2000.,
      seed: 0,
    },
  ));

  player_cmd.write(PlayerCommand::SpawnPlayer(PlayerId(0), Vec2::splat(0.)));
}

pub fn generate_player_mesh(
  mut cmd: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  // mut materials: ResMut<Assets<ColorMaterial>>,
  qry: Query<Entity, (Without<Mesh2d>, With<Player>)>,
) {
  for entity in qry {
    let mesh = Rectangle::from_size(Vec2::splat(30.0)).mesh().build();
    cmd.entity(entity).insert((
      Mesh2d(meshes.add(mesh)),
      // MeshMaterial2d(materials.add(Color::from(GREEN))),
      ChunkSpawner {
        owner_id: LEVEL_ID,
        load_radius: 2,
        unload_radius: 3,
      },
      actions!(
        Player[(
          Action::<ActionMovePlayer>::new(),
          DeadZone::default(), // Applies non-uniform normalization.
          bindings![
            // Keyboard keys captured as `bool`, but the output of `Movement` is defined as `Vec2`,
            // so you need to assign keys to axes using swizzle to reorder them and negation.
            (KeyCode::KeyW, SwizzleAxis::YXZ),
            (KeyCode::KeyA, Negate::all()),
            (KeyCode::KeyS, Negate::all(), SwizzleAxis::YXZ),
            KeyCode::KeyD,
            // In Bevy sticks split by axes and captured as 1-dimensional inputs,
            // so Y stick needs to be sweezled into Y axis.
            GamepadAxis::LeftStickX,
            (GamepadAxis::LeftStickY, SwizzleAxis::YXZ),
          ]
        )]
      ),
    ));
  }
}

const CAMERA_DECAY_RATE: f32 = 2.;
fn update_camera(
  mut camera: Single<&mut Transform, (With<Camera2d>, Without<Player>)>,
  player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
  time: Res<Time>,
) {
  let Vec3 { x, y, .. } = player.translation;
  let direction = Vec3::new(x, y, camera.translation.z);

  // Applies a smooth effect to camera movement using stable interpolation
  // between the camera position and the player position on the x and y axes.
  camera
    .translation
    .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());
}

pub fn generate_level_chunk_mesh(
  mut cmd: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ChunkMaterial>>,
  qry: Query<(&LevelChunk, Entity), Without<Mesh2d>>,
) {
  for (chunk, entity) in qry {
    // TODO: generate create resource to track meshes and materials used by chunks and
    // unload them when no longer needed
    cmd.entity(entity).insert((
      NoFrustumCulling,
      Mesh2d(meshes.add(Rectangle::from_size(Vec2::splat(chunk.size)))),
      MeshMaterial2d(materials.add(ChunkMaterial {
        id: IVec4::new(chunk.id.x(), chunk.id.y(), 0, 0),
        player_pos: Vec2::default().extend(chunk.size).extend(0.),
      })),
    ));
  }
}

pub fn update_chunk_player_pos(
  mut materials: ResMut<Assets<ChunkMaterial>>,
  qry: Query<&Transform, With<Player>>,
) {
  let Ok(player_transform) = qry.single() else {
    return;
  };

  for mat in materials.iter_mut() {
    mat.1.player_pos.x = player_transform.translation.x;
    mat.1.player_pos.y = player_transform.translation.y;
  }
}

const SHADER_ASSET_PATH: &str = "shaders/chunk.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ChunkMaterial {
  #[uniform(0)]
  id: IVec4,
  #[uniform(1)]
  player_pos: Vec4,
}

impl Material2d for ChunkMaterial {
  fn vertex_shader() -> ShaderRef {
    SHADER_ASSET_PATH.into()
  }
  fn fragment_shader() -> ShaderRef {
    SHADER_ASSET_PATH.into()
  }

  fn alpha_mode(&self) -> AlphaMode2d {
    AlphaMode2d::Blend
  }
}
