use bevy::prelude::*;
use sys_combat::{Combatant, CombatantGuages};

use crate::boss::{BossKilled, BossSpawned};

#[derive(Component)]
pub struct BossHealthBarUI;

#[derive(Component)]
pub struct BossHealthFill;

#[derive(Component)]
pub struct BossNameText;

#[derive(Component)]
pub struct BossHealthBarTarget(Entity);

pub fn on_show_boss_health_bar(
  evt: On<BossSpawned>,
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  existing: Query<Entity, With<BossHealthBarUI>>,
) {
  for entity in &existing {
    commands.entity(entity).despawn();
  }

  let boss_entity = evt.boss_entity;
  let font = asset_server.load("fonts/FiraSans-Bold.ttf");

  commands
    .spawn((
      BossHealthBarUI,
      BossHealthBarTarget(boss_entity),
      Node {
        position_type: PositionType::Absolute,
        top: Val::Px(60.0),
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: Val::Px(8.0),
        ..default()
      },
      Transform::default(),
    ))
    .with_children(|root| {
      root.spawn((
        BossNameText,
        Text::new("BOSS"),
        TextFont {
          font: font.clone(),
          font_size: 28.0,
          ..default()
        },
        TextColor(Color::srgb(1.0, 0.2, 0.2)),
      ));
      root
        .spawn((
          Node {
            width: Val::Px(400.0),
            height: Val::Px(32.0),
            border: UiRect::all(Val::Px(4.0)),
            ..default()
          },
          BackgroundColor(Color::BLACK),
          BorderColor::all(Color::srgb(1.0, 0.2, 0.2)),
        ))
        .with_children(|bar_container| {
          bar_container.spawn((
            BossHealthFill,
            Node {
              width: Val::Percent(100.0),
              height: Val::Percent(100.0),
              ..default()
            },
            BackgroundColor(Color::srgb(1.0, 0.3, 0.3)),
          ));
        });
    });
}

pub fn on_hide_boss_health_bar(
  _trigger: On<BossKilled>,
  mut commands: Commands,
  ui: Query<Entity, With<BossHealthBarUI>>,
) {
  for entity in &ui {
    commands.entity(entity).despawn();
  }
}

pub fn update_boss_health_bar(
  boss_bar: Query<&BossHealthBarTarget, With<BossHealthBarUI>>,
  bosses: Query<(&CombatantGuages, &Combatant)>,
  mut health_fill: Query<&mut Node, With<BossHealthFill>>,
) {
  let Some(target) = boss_bar.iter().next() else {
    return;
  };

  let Ok((guages, combatant)) = bosses.get(target.0) else {
    return;
  };

  if let Some(mut fill) = health_fill.iter_mut().next() {
    let hp_percent = (guages.current_hp as f32 / combatant.max_hp as f32) * 100.0;
    fill.width = Val::Percent(hp_percent);
  }
}
