use bevy::{ecs::relationship::Relationship, prelude::*};
use sys_magic::{SpellBook, SpellBookState};

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
      justify_items: JustifyItems::Center,
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
