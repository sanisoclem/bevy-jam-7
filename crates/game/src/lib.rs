use bevy::prelude::*;
use jam7::prelude::*;
pub struct GamePlugin;

impl Plugin for GamePlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins(Jam7Plugin);
  }
}
pub mod prelude {}
