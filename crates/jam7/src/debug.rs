use std::fmt::Debug;

use bevy::{
  color::palettes::{css::*, tailwind::*},
  prelude::*,
};
use bevy_egui::EguiPrimaryContextPass;
use sys_combat::*;
use sys_magic::{spells::fireball::FireballSpellGenerator, *};
use sys_move::*;

use crate::player::Player;

mod ui;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(DebugConfig {
        show_combat_effects: true,
        ..Default::default()
      })
      .add_systems(
        Update,
        (
          draw_player_ruler,
          draw_spell_ranges,
          draw_combat_effects,
          draw_forces,
        ),
      )
      .add_systems(EguiPrimaryContextPass, ui::boid_config_debug);
  }
}

#[derive(Default, Debug, Clone, Resource)]
pub struct DebugConfig {
  pub show_player_rule: bool,
  pub show_spell_ranges: bool,
  pub show_move_forces: bool,
  pub show_combat_effects: bool,
}

fn draw_player_ruler(
  mut gizmo: Gizmos,
  qry_player: Query<&Transform, With<Player>>,
  config: Res<DebugConfig>,
) {
  if !config.show_player_rule {
    return;
  }

  for player_transform in qry_player {
    let radar_radius = 200.;
    gizmo.ellipse_2d(
      Isometry2d::from_translation(player_transform.translation.xy()),
      Vec2::new(radar_radius * 0.7, radar_radius * 0.35),
      Color::from(PURPLE),
    );
  }
}
fn draw_spell_ranges(
  mut gizmo: Gizmos,
  qry_enemy: Query<(&Transform, &SpellBook)>,
  config: Res<DebugConfig>,
) {
  if !config.show_spell_ranges {
    return;
  }

  for (enemy_transform, sb) in qry_enemy {
    let Some(sp) = sb.spells.first() else {
      continue;
    };
    let SpellGenerator::Fireball(FireballSpellGenerator {
      lifetime, speed, ..
    }) = sp.generator;

    let rad = lifetime * speed;
    gizmo.ellipse_2d(
      Isometry2d::from_translation(enemy_transform.translation.xy()),
      Vec2::new(rad * 0.7, rad * 0.35),
      Color::from(ORANGE_400),
    );
  }
}

pub fn draw_forces(
  mut giz: Gizmos,
  mut qry: Query<(&Placeable, &MoveableVelocity, &Moveable)>,
  qry_stage: Query<(Entity, &IsoMovementStage)>,
  qry_children: Query<&Children>,
  config: Res<DebugConfig>,
) {
  if !config.show_move_forces {
    return;
  }
  for (stage_entity, stage) in qry_stage {
    let Some(children) = qry_children.get(stage_entity).ok() else {
      continue;
    };
    for child in children {
      let Some((p, s, _m)) = qry.get_mut(*child).ok() else {
        continue;
      };

      let origin = p.location.to_screen(stage.aspect_ratio);
      let future_pos = IsoWorldCoords::from(s.world_velocity).to_screen(stage.aspect_ratio);
      giz.ray_2d(origin, future_pos, Color::from(PINK));
    }
  }
}

fn draw_combat_effects(
  mut gizmos: Gizmos,
  qry: Query<(&Combatant, &CombatantGuages, &CombatantState, &Placeable)>,
  qry_aoe: Query<(&CombatAreaEffect, &Placeable)>,
  qry_stage: Query<&IsoMovementStage>,
  config: Res<DebugConfig>,
) {
  if !config.show_combat_effects {
    return;
  }
  let Some(stage) = qry_stage.iter().next() else {
    return;
  };

  for (c, _cg, cs, p) in qry {
    if cs.dead || cs.invulnerable {
      continue;
    }
    c.hitbox.draw_gizmo(
      &mut gizmos,
      p.location,
      stage.aspect_ratio,
      Color::from(GREEN),
    );
  }

  for (c, p) in qry_aoe {
    let Some(stage) = qry_stage.iter().next() else {
      continue;
    };
    c.shape.draw_gizmo(
      &mut gizmos,
      p.location,
      stage.aspect_ratio,
      Color::from(RED),
    );
  }
}
