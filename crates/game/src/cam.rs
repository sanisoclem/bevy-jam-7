use bevy::{
  core_pipeline::tonemapping::{DebandDither, Tonemapping},
  post_process::bloom::Bloom,
  prelude::*,
};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, setup_camera);
  }
}

pub fn setup_camera(mut cmd: Commands) {
  cmd.spawn((
    Camera2d,
    Camera {
      clear_color: ClearColorConfig::Custom(Color::BLACK),
      ..default()
    },
    Tonemapping::TonyMcMapface,
    Bloom::default(),
    DebandDither::Enabled,
  ));
}
