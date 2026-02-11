use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use sys_combat::{Combatant, CombatantGuages, KillCounter};
use sys_magic::SpellBook;
use utils::diff::{
  get_effective_dps_from_offense_score, get_effective_range_from_rangeness_score,
  get_max_hp_from_toughness_score, get_power_budget_from_kills, normalize_scores,
};

use crate::debug::DebugConfig;

pub fn boid_config_debug(
  mut config: ResMut<DebugConfig>,
  mut contexts: EguiContexts,
  mut qry: Query<(&Combatant, &CombatantGuages, &KillCounter, &mut SpellBook)>,
) -> Result {
  let Some((c, cg, kills, mut spellbook)) = qry.iter_mut().next() else {
    return Ok(());
  };

  egui::Window::new("Debug")
    .default_pos(egui::pos2(10., 130.0))
    .default_width(200.)
    .show(contexts.ctx_mut()?, |ui| {
      ui.collapsing("Gizmos", |ui| {
        ui.add(egui::Checkbox::new(
          &mut config.show_player_rule,
          "Show Player Ruler",
        ));
        ui.add(egui::Checkbox::new(
          &mut config.show_move_forces,
          "Show Forces",
        ));
        ui.add(egui::Checkbox::new(
          &mut config.show_spell_ranges,
          "Show Spell Ranges",
        ));
        ui.add(egui::Checkbox::new(
          &mut config.show_combat_effects,
          "Show Combat Effects",
        ));
      });
      ui.separator();
      egui::CollapsingHeader::new("Player")
        .default_open(true)
        .show(ui, |ui| {
          ui.label(format!("HP {}/{}", cg.current_hp, c.max_hp));
          ui.label(format!("Kills {:.3}", kills.kills));
          ui.add(egui::Checkbox::new(
            &mut spellbook.disabled,
            "Disable spellbook",
          ));
        });

      egui::CollapsingHeader::new("Enemies")
        .default_open(true)
        .show(ui, |ui| {
          let power_budget = get_power_budget_from_kills(kills.kills as f32);
          let normalized = normalize_scores(power_budget, [1., 2., 3., 4.]);
          ui.label(format!("Power Budget {:.2}", power_budget));
          ui.label(format!(
            "Score range {:.2}-{:.2}",
            normalized[0], normalized[3]
          ));
          ui.label(format!(
            "HP {}-{}",
            get_max_hp_from_toughness_score(normalized[0]),
            get_max_hp_from_toughness_score(normalized[3])
          ));
          ui.label(format!(
            "Attack range {:.2}-{:.2}",
            get_effective_range_from_rangeness_score(normalized[0]),
            get_effective_range_from_rangeness_score(normalized[3])
          ));
          ui.label(format!(
            "DPS {:.2}-{:.2}",
            get_effective_dps_from_offense_score(normalized[0]),
            get_effective_dps_from_offense_score(normalized[3])
          ));
        });
    });

  Ok(())
}
