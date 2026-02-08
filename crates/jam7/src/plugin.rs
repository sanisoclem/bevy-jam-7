use crate::{level::LevelPlugin, player::PlayerPlugin};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use sys_cam::SysCamPlugin;
use sys_move::SysMovePlugin;

pub struct Jam7Plugin;

impl Plugin for Jam7Plugin {
  fn build(&self, app: &mut App) {
    app.add_plugins((
      EnhancedInputPlugin,
      LevelPlugin,
      PlayerPlugin,
      SysMovePlugin,
      SysCamPlugin,
    ));
  }
}
