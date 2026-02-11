use bevy::prelude::*;
use sys_magic::SpellGenerator;

use crate::levelup::{ApplyLevelUp, LevelUpPerk, PendingLevelUp};

#[derive(Component)]
pub struct LevelUpUI;

#[derive(Component)]
pub struct PerkCard(pub usize);

pub fn spawn_levelup_ui(
  mut commands: Commands,
  pending: Res<PendingLevelUp>,
  asset_server: Res<AssetServer>,
) {
  if pending.target.is_none() {
    return;
  }
  let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");

  commands
    .spawn((
      LevelUpUI,
      Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(16.0),
        ..default()
      },
      BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
      ZIndex(100),
    ))
    .with_children(|root| {
      root.spawn((
        Text::new("LEVEL UP"),
        TextFont {
          font: font.clone(),
          font_size: 48.0,
          ..default()
        },
      ));
      root.spawn((
        Text::new("Choose your path"),
        TextFont {
          font: font.clone(),
          font_size: 18.0,
          ..default()
        },
      ));

      root
        .spawn(Node {
          flex_direction: FlexDirection::Row,
          column_gap: Val::Px(20.0),
          ..default()
        })
        .with_children(|row| {
          for (i, choice) in pending.choices.iter().enumerate() {
            let (title, description) = perk_display_text(choice);
            row
              .spawn((
                PerkCard(i),
                Button,
                Node {
                  width: Val::Px(220.0),
                  min_height: Val::Px(160.0),
                  flex_direction: FlexDirection::Column,
                  align_items: AlignItems::FlexStart,
                  padding: UiRect::all(Val::Px(18.0)),
                  row_gap: Val::Px(8.0),
                  border: UiRect::all(Val::Px(2.0)),
                  border_radius: BorderRadius::all(Val::Px(8.0)),
                  ..default()
                },
                BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
                BorderColor::all(Color::srgb(0.3, 0.3, 0.3)),
              ))
              .with_children(|card| {
                card.spawn((
                  Text::new(title),
                  TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                  },
                ));
                card.spawn((
                  Text::new(description),
                  TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                  },
                ));
              });
          }
        });
    });
}

fn perk_display_text(perk: &LevelUpPerk) -> (&'static str, String) {
  match perk {
    LevelUpPerk::NewSpell(p) => match &p.generator {
      SpellGenerator::Fireball(g) => (
        "🔥 New Spell: Fireball",
        format!(
          "DMG {}  SPD {:.0}  SIZE {:.1}  LIFE {:.1}s",
          g.base_damage, g.speed, g.radius, g.lifetime
        ),
      ),
      SpellGenerator::Chainlightning(g) => (
        "New Spell: Lightning",
        format!("DMG {}  SPD {:.0}", g.base_damage, g.speed,),
      ),
      SpellGenerator::Frozenorb(g) => (
        " New Spell: FrozenOrb",
        format!("DMG {}  SPD {:.0} ", g.base_damage, g.speed,),
      ),
    },
    LevelUpPerk::SpellUpgradePerk(_) => ("⬆ Spell Upgrade", "Enhance an equipped spell".into()),
  }
}

pub fn levelup_ui_interaction(
  mut commands: Commands,
  mut interaction_query: Query<
    (
      &Interaction,
      &PerkCard,
      &mut BackgroundColor,
      &mut BorderColor,
    ),
    (Changed<Interaction>, With<Button>),
  >,
  mut pending: ResMut<PendingLevelUp>,
  mut time: ResMut<Time<Virtual>>,
  ui_root: Query<Entity, With<LevelUpUI>>,
) {
  for (interaction, card, mut bg, mut border) in &mut interaction_query {
    match interaction {
      Interaction::Hovered => {
        *bg = BackgroundColor(Color::srgb(0.14, 0.14, 0.2));
        *border = BorderColor::all(Color::WHITE);
      }
      Interaction::None => {
        *bg = BackgroundColor(Color::srgb(0.08, 0.08, 0.12));
        *border = BorderColor::all(Color::srgb(0.3, 0.3, 0.3));
      }
      Interaction::Pressed => {
        let Some(target) = pending.target else {
          continue;
        };
        let perk = pending.choices.remove(card.0);

        commands.trigger(ApplyLevelUp { target, perk });
        pending.target = None;
        pending.choices.clear();
        time.unpause();
        for entity in &ui_root {
          commands.entity(entity).despawn();
        }
      }
    }
  }
}

pub fn despawn_levelup_ui(
  mut commands: Commands,
  ui_root: Query<Entity, With<LevelUpUI>>,
  pending: Res<PendingLevelUp>,
) {
  if pending.is_changed() && pending.target.is_none() {
    for entity in &ui_root {
      commands.entity(entity).despawn();
    }
  }
}
