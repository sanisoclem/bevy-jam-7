use crate::{level::LevelPlugin, player::PlayerPlugin};
use bevy::prelude::*;

pub struct Jam7Plugin;

impl Plugin for Jam7Plugin {
  fn build(&self, app: &mut App) {
    app.add_plugins((LevelPlugin, PlayerPlugin));
  }
}
