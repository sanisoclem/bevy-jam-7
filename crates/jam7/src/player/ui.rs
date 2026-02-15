use bevy::prelude::*;
use sys_combat::{Combatant, CombatantGuages, KillCounter};
use sys_prog::{LongTermProgger, Progger};

use crate::player::Player;

#[derive(Component)]
pub struct StatsUI;

#[derive(Component)]
pub struct HealthBarFill;

#[derive(Component)]
pub struct KillCountText;

#[derive(Component)]
pub struct BossKillCountText;

#[derive(Component)]
pub struct LucidityText;

#[derive(Component)]
pub struct RunCountText;

#[derive(Event)]
pub struct SpawnStatsUI;

#[derive(Event)]
pub struct DespawnStatsUI;

pub fn on_despawn_stats_ui(
  _trigger: On<DespawnStatsUI>,
  mut commands: Commands,
  ui: Query<Entity, With<StatsUI>>,
) {
  for entity in &ui {
    commands.entity(entity).despawn();
  }
}

pub fn on_spawn_stats_ui(
  _trigger: On<SpawnStatsUI>,
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  existing: Query<Entity, With<StatsUI>>,
) {
  if !existing.is_empty() {
    return;
  }

  let font = asset_server.load("fonts/FiraSans-Bold.ttf");

  commands
    .spawn((
      StatsUI,
      Node {
        width: Val::Percent(100.0),
        height: Val::Auto,
        padding: UiRect::all(Val::Px(16.0)),
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Start,
        ..default()
      },
    ))
    .with_children(|root| {
      root
        .spawn(Node {
          flex_direction: FlexDirection::Column,
          row_gap: Val::Px(4.0),
          ..default()
        })
        .with_children(|left| {
          // "HP" label
          left.spawn((
            Text::new("HP"),
            TextFont {
              font: font.clone(),
              font_size: 18.0,
              ..default()
            },
            TextColor(Color::WHITE),
          ));

          left
            .spawn((
              Node {
                width: Val::Px(200.0),
                height: Val::Px(24.0),
                border: UiRect::all(Val::Px(3.0)),
                ..default()
              },
              BackgroundColor(Color::BLACK),
              BorderColor::all(Color::WHITE),
            ))
            .with_children(|bar_container| {
              bar_container.spawn((
                HealthBarFill,
                Node {
                  width: Val::Percent(100.0),
                  height: Val::Percent(100.0),
                  ..default()
                },
                BackgroundColor(Color::srgb(0.9, 0.1, 0.1)),
              ));
            });
        });

      root
        .spawn(Node {
          flex_direction: FlexDirection::Column,
          row_gap: Val::Px(8.0),
          align_items: AlignItems::End,
          ..default()
        })
        .with_children(|right| {
          right.spawn((
            LucidityText,
            Text::new("Lucidty 0 / 0"),
            TextFont {
              font: font.clone(),
              font_size: 20.0,
              ..default()
            },
            TextColor(Color::srgb(0.5, 0.8, 1.0)),
          ));
          right.spawn((
            RunCountText,
            Text::new("Runs × 0"),
            TextFont {
              font: font.clone(),
              font_size: 18.0,
              ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
          ));
          right.spawn((
            KillCountText,
            Text::new("Kills × 0"),
            TextFont {
              font: font.clone(),
              font_size: 18.0,
              ..default()
            },
            TextColor(Color::WHITE),
          ));
          right.spawn((
            BossKillCountText,
            Text::new(""),
            TextFont {
              font: font.clone(),
              font_size: 18.0,
              ..default()
            },
            TextColor(Color::srgb(1.0, 0.85, 0.0)),
          ));
        });
    });
}

pub fn update_stats_ui(
  qry: Query<(&CombatantGuages, &Combatant, &KillCounter, &Progger), With<Player>>,
  lprog: Res<LongTermProgger>,
  mut health_fill: Query<&mut Node, With<HealthBarFill>>,
  mut kill_text: Query<
    &mut Text,
    (
      With<KillCountText>,
      Without<BossKillCountText>,
      Without<LucidityText>,
      Without<RunCountText>,
    ),
  >,
  mut boss_text: Query<
    &mut Text,
    (
      With<BossKillCountText>,
      Without<KillCountText>,
      Without<LucidityText>,
      Without<RunCountText>,
    ),
  >,
  mut lucidity_text: Query<
    &mut Text,
    (
      With<LucidityText>,
      Without<KillCountText>,
      Without<BossKillCountText>,
      Without<RunCountText>,
    ),
  >,
  mut run_text: Query<
    &mut Text,
    (
      With<RunCountText>,
      Without<KillCountText>,
      Without<BossKillCountText>,
      Without<LucidityText>,
    ),
  >,
) {
  let Some((guages, combatant, kc, progger)) = qry.iter().next() else {
    return;
  };

  if let Some(mut fill) = health_fill.iter_mut().next() {
    let hp_percent = (guages.current_hp as f32 / combatant.max_hp as f32) * 100.0;
    fill.width = Val::Percent(hp_percent);
  }

  if let Some(mut text) = kill_text.iter_mut().next() {
    **text = format!("Kills × {}", kc.kills);
  }

  if let Some(mut text) = boss_text.iter_mut().next()
    && progger.bosses_spawned > 0
  {
    **text = format!("Boss Kills × {}", progger.bosses_killed);
  }

  if let Some(mut text) = lucidity_text.iter_mut().next() {
    **text = format!(
      "Lucidty × {} / {}",
      lprog.lucidty.saturating_sub(lprog.used_lucidty),
      lprog.lucidty
    );
  }

  if let Some(mut text) = run_text.iter_mut().next() {
    **text = format!("Runs × {}", lprog.runs);
  }
}
