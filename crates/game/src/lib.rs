use crate::{audio::AudioPlugin, testing::TestingPlugin};
use bevy::prelude::*;
use jam7::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
          canvas: Some("#main-canvas".into()),
          ..default()
        }),
        ..default()
      }))
      .add_plugins(Jam7Plugin)
      .add_plugins(cam::CameraPlugin)
      .add_plugins(AudioPlugin)
      .add_plugins(TestingPlugin);

    #[cfg(feature = "dev")]
    app.add_plugins(dev::DevGamePlugin);
  }
}

pub mod audio;
pub mod audio_engine;
pub mod cam;
#[cfg(feature = "dev")]
pub mod dev;
pub mod testing;
pub mod prelude {
  pub use crate::GamePlugin;
}
