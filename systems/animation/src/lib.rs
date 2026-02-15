use bevy::{
  ecs::{lifecycle::HookContext, world::DeferredWorld},
  platform::collections::HashMap,
  prelude::*,
};
use serde::{Deserialize, Serialize};
use std::{hash::Hash, marker::PhantomData};

pub struct SysAnimationPlugin<T: Component + Hash + Eq> {
  phantom: PhantomData<T>,
}

impl<T: Component + Hash + Eq> Plugin for SysAnimationPlugin<T> {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Update, (update_animation_state::<T>))
      .add_systems(FixedUpdate, (update_sprite,))
      .world_mut()
      .register_component_hooks::<T>()
      .on_insert(on_insert_atlas_animation::<T>);
  }
}

impl<T> Default for SysAnimationPlugin<T>
where
  T: Component + Hash + Eq,
{
  fn default() -> Self {
    Self {
      phantom: PhantomData,
    }
  }
}

#[derive(Component, Debug, Clone)]
pub struct AtlasAnimation<T: Component> {
  pub phantom: PhantomData<T>,
  pub animations: HashMap<T, AnimationDefinition>,
  pub default_animation: AnimationDefinition,
  pub tint: Option<Color>,
}
#[derive(Component, Debug, Clone)]
pub struct AnimationState {
  pub current_animation: AnimationDefinition,
  pub current_frame_index: usize,
  pub done: bool,
  pub timer: Timer,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnimationDefinition {
  pub spritesheet: Handle<Image>,
  pub layout: Handle<TextureAtlasLayout>,
  pub frames: Vec<usize>,
  pub playback_speed: AnimationPlaybackSpeed,
  pub playback_loop: bool,
  pub flip_vertical: bool,
}
impl AnimationDefinition {
  pub fn create_timer(&self) -> Timer {
    self.playback_speed.create_timer(self.frames.len())
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnimationPlaybackSpeed {
  Fps(u32),
  DurationMs(u32),
}
impl AnimationPlaybackSpeed {
  pub fn get_seconds_per_frame(&self, num_frames: usize) -> f32 {
    match self {
      AnimationPlaybackSpeed::Fps(fps) => 1. / *fps as f32,
      AnimationPlaybackSpeed::DurationMs(ms) => (*ms as f32) / ((num_frames as f32) * 1000.),
    }
  }
  pub fn create_timer(&self, num_frames: usize) -> Timer {
    Timer::from_seconds(self.get_seconds_per_frame(num_frames), TimerMode::Repeating)
  }
}

fn on_insert_atlas_animation<T: Component + Hash + Eq>(
  mut world: DeferredWorld,
  HookContext { entity, .. }: HookContext,
) {
  let state_component = world.get::<T>(entity);
  let Some(anim) = world.get::<AtlasAnimation<T>>(entity) else {
    return;
  };

  let to_play = state_component
    .and_then(|t| anim.animations.get(t))
    .unwrap_or(&anim.default_animation);

  let tint = anim.tint;
  let animation = to_play.clone();
  let first_frame = *animation
    .frames
    .first()
    .expect("Animations must have at least one frame");
  let spritesheet = animation.spritesheet.clone();
  let layout = animation.layout.clone();

  world.commands().entity(entity).insert((
    AnimationState {
      current_animation: animation.clone(),
      current_frame_index: 0,
      timer: animation.create_timer(),
      done: false,
    },
    Sprite {
      image: spritesheet,
      texture_atlas: Some(TextureAtlas {
        layout,
        index: first_frame,
      }),
      color: tint.unwrap_or_default(),
      ..default()
    },
  ));
}

pub fn update_animation_state<T: Component + Hash + Eq>(
  mut qry: Query<(&T, &AtlasAnimation<T>, &mut AnimationState), Changed<T>>,
  time: Res<Time>,
) {
  for (t, anim, mut state) in qry.iter_mut() {
    let to_play = anim.animations.get(t).unwrap_or(&anim.default_animation);

    if &state.current_animation == to_play {
      let num_frames = state.current_animation.frames.len();
      state.timer.tick(time.delta());
      if state.timer.just_finished() {
        state.current_frame_index += 1;
        if state.current_frame_index >= num_frames {
          state.current_frame_index %= num_frames;
          if !state.current_animation.playback_loop {
            state.done = true;
          }
        }
      }

      continue;
    }

    state.current_animation = to_play.clone();
    state.done = false;
    state.current_frame_index = 0;
    state.timer = to_play.create_timer();
  }
}
pub fn update_sprite(mut qry: Query<(&AnimationState, &mut Sprite), Changed<AnimationState>>) {
  for (anim, mut sprite) in qry.iter_mut() {
    if anim.done {
      continue;
    }

    sprite.flip_x = anim.current_animation.flip_vertical;
    if sprite.image != anim.current_animation.spritesheet {
      sprite.image = anim.current_animation.spritesheet.clone();
      if let Some(atlas) = sprite.texture_atlas.as_mut() {
        atlas.layout = anim.current_animation.layout.clone();
      } else {
        sprite.texture_atlas = Some(TextureAtlas {
          layout: anim.current_animation.layout.clone(),
          index: 0,
        });
      }
    }

    let Some(atlas) = sprite.texture_atlas.as_mut() else {
      continue;
    };

    atlas.index = anim
      .current_animation
      .frames
      .get(anim.current_frame_index)
      .copied()
      .unwrap_or(0);
  }
}
