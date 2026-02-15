use bevy::{ecs::relationship::Relationship, prelude::*};
use sys_magic::{SpellBook, SpellBookState, SpellDownside};

#[derive(Component)]
pub struct SpellBarUI {
  player: Entity,
}

#[derive(Component)]
pub struct SpellSlot {
  pub slot_index: usize,
}

#[derive(Component)]
pub struct SpellCooldownOverlay;

#[derive(Component)]
pub struct SpellCooldownText;

#[derive(Event)]
pub struct SpawnSpellBarUI {
  pub player_entity: Entity,
}

#[derive(Event)]
pub struct DespawnSpellBarUI;

#[derive(Component)]
pub struct SpellTooltip;

#[derive(Component)]
pub struct SpellSlotButton {
  pub slot_index: usize,
}

pub fn on_spawn_spell_bar_ui(
  evt: On<SpawnSpellBarUI>,
  mut commands: Commands,
  existing: Query<Entity, With<SpellBarUI>>,
) {
  if !existing.is_empty() {
    return;
  }

  commands.spawn((
    SpellBarUI {
      player: evt.player_entity,
    },
    Node {
      position_type: PositionType::Absolute,
      bottom: Val::Px(0.0),
      left: Val::Percent(0.0),
      right: Val::Percent(0.0),
      flex_direction: FlexDirection::Row,
      justify_content: JustifyContent::Center,
      align_items: AlignItems::Center,
      column_gap: Val::Px(12.0),
      padding: UiRect::all(Val::Px(10.)),
      ..default()
    },
    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
    Transform::default(),
  ));
}

pub fn on_despawn_spell_bar_ui(
  _trigger: On<DespawnSpellBarUI>,
  mut commands: Commands,
  ui: Query<Entity, With<SpellBarUI>>,
) {
  for entity in &ui {
    commands.entity(entity).despawn();
  }
}

