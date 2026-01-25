use crate::audio::{AudioChannelLayout, AudioChannelSettings, AudioCommand, AudioExtensions};
use bevy::{platform::collections::HashMap, prelude::*};
use jam7::prelude::*;
use macros::AudioLibrary;

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
      .configure_audio::<GameAudioLibrary, GameAudioChannels>()
      .add_systems(Update, toggle_music);

    #[cfg(feature = "dev")]
    app.add_plugins(dev::DevGamePlugin);
  }
}

#[derive(AudioLibrary, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum GameAudioLibrary {
  #[looped("menu.ogg")]
  Menu,
  #[intro_looped("battle_0.ogg", "battle_1.ogg")]
  Battle,
  #[once("t1.ogg")]
  T1,
  #[once("t2.ogg")]
  T2,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum GameAudioChannels {
  Music,
  Effects,
  UI,
}
impl AudioChannelLayout for GameAudioChannels {
  fn initial_state() -> HashMap<Self, audio::AudioChannelSettings> {
    let mut h = HashMap::new();
    h.insert(
      GameAudioChannels::Music,
      AudioChannelSettings {
        volume: 1.0,
        ..Default::default()
      },
    );
    h.insert(
      GameAudioChannels::Effects,
      AudioChannelSettings {
        volume: 1.0,
        ..Default::default()
      },
    );
    h
  }
}

fn toggle_music(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut cmds: MessageWriter<audio::AudioCommand<GameAudioLibrary, GameAudioChannels>>,
) {
  if keyboard_input.just_pressed(KeyCode::KeyM) {
    cmds.write(AudioCommand::ReplaceAllAndFadeInto(
      GameAudioLibrary::Menu,
      GameAudioChannels::Music,
    ));
  }
  if keyboard_input.just_pressed(KeyCode::KeyN) {
    cmds.write(AudioCommand::ReplaceAllAndFadeInto(
      GameAudioLibrary::Battle,
      GameAudioChannels::Music,
    ));
  }

  if keyboard_input.just_pressed(KeyCode::KeyK) {
    cmds.write(AudioCommand::InsertOnce(
      GameAudioLibrary::T1,
      GameAudioChannels::Effects,
    ));
  }
  if keyboard_input.just_pressed(KeyCode::KeyL) {
    cmds.write(AudioCommand::InsertOnce(
      GameAudioLibrary::T2,
      GameAudioChannels::Effects,
    ));
  }
  if keyboard_input.just_pressed(KeyCode::KeyS) {
    cmds.write(AudioCommand::StopAllInChannel(GameAudioChannels::Music));
  }
}

pub mod audio;
pub mod cam;
#[cfg(feature = "dev")]
pub mod dev;
pub mod prelude {
  pub use crate::GamePlugin;
}
