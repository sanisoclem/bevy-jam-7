use crate::gyms::AlphaGymPlugin;
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
      // .add_plugins(BetaGymPlugin);
      .add_plugins(AlphaGymPlugin);

    #[cfg(feature = "dev")]
    app.add_plugins(dev::DevGamePlugin);
  }
}

#[cfg(feature = "dev")]
pub mod dev;
pub mod gyms;
pub mod prelude {
  pub use crate::GamePlugin;
}
