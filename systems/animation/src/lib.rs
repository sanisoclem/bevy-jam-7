use bevy::{color::palettes::tailwind::PURPLE_500, platform::collections::HashMap, prelude::*};
use std::{hash::Hash, marker::PhantomData};

#[derive(Default)]
pub struct SysAnimationPlugin<T: Component + Hash + Eq> {
  phantom: PhantomData<T>,
}

impl<T: Component + Hash + Eq> Plugin for SysAnimationPlugin<T> {
  fn build(&self, app: &mut App) {
    app.add_systems(
      Update,
      (
        create_animation_state::<T>,
        update_animation_state::<T>,
        update_sprite,
      ),
    );
  }
}

#[derive(Component, Debug, Clone)]
pub struct AtlasAnimation<T: Component> {
  pub phantom: PhantomData<T>,
  pub animations: HashMap<T, AnimationDefinition>,
  pub default_animation: AnimationDefinition,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
pub fn create_animation_state<T: Component + Hash + Eq>(
  mut cmd: Commands,
  qry: Query<(Entity, &T, &AtlasAnimation<T>), Without<AnimationState>>,
) {
  for (entity, t, anim) in qry.iter() {
    let Ok(mut cmd_entity) = cmd.get_entity(entity) else {
      continue;
    };
    let to_play = anim.animations.get(t).unwrap_or(&anim.default_animation);
    cmd_entity.insert((
      AnimationState {
        current_animation: to_play.clone(),
        current_frame_index: 0,
        timer: to_play.create_timer(),
        done: false,
      },
      Sprite {
        image: to_play.spritesheet.clone(),
        texture_atlas: Some(TextureAtlas {
          layout: to_play.layout.clone(),
          index: *to_play
            .frames
            .first()
            .expect("Animations must have at least one frame"),
        }),
        ..default()
      },
    ));
  }
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
    sprite.color = Color::from(PURPLE_500);
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
