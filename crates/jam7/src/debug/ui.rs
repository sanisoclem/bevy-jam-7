use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::debug::DebugConfig;

pub fn boid_config_debug(mut config: ResMut<DebugConfig>, mut contexts: EguiContexts) -> Result {
  egui::Window::new("Debug Info").show(contexts.ctx_mut()?, |ui| {
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

  Ok(())
}
