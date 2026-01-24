use bevy::{
  audio::{AddAudioSource as _, Source},
  platform::collections::HashMap,
  prelude::*,
};
use rodio::source::from_iter;
use std::hash::Hash;
use utils::assets::{AssetBarrier, AssetBarrierGuard};

pub trait AudioExtensions {
  fn configure_audio<L: AudioLibrary, C: AudioChannelLayout>(&mut self) -> &mut Self;
}

impl AudioExtensions for App {
  fn configure_audio<L: AudioLibrary, C: AudioChannelLayout>(&mut self) -> &mut Self {
    self
      .add_audio_source::<ProcessedAudio>()
      .init_resource::<AudioLibraryResource<L, C>>()
      .add_message::<AudioCommand<L, C>>()
      .add_systems(
        Update,
        process_assets::<L, C>.run_if(should_process_assets::<L, C>),
      )
      .add_systems(
        Update,
        process_audio_commands::<L, C>.run_if(assets_loaded::<L, C>),
      )
      .add_systems(
        Update,
        (
          update_channel_volume::<L, C>,
          update_entity_volumes::<L, C>,
          update_sink_volumes::<L, C>,
        )
          .chain(),
      )
    // .add_systems(Update, fade)
  }
}

pub trait AudioLibrary: Sized + Hash + Copy + Sync + Send + Eq + PartialEq + 'static {
  fn load_all(asset_server: &AssetServer, guard: AssetBarrierGuard) -> HashMap<Self, AudioDef>;
}

pub trait AudioChannelLayout: Sized + Hash + Eq + PartialEq + Copy + Sync + Send + 'static {
  fn initial_state() -> HashMap<Self, AudioChannelSettings>;
}

pub enum AudioDef {
  Once(Handle<AudioSource>),
  Looped(Handle<AudioSource>),
  IntroLooped {
    intro: Handle<AudioSource>,
    main: Handle<AudioSource>,
  },
}

impl AudioDef {
  pub fn try_into_processed_audio(
    &self,
    asset_server: &AssetServer,
    audio_assets: &Assets<AudioSource>,
  ) -> Option<Handle<ProcessedAudio>> {
    match self {
      AudioDef::Once(s) => {
        if let Some(ss) = audio_assets.get(s) {
          let processed = asset_server.add(ProcessedAudio {
            sources: vec![ss.clone()],
            process: |sources| Box::new(sources.first().unwrap().decoder()),
          });

          return Some(processed);
        }
      }
      AudioDef::Looped(s) => {
        if let Some(ss) = audio_assets.get(s) {
          let processed = asset_server.add(ProcessedAudio {
            sources: vec![ss.clone()],
            process: |sources| Box::new(sources.first().unwrap().decoder().repeat_infinite()),
          });

          return Some(processed);
        }
      }
      AudioDef::IntroLooped { intro, main } => {
        if let Some(is) = audio_assets.get(intro)
          && let Some(ms) = audio_assets.get(main)
        {
          let processed = asset_server.add(ProcessedAudio {
            sources: vec![is.clone(), ms.clone()], // wow
            process: |sources| {
              let copy: Vec<_> = sources
                .iter()
                .enumerate()
                .map(|(i, f)| -> Box<dyn Source<Item = i16> + Send + Sync> {
                  if i == 1 {
                    Box::new(f.decoder().repeat_infinite())
                  } else {
                    Box::new(f.decoder())
                  }
                })
                .collect();
              Box::new(from_iter(copy))
            },
          });
          return Some(processed);
        }
      }
    };

    None
  }
}

#[derive(Component)]
pub struct AudioController<C: AudioChannelLayout> {
  channel: C,
  volume: f32,
  current_volume_cmd: Option<EasingGoal>,
  despawn_on_stop: bool,
  despawn_on_zero_volume: bool,
}

#[derive(Asset, TypePath, Clone)]
pub struct ProcessedAudio {
  pub sources: Vec<AudioSource>,
  pub process: fn(sources: &Vec<AudioSource>) -> Box<dyn Source<Item = i16> + Sync + Send>,
}

impl Decodable for ProcessedAudio {
  type DecoderItem = i16;
  type Decoder = Box<dyn Source<Item = i16> + Sync + Send>;
  fn decoder(&self) -> Self::Decoder {
    (self.process)(&self.sources)
  }
}

#[derive(Resource)]
pub struct AudioLibraryResource<T, C> {
  definitions: HashMap<T, AudioDef>,
  processed: Option<HashMap<T, Handle<ProcessedAudio>>>,
  channels: HashMap<C, AudioChannelSettings>,
  barrier: AssetBarrier,
}

#[derive(Default)]
pub struct AudioChannelSettings {
  pub volume: f32,
  pub current_vol_cmd: Option<EasingGoal>,
}

pub enum EasingGoal {
  Instant(f32),
  // Linear(f32, Timer)
}

impl<T, C> FromWorld for AudioLibraryResource<T, C>
where
  T: AudioLibrary,
  C: AudioChannelLayout,
{
  fn from_world(world: &mut World) -> Self {
    let (barrier, guard) = AssetBarrier::new();
    let asset_server = world
      .get_resource::<AssetServer>()
      .expect("Unable to get AssetServer");

    let defs = <T as AudioLibrary>::load_all(asset_server, guard);
    Self {
      definitions: defs,
      processed: default(),
      channels: C::initial_state(),
      barrier,
    }
  }
}

