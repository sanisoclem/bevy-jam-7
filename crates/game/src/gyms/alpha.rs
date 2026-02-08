use bevy::prelude::*;
use jam7::level::LevelCommand;

pub struct AlphaGymPlugin;

impl Plugin for AlphaGymPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, setup);
  }
}

pub fn setup(mut level_cmd: MessageWriter<LevelCommand>) {
  level_cmd.write(LevelCommand::StartLevel("alpha".to_owned()));
}

// pub fn generate_player_mesh(
//   mut cmd: Commands,
//   mut meshes: ResMut<Assets<Mesh>>,
//   mut materials: ResMut<Assets<ColorMaterial>>,
//   qry: Query<Entity, (Without<Mesh2d>, With<Player>)>,
// ) {
//   for entity in qry {
//     let mesh = Rectangle::from_size(Vec2::splat(32.0)).mesh().build();
//     cmd.entity(entity).insert((
//       Mesh2d(meshes.add(mesh)),
//       MeshMaterial2d(materials.add(Color::from(GREEN))),
//       ChunkSpawner {
//         owner_id: LEVEL_ID,
//         load_radius: 5,
//         unload_radius: 8,
//       },
//       actions!(
//         Player[(
//           Action::<ActionMovePlayer>::new(),
//           DeadZone::default(), // Applies non-uniform normalization.
//           bindings![
//             // Keyboard keys captured as `bool`, but the output of `Movement` is defined as `Vec2`,
//             // so you need to assign keys to axes using swizzle to reorder them and negation.
//             (KeyCode::KeyW, SwizzleAxis::YXZ),
//             (KeyCode::KeyA, Negate::all()),
//             (KeyCode::KeyS, Negate::all(), SwizzleAxis::YXZ),
//             KeyCode::KeyD,
//             // In Bevy sticks split by axes and captured as 1-dimensional inputs,
//             // so Y stick needs to be sweezled into Y axis.
//             GamepadAxis::LeftStickX,
//             (GamepadAxis::LeftStickY, SwizzleAxis::YXZ),
//           ]
//         )]
//       ),
//     ));
//   }
// }

// const CAMERA_DECAY_RATE: f32 = 2.;
// fn update_camera(
//   mut camera: Single<&mut Transform, (With<Camera2d>, Without<Player>)>,
//   player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
//   time: Res<Time>,
// ) {
//   let Vec3 { x, y, .. } = player.translation;
//   let direction = Vec3::new(x, y, camera.translation.z);
//
//   camera
//     .translation
//     .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());
// }
