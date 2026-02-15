use bevy::prelude::*;
use sys_audio::GameAudioCommand;
use sys_magic::{SpellBook, SpellGenerator};

use crate::{
  LongTermProgger,
  levelup::{ApplyLevelUp, LevelUp, LevelUpPerk, PendingLevelUp, ShowPendingLevelUpUi},
  spells::SpellUpgrade,
};

#[derive(Component)]
pub struct LevelUpUI;

#[derive(Component)]
pub struct PerkCard(Entity, pub usize);

#[derive(Component)]
pub struct RerollButton(Entity);

pub fn on_levelup_ui(
  evt: On<ShowPendingLevelUpUi>,
  mut cmd: Commands,
  qry: Query<(Entity, &PendingLevelUp, &SpellBook)>,
  ui_root: Query<Entity, With<LevelUpUI>>,
  asset_server: Res<AssetServer>,
  lprog: Res<LongTermProgger>,
) {
  let Some((entity, pending, sb)) = qry.get(evt.0).ok() else {
    return;
  };

  for entity in &ui_root {
    cmd.entity(entity).despawn();
  }

  let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");

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
            let generator = match &choice {
              LevelUpPerk::NewSpell(e, _) => &e.generator,
              LevelUpPerk::SpellUpgradePerk(e) => &sb.spells[e.slot].generator,
            };
            let tex = match generator {
              sys_magic::SpellGenerator::Fireball(_) => "fireball_big",
              sys_magic::SpellGenerator::Chainlightning(_) => "lightning",
              sys_magic::SpellGenerator::Frozenorb(_) => "frozenorb",
            };

            let texture_handle = asset_server.load(format!("ui/{}.png", tex));
            let texts = perk_display_text(choice);
            row
              .spawn((
                PerkCard(entity, i),
                Button,
                Node {
                  width: Val::Px(220.0),
                  min_height: Val::Px(160.0),
                  border: UiRect::all(Val::Px(2.0)),
                  border_radius: BorderRadius::all(Val::Px(8.0)),
                  ..default()
                },
                BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
                BorderColor::all(Color::srgb(0.3, 0.3, 0.3)),
              ))
              .with_children(|card| {
                card.spawn((
                  ImageNode::from_atlas_image(texture_handle.clone(), TextureAtlas::default()),
                  Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    bottom: Val::Px(0.0),
                    ..default()
                  },
                  Pickable::IGNORE,
                ));
                card
                  .spawn((Node {
                    width: Val::Px(220.0),
                    min_height: Val::Px(160.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    padding: UiRect::all(Val::Px(18.)),
                    row_gap: Val::Px(8.0),
                    ..default()
                  },))
                  .with_children(|desc| {
                    desc.spawn((
                      Text::new(texts.title),
                      TextFont {
                        font: font.clone(),
                        font_size: 20.0,
                        ..default()
                      },
                    ));
                    for (t, downgrade) in texts.line_items {
                      desc.spawn((
                        Text::new(t),
                        TextFont {
                          font: font.clone(),
                          font_size: 13.0,
                          ..default()
                        },
                        (if downgrade {
                          TextColor(Color::srgb(0.9, 0.1, 0.1))
                        } else {
                          TextColor(Color::srgb(0.9, 0.9, 0.9))
                        }),
                      ));
                    }
                  });
              });
          }
        });

      if lprog.used_lucidty + lprog.reroll_cost() <= lprog.lucidty {
        root
          .spawn((
            Button,
            RerollButton(evt.0),
            Node {
              padding: UiRect::axes(Val::Px(32.0), Val::Px(16.0)),
              border: UiRect::all(Val::Px(2.0)),
              border_radius: BorderRadius::all(Val::Px(8.0)),
              ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.5, 0.8)),
            BorderColor::all(Color::srgb(0.4, 0.7, 1.0)),
          ))
          .with_children(|button| {
            button.spawn((
              Text::new(format!(
                "Reroll ({}/{})",
                lprog.reroll_cost(),
                lprog.lucidty - lprog.used_lucidty
              )),
              TextFont {
                font: font.clone(),
                font_size: 24.0,
                ..default()
              },
            ));
          });
      }
    });
}

pub struct PerkItemDisplay {
  pub title: String,
  pub line_items: Vec<(String, bool)>,
}
fn perk_display_text(perk: &LevelUpPerk) -> PerkItemDisplay {
  let title = match perk {
    LevelUpPerk::NewSpell(p, _) => match &p.generator {
      SpellGenerator::Fireball(_) => "Marshmallow".to_owned(),
      SpellGenerator::Chainlightning(_) => "Candy Cane".to_owned(),
      SpellGenerator::Frozenorb(_) => "Cotton Candy".to_owned(),
    },
    LevelUpPerk::SpellUpgradePerk(_x) => "Upgrade".to_owned(),
  };

  let line_items = match perk {
    LevelUpPerk::NewSpell(_, d) => d,
    LevelUpPerk::SpellUpgradePerk(x) => &x.upgrades,
  }
  .iter()
  .map(|(x, v, d)| {
    (
      d.replace("{}", &format!("{:.1}", v)),
      matches!(x, SpellUpgrade::SpellDownsideUpgrade(_)),
    )
  })
  .collect();

  PerkItemDisplay { title, line_items }
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
        let Some((pending_entity, _pending)) = qry.get(card.0).ok() else {
          return;
        };

        cmd.trigger(GameAudioCommand::InsertOnce(
          sys_audio::GameAudioLibrary::ButtonEffect,
          sys_audio::GameAudioChannels::UI,
        ));
        cmd.trigger(ApplyLevelUp {
          target: pending_entity,
          slot: card.1,
        });
        for entity in &ui_root {
          cmd.entity(entity).despawn();
        }
      }
    }
  }
}

pub fn reroll_interactions(
  mut cmd: Commands,
  button_query: Query<
    (
      &Interaction,
      &mut BackgroundColor,
      &mut BorderColor,
      &RerollButton,
    ),
    (Changed<Interaction>, With<Button>),
  >,
  mut lprog: ResMut<LongTermProgger>,
) {
  for (interaction, mut bg, mut border, rr) in button_query {
    match interaction {
      Interaction::Hovered => {
        *bg = BackgroundColor(Color::srgb(0.16, 0.16, 0.22));
        *border = BorderColor::all(Color::srgb(0.5, 0.5, 0.6));
      }
      Interaction::None => {
        *bg = BackgroundColor(Color::srgb(0.12, 0.12, 0.16));
        *border = BorderColor::all(Color::srgb(0.3, 0.3, 0.4));
      }
      Interaction::Pressed => {
        let cost = lprog.reroll_cost();

        if lprog.used_lucidty + cost > lprog.lucidty {
          continue;
        }
        lprog.used_lucidty += cost;
        cmd.trigger(LevelUp { target: rr.0 });
      }
    }
  }
}
