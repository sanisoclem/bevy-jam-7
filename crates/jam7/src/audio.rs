use bevy::{platform::collections::HashMap, prelude::*};
use macros::AudioLibrary;
use sys_audio::{AudioChannelLayout, AudioChannelSettings, AudioCommand, SysAudioPlugin};
use sys_prog::boss::{BossKilled, BossSpawned};

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(SysAudioPlugin::<GameAudioLibrary, GameAudioChannels>::default())
      // .add_systems(Update, toggle_music)
      .add_observer(on_boss_spawned)
      .add_observer(on_boss_killed);
  }
}

pub type GameAudioCommand = AudioCommand<GameAudioLibrary, GameAudioChannels>;

#[derive(AudioLibrary, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum GameAudioLibrary {
  #[looped("menu.ogg")]
  Menu,
  #[intro_looped("battle_0.ogg", "battle_1.ogg")]
  Battle,
  #[intro_looped("audio/intro-lofi.ogg", "audio/loop-lofi.ogg")]
  Lofi,
  #[intro_looped("audio/intro-boss.ogg", "audio/loop-boss.ogg")]
  Boss,
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

fn on_boss_spawned(_evt: On<BossSpawned>, mut music_cmd: MessageWriter<GameAudioCommand>) {
  music_cmd.write(GameAudioCommand::ReplaceAllAndFadeInto(
    GameAudioLibrary::Boss,
    GameAudioChannels::Music,
  ));
}
fn on_boss_killed(_evt: On<BossKilled>, mut music_cmd: MessageWriter<GameAudioCommand>) {
  music_cmd.write(GameAudioCommand::ReplaceAllAndFadeInto(
    GameAudioLibrary::Lofi,
    GameAudioChannels::Music,
  ));
}

// fn toggle_music(
//   keyboard_input: Res<ButtonInput<KeyCode>>,
//   mut cmds: MessageWriter<GameAudioCommand>,
// ) {
//   if keyboard_input.just_pressed(KeyCode::KeyM) {
//     cmds.write(AudioCommand::ReplaceAllAndFadeInto(
//       GameAudioLibrary::Menu,
//       GameAudioChannels::Music,
//     ));
//   }
//   if keyboard_input.just_pressed(KeyCode::KeyN) {
//     cmds.write(AudioCommand::ReplaceAllAndFadeInto(
//       GameAudioLibrary::Battle,
//       GameAudioChannels::Music,
//     ));
//   }
//
//   if keyboard_input.just_pressed(KeyCode::KeyK) {
//     cmds.write(AudioCommand::InsertOnce(
//       GameAudioLibrary::T1,
//       GameAudioChannels::Effects,
//     ));
//   }
//   if keyboard_input.just_pressed(KeyCode::KeyL) {
//     cmds.write(AudioCommand::InsertOnce(
//       GameAudioLibrary::T2,
//       GameAudioChannels::Effects,
//     ));
//   }
//   if keyboard_input.just_pressed(KeyCode::KeyP) {
//     cmds.write(AudioCommand::StopAllInChannel(GameAudioChannels::Music));
//   }
// }
