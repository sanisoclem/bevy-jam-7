#[cfg(feature = "dev")]
use crate::debug::DebugPlugin;
use crate::{level::LevelPlugin, player::PlayerPlugin};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use sys_audio::SysAudioPlugin;
use sys_cam::SysCamPlugin;
use sys_candy::SysCandyPlugin;
use sys_combat::SysCombatPlugin;
use sys_enemy::SysEnemyPlugin;
use sys_magic::SysMagicPlugin;
use sys_move::SysMovePlugin;
use sys_prog::SysProgPlugin;

pub struct Jam7Plugin;

impl Plugin for Jam7Plugin {
  fn build(&self, app: &mut App) {
    app.add_plugins((
      EnhancedInputPlugin,
      LevelPlugin,
      PlayerPlugin,
      SysAudioPlugin,
      SysMovePlugin,
      SysCamPlugin,
      SysCombatPlugin,
      SysMagicPlugin,
      SysEnemyPlugin,
      SysProgPlugin,
      SysCandyPlugin,
    ));

    #[cfg(feature = "dev")]
    app.add_plugins(DebugPlugin);
  }
}
