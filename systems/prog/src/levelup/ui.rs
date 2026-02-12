use bevy::prelude::*;
use sys_magic::SpellGenerator;

use crate::levelup::{ApplyLevelUp, LevelUpPerk, PendingLevelUp, ShowPendingLevelUpUi};

#[derive(Component)]
pub struct LevelUpUI;

#[derive(Component)]
pub struct PerkCard(Entity, pub usize);

#[derive(Component)]
pub struct LevelUpUiShown;

pub fn on_levelup_ui(
  evt: On<ShowPendingLevelUpUi>,
  mut cmd: Commands,
  qry: Query<(Entity, &PendingLevelUp), Without<LevelUpUiShown>>,
  asset_server: Res<AssetServer>,
) {
  let Some((entity, pending)) = qry.get(evt.0).ok() else {
    return;
  };

  let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");

  cmd.entity(entity).insert(LevelUpUiShown);
  cmd
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
                PerkCard(entity, i),
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
    LevelUpPerk::SpellUpgradePerk(x) => ("⬆ Spell Upgrade", format!("{}", x)),
  }
}

pub fn levelup_ui_interaction(
  mut cmd: Commands,
  mut interaction_query: Query<
    (
      &Interaction,
      &PerkCard,
      &mut BackgroundColor,
      &mut BorderColor,
    ),
    (Changed<Interaction>, With<Button>),
  >,
  qry: Query<(Entity, &PendingLevelUp)>,
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
        let Some((pending_entity, pending)) = qry.get(card.0).ok() else {
          return;
        };

        cmd.trigger(ApplyLevelUp {
          target: pending_entity,
          slot: card.1,
        });
        cmd.entity(pending_entity).remove::<LevelUpUiShown>();
        for entity in &ui_root {
          cmd.entity(entity).despawn();
        }
      }
    }
  }
}
