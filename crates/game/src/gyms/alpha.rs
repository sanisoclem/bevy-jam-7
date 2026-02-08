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
