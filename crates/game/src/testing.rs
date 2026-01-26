use bevy::{
  camera::visibility::NoFrustumCulling,
  color::palettes::css::{BLUE, GREEN, PURPLE, RED, YELLOW},
  prelude::*,
  sprite::Text2dShadow,
};
use bevy_enhanced_input::prelude::*;
use jam7::{
  level::{ChunkSpawner, LevelChunk, LevelCommand, LevelDescriptor, LevelId},
  player::{ActionMovePlayer, Player, PlayerCommand, PlayerId},
};

pub struct TestingPlugin;

impl Plugin for TestingPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(
        Update,
        (
          generate_level_chunk_mesh,
          generate_player_mesh,
          update_camera,
        ),
      )
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
      chunk_size: 2000.,
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
  let colors: Vec<_> = vec![PURPLE, RED, GREEN, BLUE, YELLOW]
    .into_iter()
    .map(Color::from)
    .collect();
  for (chunk, entity) in qry {
    // TODO: generate create resource to track meshes and materials used by chunks and
    // unload them when no longer needed
    let i = (chunk.id.x() * chunk.id.y()).unsigned_abs() as usize;
    cmd.entity(entity).insert((
      Text2d::new(format!("{},{}", chunk.id.x(), chunk.id.y())),
      NoFrustumCulling,
      text_font.clone(),
      TextLayout::new_with_justify(Justify::Center),
      TextBackgroundColor(Color::BLACK.with_alpha(0.5)),
      Text2dShadow::default(),
      Mesh2d(meshes.add(Rectangle::from_size(Vec2::splat(chunk.size)))),
      MeshMaterial2d(materials.add(*colors.get(i % colors.len()).unwrap())),
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
    let mesh = Rectangle::from_size(Vec2::splat(30.0)).mesh().build();
    cmd.entity(entity).insert((
      Mesh2d(meshes.add(mesh)),
      MeshMaterial2d(materials.add(Color::from(GREEN))),
      ChunkSpawner {
        level: LevelId(0),
        load_radius: 1,
        unload_radius: 2,
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