#[derive(Message, Debug)]
pub enum AudioCommand<L, C> {
  StopAllInChannel(C),
  ReplaceAllAndFadeInto(L, C),
  InsertOnce(L, C),
  // SetChannelVolume(C, f32, u32),
  // MuteChannel(C, bool),
}

fn should_process_assets<L, C>(audio_lib: Res<AudioLibraryResource<L, C>>) -> bool
where
  L: AudioLibrary,
  C: AudioChannelLayout,
{
  audio_lib.barrier.is_ready() && audio_lib.processed.is_none()
}

fn process_assets<L, C>(
  mut audio_lib: ResMut<AudioLibraryResource<L, C>>,
  asset_server: Res<AssetServer>,
  audio_assets: Res<Assets<AudioSource>>,
) where
  L: AudioLibrary,
  C: AudioChannelLayout,
{
  if !audio_lib.barrier.is_ready() || audio_lib.processed.is_some() {
    return;
  }
  let mut h = HashMap::new();
  for (k, v) in audio_lib.definitions.iter() {
    if let Some(p) = v.try_into_processed_audio(&asset_server, &audio_assets) {
      h.insert(*k, p);
    }
  }
  audio_lib.processed = Some(h);
}

fn assets_loaded<L, C>(audio_lib: Res<AudioLibraryResource<L, C>>) -> bool
where
  L: AudioLibrary,
  C: AudioChannelLayout,
{
  audio_lib.barrier.is_ready() && audio_lib.processed.is_some()
}

pub fn process_audio_commands<L, C>(
  mut commands: Commands,
  mut cmds: MessageReader<AudioCommand<L, C>>,
  mut qry: Query<&mut AudioController<C>>,
  audio_lib: Res<AudioLibraryResource<L, C>>,
) where
  L: AudioLibrary,
  C: AudioChannelLayout,
{
  let Some(lib) = &audio_lib.processed else {
    return;
  };

  for cmd in cmds.read() {
    match cmd {
      AudioCommand::StopAllInChannel(channel) => {
        for mut ctl in qry.iter_mut() {
          if &ctl.channel != channel {
            continue;
          }
          ctl.current_volume_cmd = Some(EasingGoal::Instant(0.));
        }
      }
      AudioCommand::ReplaceAllAndFadeInto(to_play, channel) => {
        // TODO: this can be further optimized if slow
        for mut ctl in qry.iter_mut() {
          if &ctl.channel != channel {
            continue;
          }
          ctl.current_volume_cmd = Some(EasingGoal::Instant(0.));
        }
        let Some(handle) = lib.get(to_play) else {
          continue;
        };
        // TODO: set initial volume to 0 and set an easing goal to 1.0
        commands
          .spawn(AudioPlayer(handle.clone()))
          .insert(AudioController {
            channel: *channel,
            volume: 1.0,
            current_volume_cmd: None,
            despawn_on_stop: true,
            despawn_on_zero_volume: true,
          });
      }
      AudioCommand::InsertOnce(to_play, channel) => {
        let Some(handle) = lib.get(to_play) else {
          continue;
        };
        commands
          .spawn(AudioPlayer(handle.clone()))
          .insert(AudioController {
            channel: *channel,
            volume: 1.0,
            current_volume_cmd: None,
            despawn_on_stop: true,
            despawn_on_zero_volume: true,
          });
      }
    };
  }
}

pub fn update_channel_volume<L, C>(mut audio_lib: ResMut<AudioLibraryResource<L, C>>)
where
  L: AudioLibrary,
  C: AudioChannelLayout,
{
  for (_k, ctl) in audio_lib.channels.iter_mut() {
    let Some(cmd) = &ctl.current_vol_cmd else {
      continue;
    };
    match cmd {
      EasingGoal::Instant(new_value) => {
        ctl.volume = *new_value;
        ctl.current_vol_cmd = None;
      }
    };
  }
}

pub fn update_entity_volumes<L, C>(mut qry: Query<&mut AudioController<C>>)
where
  L: AudioLibrary,
  C: AudioChannelLayout,
{
  for mut ctl in qry.iter_mut() {
    let Some(cmd) = &ctl.current_volume_cmd else {
      continue;
    };
    match cmd {
      EasingGoal::Instant(new_value) => {
        ctl.volume = *new_value;
        ctl.current_volume_cmd = None;
      }
    }
  }
}

pub fn update_sink_volumes<L, C>(
  mut cmd: Commands,
  audio_lib: Res<AudioLibraryResource<L, C>>,
  mut qry: Query<(Entity, &mut AudioSink, &AudioController<C>)>,
) where
  L: AudioLibrary,
  C: AudioChannelLayout,
{
  for (e, mut sink, ctl) in qry.iter_mut() {
    let Some(channel_settings) = audio_lib.channels.get(&ctl.channel) else {
      continue;
    };

    sink.set_volume(bevy::audio::Volume::Linear(
      channel_settings.volume * ctl.volume,
    ));

    if (sink.is_paused() && ctl.despawn_on_stop)
      || (sink.volume().to_linear() <= 0.01 && ctl.despawn_on_zero_volume)
    {
      cmd.entity(e).despawn();
    }
  }
}