pub fn update_spells_changed(
  asset_server: Res<AssetServer>,
  qry: Query<&SpellBook, Changed<SpellBook>>,
  qry_ui: Query<(Entity, &SpellBarUI)>,
  qry_ss: Query<(&SpellSlot, &ChildOf)>,
  mut cmd: Commands,
) {
  let font = asset_server.load("fonts/FiraSans-Bold.ttf");
  for (ui_entity, ui) in qry_ui {
    let Some(sb) = qry.get(ui.player).ok() else {
      continue;
    };
    let max_slot = qry_ss
      .iter()
      .filter(|(_, parent)| parent.get() == ui_entity)
      .map(|(s, _)| s.slot_index)
      .max();

    for (slot_index, spell) in sb.spells.iter().enumerate() {
      if max_slot.is_some_and(|max| slot_index <= max) {
        continue;
      }
      let tex = match spell.generator {
        sys_magic::SpellGenerator::Fireball(_) => "fireball",
        sys_magic::SpellGenerator::Chainlightning(_) => "lightning",
        sys_magic::SpellGenerator::Frozenorb(_) => "frozenorb",
      };
      let texture_handle = asset_server.load(format!("ui/{}.png", tex));
      cmd.entity(ui_entity).with_children(|root| {
        root
          .spawn((
            SpellSlot { slot_index },
            SpellSlotButton { slot_index },
            Button,
            Node {
              width: Val::Px(60.0),
              height: Val::Px(60.0),
              border: UiRect::all(Val::Px(1.0)),
              justify_content: JustifyContent::Center,
              align_items: AlignItems::Center,
              padding: UiRect::all(Val::Px(3.0)),
              ..default()
            },
            BorderColor::all(Color::WHITE),
          ))
          .with_children(|slot| {
            slot.spawn((
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
            slot.spawn((
              SpellCooldownOverlay,
              Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(0.0),
                bottom: Val::Px(0.0),
                ..default()
              },
              BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
              Pickable::IGNORE,
            ));
            slot.spawn((
              SpellCooldownText,
              Text::new(""),
              TextFont {
                font: font.clone(),
                font_size: 20.0,
                ..default()
              },
              TextColor(Color::WHITE),
              ZIndex(1),
              Pickable::IGNORE,
            ));
          });
      });
    }
  }
}

pub fn update_spell_bar_ui(
  spellbook_state: Query<&SpellBookState>,
  spell_slots: Query<(&SpellSlot, &Children)>,
  mut cooldown_overlays: Query<&mut Node, (With<SpellCooldownOverlay>, Without<SpellCooldownText>)>,
  mut cooldown_texts: Query<&mut Text, With<SpellCooldownText>>,
) {
  let Some(state) = spellbook_state.iter().next() else {
    return;
  };

  for (slot, children) in &spell_slots {
    let Some(spell_state) = state.spells_states.get(slot.slot_index) else {
      continue;
    };
    let Some(cooldown) = &spell_state.cooldown else {
      continue;
    };

    let remaining = cooldown.remaining_secs();
    let fraction = cooldown.fraction();

    for child in children.iter() {
      if let Ok(mut overlay) = cooldown_overlays.get_mut(child) {
        overlay.height = Val::Percent(fraction * 100.0);
      }

      if let Ok(mut text) = cooldown_texts.get_mut(child) {
        if remaining > 0.0 {
          **text = format!("{:.1}", remaining);
        } else {
          **text = String::new();
        }
      }
    }
  }
}

pub fn handle_spell_tooltip(
  mut cmd: Commands,
  asset_server: Res<AssetServer>,
  spellbook_query: Query<&SpellBook>,
  ui_query: Query<&SpellBarUI>,
  interaction_query: Query<(&Interaction, &SpellSlotButton), Changed<Interaction>>,
  mut active_tooltip: Local<Option<(Entity, usize)>>,
) {
  let font = asset_server.load("fonts/FiraSans-Bold.ttf");
  let mut active_slot: Option<&SpellSlotButton> = None;
  for (interaction, slot) in interaction_query {
    match *interaction {
      Interaction::Hovered => {
        active_slot = Some(slot);
      }
      Interaction::None => {
        if let Some((e, i)) = active_tooltip.as_ref()
          && *i == slot.slot_index
        {
          cmd.entity(*e).despawn();
          *active_tooltip = None;
        }
      }
      _ => {}
    }
  }

  if let Some(slot) = active_slot
    && active_tooltip.is_none()
  {
    let ui = ui_query.iter().next();
    let spellbook = ui.and_then(|ui| spellbook_query.get(ui.player).ok());

    if let Some(spell) = spellbook.and_then(|sb| sb.spells.get(slot.slot_index)) {
      let (spell_name, spell_desc) = match &spell.generator {
        sys_magic::SpellGenerator::Fireball(g) => (
          "Marshmallow",
          format!(
            "Damage: {:.0}\nSpeed: {:.0}\nSize: {:.0}\nExplosion size: {:.0}\nExplosion multiplier: {:.2}\nExplosion duration: {:.2}",
            g.base_damage,
            g.speed,
            g.radius,
            g.explosion_radius,
            g.explosion_damage_multiplier,
            g.explosion_lifetime
          ),
        ),
        sys_magic::SpellGenerator::Chainlightning(g) => (
          "Candy Canes",
          format!(
            "Damage: {:.0}\nSpeed: {:.0}\nFirst hit bonus: {:.0}\nDischarges: {:.0}\nMax bounces: {:.0}\nBounce multiplier: {:.2}",
            g.base_damage,
            g.speed,
            g.first_hit_damage,
            g.bounce_children,
            g.max_bounce,
            (1. + g.bounce_mult)
          ),
        ),
        sys_magic::SpellGenerator::Frozenorb(g) => (
          "Cotton Candy",
          format!(
            "Damage: {:.0}\nSpeed: {:.0}\nSize: {:.0}\nShard count: {:.0}\nShard frequency: {:.2}\nShard lifetime: {:0.2}",
            g.base_damage, g.speed, g.orb_size, g.shard_count, g.shard_frequency, g.shard_lifetime
          ),
        ),
      };

      *active_tooltip = Some((
        cmd
          .spawn((
            SpellTooltip,
            Node {
              position_type: PositionType::Absolute,
              bottom: Val::Px(80.0),
              // left: Val::Px(20.0 + (slot.slot_index as f32 * 72.0)),
              padding: UiRect::all(Val::Px(12.0)),
              left: Val::Px(0.0),
              right: Val::Px(0.0),
              flex_direction: FlexDirection::Row,
              justify_content: JustifyContent::Center,
              ..default()
            },
            ZIndex(1000),
          ))
          .with_children(|tooltip_container| {
            tooltip_container
              .spawn((
                Node {
                  flex_direction: FlexDirection::Column,
                  padding: UiRect::all(Val::Px(12.0)),
                  row_gap: Val::Px(4.0),
                  border: UiRect::all(Val::Px(2.0)),
                  ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.95)),
                BorderColor::all(Color::srgb(0.6, 0.6, 0.6)),
              ))
              .with_children(|tooltip| {
                tooltip.spawn((
                  Text::new(spell_name),
                  TextFont {
                    font: font.clone(),
                    font_size: 18.0,
                    ..default()
                  },
                  TextColor(Color::srgb(1.0, 0.9, 0.5)),
                ));
                tooltip.spawn((
                  Text::new(spell_desc),
                  TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                  },
                  TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
                for x in spell.downside.iter() {
                  tooltip.spawn((
                    Text::new(match x {
                      SpellDownside::FriendFire => "Damages you".to_owned(),
                      SpellDownside::ForceMovement { .. } => "Knocks you back".to_owned(),
                      SpellDownside::HpDrain { strength } => {
                        format!("Drains {:.2}% of your HP", strength)
                      }
                    }),
                    TextFont {
                      font: font.clone(),
                      font_size: 14.0,
                      ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.1, 0.1)),
                  ));
                }
              });
          })
          .id(),
        slot.slot_index,
      ));
    }
  }
}
