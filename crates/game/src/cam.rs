use bevy::{
  core_pipeline::tonemapping::{DebandDither, Tonemapping},
  post_process::bloom::Bloom,
  prelude::*,
};
use bevy_enhanced_input::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_input_context::<PlayerCamera>()
      .add_systems(Startup, setup_camera)
      .add_observer(apply_game_camera_zoom);
  }
}

#[derive(InputAction)]
#[action_output(f32)]
pub struct ZoomCamera;

#[derive(Component, Debug)]
pub struct PlayerCamera;

pub fn setup_camera(mut cmd: Commands) {
  cmd.spawn((
    Camera2d,
    Camera {
      clear_color: ClearColorConfig::Custom(Color::BLACK),
      ..default()
    },
    Transform::default(),
    Tonemapping::TonyMcMapface,
    Bloom::default(),
    DebandDither::Enabled,
    PlayerCamera,
    actions!(
      PlayerCamera[(
        Action::<ZoomCamera>::new(),
        DeadZone::default(),
        Bindings::spawn((
          Spawn((Binding::mouse_wheel(), SwizzleAxis::YXZ)),
          Bidirectional::new(GamepadButton::DPadUp, GamepadButton::DPadDown),
        )),
      )]
    ),
  ));
}

pub fn apply_game_camera_zoom(
  movement: On<Fire<ZoomCamera>>,
  mut cameras: Query<&mut Transform, With<PlayerCamera>>,
) {
  let mut transform = cameras.get_mut(movement.context).unwrap();
  transform.scale *= 1. + movement.value * 0.1;
  if transform.scale.x <= 0.0 {
    transform.scale = Vec3::splat(1.0);
  }
}
