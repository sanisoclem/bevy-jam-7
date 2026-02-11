use bevy::{
  color::palettes::tailwind::PURPLE_500, platform::collections::HashMap, prelude::*,
  sprite::Anchor, time::Stopwatch,
};
use bevy_enhanced_input::prelude::*;
use std::marker::PhantomData;
use sys_animation::{AnimationDefinition, AtlasAnimation, SysAnimationPlugin};
use sys_cam::CameraTarget;
use sys_combat::{Combatant, CombatantState, DeathBehavior, HitTestableShape, KillCounter};
use sys_enemy::{EnemySpawner, EnemySpawnerState};
use sys_magic::{
  EquippedSpell, EquippedSpellState, SpellBook, SpellBookState, SpellDownside, SpellGenerator,
  spells::fireball::FireballSpellGenerator,
};
use sys_move::{IsoMovementStage, IsoWorldCoords, MoveDirection, MoveState, Moveable, Placeable};
use utils::diff::TEAM_PLAYER;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(SysAnimationPlugin::<PlayerAnimationState>::default())
      .add_input_context::<Player>()
      .add_systems(Update, update_animation_state)
      .add_observer(apply_movement)
      .add_observer(stop_movement);
  }
}

#[derive(Component, Debug)]
pub struct Player;

pub fn create_player(
  asset_server: &AssetServer,
  layouts: &mut Assets<TextureAtlasLayout>,
  spawn_parent: Entity,
) -> impl Bundle {
  let idle_image = asset_server.load("char/placeholder/idle.png");
  let run_image = asset_server.load("char/placeholder/run.png");
  let idle_layout = layouts.add(TextureAtlasLayout::from_grid(
    UVec2::splat(460),
    8,
    5,
    None,
    None,
  ));
  let run_layout = layouts.add(TextureAtlasLayout::from_grid(
    UVec2::splat(460),
    4,
    5,
    None,
    None,
  ));

  let dirs = [
    (MoveDirection::South, 0, false),
    (MoveDirection::Southwest, 1, false),
    (MoveDirection::West, 2, false),
    (MoveDirection::Northwest, 3, false),
    (MoveDirection::North, 4, false),
    (MoveDirection::Southeast, 1, true),
    (MoveDirection::East, 2, true),
    (MoveDirection::Northeast, 3, true),
  ];

  let idles = dirs.iter().map(|(d, row, flip)| {
    (
      PlayerAnimationState {
        is_moving: false,
        direction: d.clone(),
      },
      AnimationDefinition {
        layout: idle_layout.clone(),
        spritesheet: idle_image.clone(),
        frames: vec![0, 1, 2, 3, 4, 5, 6, 7]
          .into_iter()
          .map(|x| (row * 8) + x)
          .collect(),
        playback_speed: sys_animation::AnimationPlaybackSpeed::Fps(5),
        playback_loop: true,
        flip_vertical: *flip,
      },
    )
  });

  let runs = dirs.iter().map(|(d, row, flip)| {
    (
      PlayerAnimationState {
        is_moving: true,
        direction: d.clone(),
      },
      AnimationDefinition {
        layout: run_layout.clone(),
        spritesheet: run_image.clone(),
        frames: vec![0, 1, 2, 3]
          .into_iter()
          .map(|x| (row * 4) + x)
          .collect(),
        playback_speed: sys_animation::AnimationPlaybackSpeed::Fps(15),
        playback_loop: true,
        flip_vertical: *flip,
      },
    )
  });

  let animations: HashMap<PlayerAnimationState, AnimationDefinition> = idles.chain(runs).collect();
  let default_animation = animations
    .get(&PlayerAnimationState::default())
    .unwrap()
    .clone();
  (
    Player,
    KillCounter { kills: 150 },
    CameraTarget,
    Transform::default().with_scale(Vec3::splat(0.1)),
    Visibility::default(),
    SpellBook {
      spells: vec![],
      disabled: false,
    },
    SpellBookState {
      spells_states: vec![EquippedSpellState::default()],
    },
    EnemySpawner {
      spawn_parent,
      despawn_radius: 1000,
      no_spawn_radius: 400,
      spawn_radius: 700,
      initial_cooldown: 1.,
      cooldown_decay_rate: 1.5,
    },
    EnemySpawnerState {
      stopwatch: Stopwatch::new(),
      cooldown: Timer::from_seconds(0.5, TimerMode::Once),
    },
    Combatant {
      max_hp: 1000000,
      hitbox: HitTestableShape::Circle { radius: 7.0 },
      team: TEAM_PLAYER,
      regen: 0,
      regen_delay: 0,
      death_behavior: DeathBehavior::Respawn(
        Timer::from_seconds(5.0, TimerMode::Once),
        Timer::from_seconds(2.0, TimerMode::Once),
      ),
    },
    (
      Anchor(Vec2::new(0., -0.3)),
      PlayerAnimationState::default(),
      AtlasAnimation {
        animations,
        default_animation,
        phantom: PhantomData,
        tint: Some(Color::from(PURPLE_500)),
      },
    ),
    Moveable {
      damping: 1.0,
      // mass: 0.01,
      net_forces: Vec2::default(),
      impulses: Vec::new(),
    },
    Placeable {
      layer: 5,
      location: IsoWorldCoords::default(),
    },
    actions!(
      Player[(
        Action::<ActionMovePlayer>::new(),
        DeadZone::default(), // Applies non-uniform normalization.
        bindings![
          // Keyboard keys captured as `bool`, but the output of `Movement` is defined as `Vec2`,
          // so you need to assign keys to axes using swizzle to reorder them and negation.
          (KeyCode::KeyW, SwizzleAxis::YXZ),
          (KeyCode::KeyA, Negate::all()),
          (KeyCode::KeyS, Negate::all(), SwizzleAxis::YXZ),
          KeyCode::KeyD,
          // In Bevy sticks split by axes and captured as 1-dimensional inputs,
          // so Y stick needs to be sweezled into Y axis.
          GamepadAxis::LeftStickX,
          (GamepadAxis::LeftStickY, SwizzleAxis::YXZ),
        ]
      )]
    ),
  )
}

