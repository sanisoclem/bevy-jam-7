use bevy::prelude::*;

use crate::level::LevelPlugin;
pub struct Jam7Plugin;

impl Plugin for Jam7Plugin {
  fn build(&self, app: &mut App) {
    app.add_plugins(LevelPlugin);
  }
}
