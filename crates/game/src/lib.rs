use crate::audio::{AudioChannelLayout, AudioExtensions};
use bevy::prelude::*;
use jam7::prelude::*;
use macros::AudioLibrary;

pub struct GamePlugin;

impl Plugin for GamePlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(DefaultPlugins)
      .add_plugins(Jam7Plugin)
      .add_plugins(cam::CameraPlugin)
      .configure_audio::<GameAudioLibrary, GameAudioChannels>();

    #[cfg(feature = "dev")]
    app.add_plugins(dev::DevGamePlugin);
  }
}

#[derive(AudioLibrary, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum GameAudioLibrary {
  #[intro_looped("menu_intro.ogg", "menu.ogg")]
  Menu,
}

pub enum GameAudioChannels {
  Music,
  Effects,
  UI,
}
impl AudioChannelLayout for GameAudioChannels {}

pub mod audio;
pub mod cam;
#[cfg(feature = "dev")]
pub mod dev;
pub mod prelude {
  pub use crate::GamePlugin;
}
