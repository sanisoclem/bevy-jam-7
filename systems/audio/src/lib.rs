use bevy::{platform::collections::HashMap, prelude::*};
use macros::AudioLibrary;

mod engine;

pub use engine::*;

pub struct SysAudioPlugin;

impl Plugin for SysAudioPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins(AudioPlugin::<GameAudioLibrary, GameAudioChannels>::default());
    // .add_systems(Update, toggle_music)
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
  #[once("audio/POWERUP-A9DDF1B8730ECF3C.ogg")]
  ButtonEffect,
  #[once("audio/LASER-FC7F6DC1039B9903.ogg")]
  Laser,
  #[once("audio/HIT-95E68C43E9618ADF.ogg")]
  Hit1,
  #[once("audio/EXPLOSION-424ED9E91B552907.ogg")]
  Explosion1,
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
