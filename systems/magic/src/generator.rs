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
  ) -> (SpellBook, SpellBookState) {
    (
      SpellBook {
        spells: vec![EquippedSpell {
          generator: SpellGenerator::Fireball(FireballSpellGenerator {
            radius: 3.,
            base_damage: 3,
            lifetime: 2.,
            speed: effective_range / 2.,
            explosion_lifetime: 1.,
            explosion_damage_multiplier: 2.5,
            explosion_radius: 30.,
          }),
          cooldown: Timer::from_seconds(10. / effective_dps, TimerMode::Repeating),
          downside: None,
        }],
        disabled: false,
      },
      SpellBookState {
        spells_states: vec![EquippedSpellState::default()],
      },
    )
  }
}
