use bevy::{color::palettes::css::CRIMSON, prelude::*};
use std::mem::discriminant;

use crate::{LongTermProgDescriptor, LongTermProgger};

#[derive(Component)]
pub struct DeathUI;

#[derive(Component)]
pub struct RestartGame;

#[derive(Component)]
pub struct LucidityCheckbox {
  pub is_checked: bool,
  pub feature: LongTermProgDescriptor,
}

#[derive(Event)]
pub struct ShowDeathUi {
  pub accumulated_lucidty: u32,
}

#[derive(Event)]
pub struct RequestGameRestart;

pub fn spawn_death_ui(
  evt: On<ShowDeathUi>,
  mut cmd: Commands,
  asset_server: Res<AssetServer>,
  mut lprog: ResMut<LongTermProgger>,
) {
  info!(
    "accumulating lucidty {} + {}",
    lprog.lucidty, evt.accumulated_lucidty
  );
  lprog.lucidty += evt.accumulated_lucidty;
  let lucidity = lprog.lucidty;
  let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");
  let texture_handle = asset_server.load("ui/death.png");

  cmd
    .spawn((
      DeathUI,
      Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        flex_direction: FlexDirection::Column,
        ..default()
      },
      BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
      ZIndex(100),
    ))
    .with_children(|root| {
      root
        .spawn(Node {
          flex_direction: FlexDirection::Column,
          align_items: AlignItems::Center,
          row_gap: Val::Px(12.0),
          margin: UiRect::bottom(Val::Px(40.0)),
          ..default()
        })
        .with_children(|top| {
          top.spawn((
            ImageNode::from_atlas_image(texture_handle, TextureAtlas::default()),
            Node {
              width: px(240),
              height: px(240),
              ..default()
            },
          ));

          top.spawn((
            Text::new("YOU HAVE DIED"),
            TextFont {
              font: font.clone(),
              font_size: 48.0,
              ..default()
            },
            TextColor(Color::srgb(0.9, 0.3, 0.3)),
          ));
        });

      root
        .spawn((
          Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(16.0),
            padding: UiRect::all(Val::Px(24.0)),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            margin: UiRect::bottom(Val::Px(40.0)),
            ..default()
          },
          BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
          BorderColor::all(Color::srgb(0.4, 0.4, 0.5)),
        ))
        .with_children(|mid| {
          mid.spawn((
            Text::new(format!("Lucidity: {}", lucidity)),
            TextFont {
              font: font.clone(),
              font_size: 36.0,
              ..default()
            },
            TextColor(Color::srgb(0.7, 0.9, 1.0)),
          ));

          mid.spawn((
            Text::new("Spend lucidity to unlock permanent upgrades"),
            TextFont {
              font: font.clone(),
              font_size: 14.0,
              ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.7)),
          ));
          mid
            .spawn((
              Node {
                flex_direction: FlexDirection::Column,
                flex_wrap: FlexWrap::Wrap,
                align_items: AlignItems::Start,
                max_height: Val::Px(200.0),
                row_gap: Val::Px(16.0),
                column_gap: Val::Px(16.0),
                ..default()
              },
              BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
              BorderColor::all(Color::srgb(0.4, 0.4, 0.5)),
            ))
            .with_children(|feats| {
              for feat in lprog
                .lprog_config
                .as_ref()
                .expect("Need to have lprog config")
                .features
                .iter()
              {
                let is_checked = lprog
                  .active_lprog_features
                  .iter()
                  .any(|x| discriminant(&x.feature) == discriminant(&feat.feature));
                feats
                  .spawn((
                    LucidityCheckbox {
                      feature: feat.clone(),
                      is_checked,
                    },
                    Button,
                    Node {
                      flex_direction: FlexDirection::Row,
                      align_items: AlignItems::Center,
                      column_gap: Val::Px(12.0),
                      padding: UiRect::all(Val::Px(12.0)),
                      border: UiRect::all(Val::Px(1.0)),
                      border_radius: BorderRadius::all(Val::Px(4.0)),
                      ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
                    BorderColor::all(Color::srgb(0.3, 0.3, 0.4)),
                  ))
                  .with_children(|checkbox_row| {
                    // checkbox indicator
                    checkbox_row.spawn((
                      Node {
                        width: Val::Px(20.0),
                        height: Val::Px(20.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                      },
                      (if is_checked {
                        BackgroundColor(Color::srgb(0.2, 0.7, 0.3))
                      } else {
                        BackgroundColor(Color::srgb(0.05, 0.05, 0.08))
                      }),
                      BorderColor::all(Color::srgb(0.5, 0.5, 0.6)),
                    ));
                    checkbox_row.spawn((
                      Text::new(format!("{} ({})", feat.description, feat.cost)),
                      TextFont {
                        font: font.clone(),
                        font_size: 16.0,
                        ..default()
                      },
                    ));
                  });
              }
            });
        });

      root
        .spawn((
          Button,
          RestartGame,
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
            Text::new("Start Again"),
            TextFont {
              font: font.clone(),
              font_size: 24.0,
              ..default()
            },
          ));
        });
    });
}

pub fn death_ui_interaction(
  mut cmd: Commands,
  mut checkbox_query: Query<
    (
      &Interaction,
      &mut LucidityCheckbox,
      &mut BackgroundColor,
      &mut BorderColor,
      &Children,
    ),
    (Changed<Interaction>, With<LucidityCheckbox>),
  >,
  mut checkbox_indicator_query: Query<
    &mut BackgroundColor,
    (Without<LucidityCheckbox>, Without<Button>),
  >,
  mut button_query: Query<
    (&Interaction, &mut BackgroundColor, &mut BorderColor),
    (
      Changed<Interaction>,
      With<RestartGame>,
      With<Button>,
      Without<LucidityCheckbox>,
    ),
  >,
  ui_root: Query<Entity, With<DeathUI>>,
  mut lprog: ResMut<LongTermProgger>,
) {
  for (interaction, mut checkbox, mut bg, mut border, children) in &mut checkbox_query {
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
        if !checkbox.is_checked && lprog.used_lucidty + checkbox.feature.cost > lprog.lucidty {
          continue;
        }

        checkbox.is_checked = !checkbox.is_checked;
        if let Some(indicator_entity) = children.first()
          && let Ok(mut indicator_bg) = checkbox_indicator_query.get_mut(*indicator_entity)
        {
          *indicator_bg = match checkbox.is_checked {
            true => BackgroundColor(Color::srgb(0.2, 0.7, 0.3)),
            false => BackgroundColor(Color::srgb(0.05, 0.05, 0.08)),
          };
        }

        if checkbox.is_checked {
          lprog.used_lucidty += checkbox.feature.cost;
          lprog.active_lprog_features.push(checkbox.feature.clone());
        } else {
          lprog.used_lucidty -= checkbox.feature.cost;
          lprog
            .active_lprog_features
            .retain(|x| discriminant(&x.feature) != discriminant(&checkbox.feature.feature));
        }
      }
    }
  }

  for (interaction, mut bg, mut border) in &mut button_query {
    match interaction {
      Interaction::Hovered => {
        *bg = BackgroundColor(Color::srgb(0.3, 0.6, 0.9));
        *border = BorderColor::all(Color::srgb(0.5, 0.8, 1.0));
      }
      Interaction::None => {
        *bg = BackgroundColor(Color::srgb(0.2, 0.5, 0.8));
        *border = BorderColor::all(Color::srgb(0.4, 0.7, 1.0));
      }
      Interaction::Pressed => {
        cmd.trigger(RequestGameRestart);
        for entity in &ui_root {
          cmd.entity(entity).despawn();
        }
      }
    }
  }
}
