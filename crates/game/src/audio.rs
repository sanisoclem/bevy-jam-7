use crate::audio_engine::{
  AudioChannelLayout, AudioChannelSettings, AudioCommand, AudioExtensions,
};
use bevy::{platform::collections::HashMap, prelude::*};
use macros::AudioLibrary;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
  fn build(&self, app: &mut App) {
    app
      .configure_audio::<GameAudioLibrary, GameAudioChannels>()
      .add_systems(Update, toggle_music);
  }
}

pub type GameAudioCommand = AudioCommand<GameAudioLibrary, GameAudioChannels>;

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
  fn initial_state() -> HashMap<Self, AudioChannelSettings> {
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
  mut cmds: MessageWriter<GameAudioCommand>,
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
  if keyboard_input.just_pressed(KeyCode::KeyP) {
    cmds.write(AudioCommand::StopAllInChannel(GameAudioChannels::Music));
  }
}