#[derive(Component, Debug, Clone, Default, Eq, Hash, PartialEq)]
pub struct PlayerAnimationState {
  pub is_moving: bool,
  pub direction: MoveDirection,
}

#[derive(InputAction)]
#[action_output(Vec2)]
pub struct ActionMovePlayer;

fn update_animation_state(qry: Query<(&mut PlayerAnimationState, &MoveState), With<Player>>) {
  for (mut anim, mov) in qry {
    anim.is_moving = mov.is_moving_voluntary;
    anim.direction = mov.direction.clone();
  }
}

fn apply_movement(
  movement: On<Fire<ActionMovePlayer>>,
  mut players: Query<(&mut Moveable, &ChildOf, &CombatantState), With<Player>>,
  qry_stage: Query<&IsoMovementStage>,
) {
  let Ok((mut mv, co, cs)) = players.get_mut(movement.context) else {
    return;
  };
  let Ok(stage) = qry_stage.get(co.parent()) else {
    return;
  };

  if cs.dead || cs.stunned {
    mv.net_forces = Vec2::splat(0.);
  } else {
    let world_direction: Vec2 = *IsoWorldCoords::from_screen(movement.value, stage.aspect_ratio);
    mv.net_forces = world_direction.normalize() * 200.;
  }
}

fn stop_movement(
  movement: On<Complete<ActionMovePlayer>>,
  mut players: Query<&mut Moveable, With<Player>>,
) {
  let Ok(mut mv) = players.get_mut(movement.context) else {
    return;
  };
  mv.net_forces = Vec2::splat(0.);
}
