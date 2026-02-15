use crate::{components::*, spells::fireball::FireballSpellGenerator};
use bevy::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct SpellBookGenerator;

impl SpellBookGenerator {
  pub fn create_spellbook(
    &self,
    _num_spells: u32,
    effective_range: f32,
    effective_dps: f32,
    aps: f32,
  ) -> (SpellBook, SpellBookState) {
    (
      SpellBook {
        spells: vec![EquippedSpell {
          generator: SpellGenerator::Fireball(FireballSpellGenerator {
            radius: 8. + (effective_dps / 30.),
            base_damage: (effective_dps / aps) as u32,
            lifetime: 2.,
            speed: effective_range / 2.,
            explosion_lifetime: 1.,
            explosion_damage_multiplier: 2.5,
            explosion_radius: 30. + (effective_dps),
          }),
          cooldown: Timer::from_seconds(1. / aps, TimerMode::Repeating),
          downside: Vec::new(),
        }],
        disabled: false,
      },
      SpellBookState {
        spells_states: vec![EquippedSpellState::default()],
      },
    )
  }
}
