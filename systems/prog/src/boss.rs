use bevy::prelude::*;
use sys_combat::{Combatant, CombatantGuages, CombatantKilled, KillCounter};
use sys_enemy::{Enemy, EnemyAnimationState, EnemySpawner};
use sys_magic::{SpellBook, SpellBookState};
use sys_move::{IsoWorldCoords, MoveState, Moveable, Placeable};

use crate::{Progger, boss::ui::ShowBossHealthBar};

pub mod ui;

#[derive(Component)]
pub struct BossEnemy {
  focus: IsoWorldCoords,
  focus_timer: Timer,
}

pub fn spawn_boss(
  query: Query<(
    &KillCounter,
    &mut Progger,
    &mut EnemySpawner,
    &SpellBook,
    &SpellBookState,
    &Placeable,
  )>,
  qry_enemy: Query<(Entity, &Transform, &Combatant), With<Enemy>>,
  mut cmd: Commands,
) {
  for (kc, mut prog, mut spawner, sb, sbs, pos) in query {
    if kc.kills / (100 * (prog.bosses_spawned + 1)) < prog.bosses_spawned + 1 {
      continue;
    }

    let Some((enemy, enemy_pos, c)) = qry_enemy.iter().next() else {
      continue;
    };

    prog.bosses_spawned += 1;
    spawner.disabled = true;

    let spells_to_take = (prog.bosses_spawned / 3).max(1) as usize;
    let sbclone = SpellBook {
      spells: sb.spells.iter().take(spells_to_take).cloned().collect(),
      disabled: false,
    };
    let sbsclone = SpellBookState {
      spells_states: sbs
        .spells_states
        .iter()
        .take(spells_to_take)
        .cloned()
        .collect(),
    };
    let max_hp = (c.max_hp * (10 * prog.bosses_spawned)).max(1000);
    let mut cclone = c.clone();
    cclone.max_hp = max_hp;

    cmd.trigger(ShowBossHealthBar { boss_entity: enemy });
    cmd.entity(enemy).remove::<Enemy>().insert((
      BossEnemy {
        focus: pos.location,
        focus_timer: Timer::from_seconds(3.0, TimerMode::Repeating),
      },
      sbclone,
      sbsclone,
      cclone,
      CombatantGuages {
        current_hp: max_hp,
        invulnerability_timer: Some(Timer::from_seconds(5.0, TimerMode::Once)),
        reeling_timer: None,
        stun_timer: None,
        death_timer: None,
      },
      Transform::default()
        .with_translation(enemy_pos.translation)
        .with_scale(enemy_pos.scale * prog.bosses_spawned as f32),
    ));
  }
}

pub fn wait_for_boss_kills(
  mut reader: MessageReader<CombatantKilled>,
  qry: Query<&BossEnemy>,
  mut qry_prog: Query<(&mut Progger, &mut EnemySpawner)>,
  mut cmd: Commands,
) {
  for msg in reader.read() {
    let Some(_boss) = qry.get(msg.victim).ok() else {
      continue;
    };
    let Some((mut progger, mut spawner)) = qry_prog.get_mut(msg.killer).ok() else {
      // this means if bosses kills themselves, it wont count
      continue;
    };

    progger.bosses_killed += 1;
    spawner.disabled = false;

    cmd.trigger(ShowBossKill {
      count: progger.bosses_killed,
    });
  }
}

pub fn update_boss_objectives(
  qry_enemies: Query<(&mut BossEnemy, &Placeable, &mut Moveable)>,
  player: Query<&Placeable, With<EnemySpawner>>,
  time: Res<Time>,
) {
  let Some(player_pos) = player.iter().next().map(|p| p.location) else {
    return;
  };

  for (mut enemy, placeable, mut mov) in qry_enemies {
    enemy.focus_timer.tick(time.delta());
    enemy.focus = if enemy.focus_timer.just_finished() {
      player_pos
    } else {
      enemy.focus
    };

    let dist_to_player = placeable.location.distance(enemy.focus);

    let direction = (enemy.focus - placeable.location).normalize_or_zero();
    let desired_range = 500.;
    let distance_error = dist_to_player - desired_range;

    mov.net_forces = if distance_error.abs() < 100.0 {
      direction.perp()
    } else if distance_error > 0.0 {
      direction
    } else {
      -direction
    } * 400.;

    continue;
  }
}
pub fn update_animation_state(qry: Query<(&mut EnemyAnimationState, &MoveState), With<BossEnemy>>) {
  for (mut anim, mov) in qry {
    anim.moving = mov.is_moving_voluntary;
    anim.facing = mov.direction.clone();
  }
}

#[derive(Component)]
pub struct BossKillText {
  timer: Timer,
}
#[derive(Component)]
pub struct BossKillUi;

#[derive(Event)]
pub struct ShowBossKill {
  pub count: u32,
}

pub fn spawn_boss_kill_text(
  trigger: On<ShowBossKill>,
  mut cmd: Commands,
  asset_server: Res<AssetServer>,
) {
  let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");

  cmd
    .spawn((
      BossKillUi,
      Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(16.0),
        ..default()
      },
      BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
      ZIndex(100),
    ))
    .with_children(|root| {
      root.spawn((
        BossKillText {
          timer: Timer::from_seconds(5.0, TimerMode::Once),
        },
        Text::new(if trigger.count > 1 {
          format!("{} Boss Kills!", trigger.count)
        } else {
          "Boss Kill!".to_owned()
        }),
        TextFont {
          font,
          font_size: 48.0,
          ..default()
        },
        TextColor(Color::srgb(1.0, 0.8, 0.2)),
        Transform::from_translation(Vec3::new(0.0, 0.0, 100.0)),
        ZIndex(1000),
      ));
    });
}

pub fn update_boss_kill_text(
  mut commands: Commands,
  despawn_qry: Query<Entity, With<BossKillUi>>,
  mut query: Query<(&mut BossKillText, &mut TextColor)>,
  time: Res<Time<Real>>,
) {
  for (mut text, mut color) in &mut query {
    text.timer.tick(time.delta());

    let remaining = text.timer.remaining_secs();

    const FADE_TIME: f32 = 0.5;
    if remaining <= FADE_TIME {
      let alpha = remaining / FADE_TIME;
      color.0 = color.0.with_alpha(alpha);
    }

    if text.timer.just_finished() {
      for entity in despawn_qry {
        commands.entity(entity).despawn();
      }
    }
  }
}
