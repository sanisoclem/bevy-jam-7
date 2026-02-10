use std::marker::PhantomData;

use bevy::{
  color::palettes::{
    css::PURPLE,
    tailwind::{AMBER_500, ORANGE_400, PURPLE_500},
  },
  platform::collections::HashMap,
  prelude::*,
  sprite::Anchor,
  time::Stopwatch,
};
use bevy_enhanced_input::prelude::*;
use sys_animation::{AnimationDefinition, AtlasAnimation, SysAnimationPlugin};
use sys_cam::CameraTarget;
use sys_combat::{Combatant, CombatantKilled, CombatantState, DeathBehavior, HitTestableShape};
use sys_enemy::{Enemy, EnemySpawner, EnemySpawnerState};
use sys_magic::{
  EquippedSpell, EquippedSpellState, SpellBook, SpellBookState, SpellGenerator, SpellTrigger,
};
use sys_move::{IsoMovementStage, IsoWorldCoords, MoveDirection, MoveState, Moveable, Placeable};
use utils::diff::TEAM_PLAYER;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(SysAnimationPlugin::<MoveState>::default())
      .add_input_context::<Player>()
      .add_systems(Update, count_kills)
      .add_observer(apply_movement)
      .add_observer(stop_movement);

    #[cfg(feature = "dev")]
    app.add_systems(Update, draw_gizmos);
  }
}

#[derive(Component, Debug)]
pub struct Player {
  kills: u32,
}

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
      MoveState {
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
      MoveState {
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

  let animations: HashMap<MoveState, AnimationDefinition> = idles.chain(runs).collect();
  let default_animation = animations.get(&MoveState::default()).unwrap().clone();
  (
    Player { kills: 0 },
    CameraTarget,
    Transform::default().with_scale(Vec3::splat(0.1)),
    Visibility::default(),
    SpellBook {
      spells: vec![EquippedSpell {
        generator: SpellGenerator::Fireball {
          radius: 3.,
          base_damage: 100,
          lifetime: 15.5,
          speed: 100.,
          explosion_lifetime: 7.,
          explosion_damage_multiplier: 2.5,
          explosion_radius: 180.,
        },
        cooldown: Timer::from_seconds(30.1, TimerMode::Repeating),
        trigger: SpellTrigger::Auto,
      }],
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
    Anchor(Vec2::new(0., -0.3)),
    AtlasAnimation {
      animations,
      default_animation,
      phantom: PhantomData,
      tint: Some(Color::from(PURPLE_500)),
    },
    Moveable {
      damping: 1.0,
      // mass: 0.01,
      net_forces: Vec2::default(),
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

#[derive(InputAction)]
#[action_output(Vec2)]
pub struct ActionMovePlayer;

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

fn count_kills(
  mut kill_reader: MessageReader<CombatantKilled>,
  mut qry_player: Query<&mut Player>,
) {
  for msg in kill_reader.read() {
    let Some(mut killer) = qry_player.get_mut(msg.killer).ok() else {
      continue;
    };

    killer.kills += 1;
  }
}

fn draw_gizmos(
  mut gizmo: Gizmos,
  qry_player: Query<(&Placeable, &Transform), With<Player>>,
  qry_enemy: Query<(&Placeable, &Transform, &SpellBook), With<Enemy>>,
) {
  for (player_pos, player_transform) in qry_player {
    gizmo.ellipse_2d(
      Isometry2d::from_translation(player_transform.translation.xy()),
      Vec2::new(155. * 0.7, 155. * 0.35),
      Color::from(PURPLE),
    );
    let aspect_ratio = 0.5;
    let color = Color::from(PURPLE);
    let location = player_pos.location;
    let half_extents = Vec2::new(155., 155.);
    let top_right = location + IsoWorldCoords::new(half_extents.x, half_extents.y);
    let top_left = location + IsoWorldCoords::new(-half_extents.x, half_extents.y);
    let bot_left = location + IsoWorldCoords::new(-half_extents.x, -half_extents.y);
    let bot_right = location + IsoWorldCoords::new(half_extents.x, -half_extents.y);

    let tr = top_right.to_screen(aspect_ratio);
    let tl = top_left.to_screen(aspect_ratio);
    let bl = bot_left.to_screen(aspect_ratio);
    let br = bot_right.to_screen(aspect_ratio);

    gizmo.line_2d(tl, tr, color);
    gizmo.line_2d(tr, br, color);
    gizmo.line_2d(br, bl, color);
    gizmo.line_2d(bl, tl, color);
    for (enemy_pos, enemy_transform, sb) in qry_enemy {
      if enemy_pos.location.distance(player_pos.location) <= 155. {
        gizmo.line_2d(
          player_transform.translation.xy(),
          enemy_transform.translation.xy(),
          Color::from(AMBER_500),
        );
      }

      let Some(sp) = sb.spells.first() else {
        continue;
      };
      let SpellGenerator::Fireball {
        lifetime, speed, ..
      } = sp.generator;

      let rad = lifetime * speed;
      gizmo.ellipse_2d(
        Isometry2d::from_translation(enemy_transform.translation.xy()),
        Vec2::new(rad * 0.7, rad * 0.35),
        Color::from(ORANGE_400),
      );
    }
  }
}
