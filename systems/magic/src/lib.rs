use bevy::prelude::*;
use sys_combat::CombatantState;

mod components;
mod generator;
pub mod spells;

pub use components::*;
pub use generator::*;

pub struct SysMagicPlugin;

impl Plugin for SysMagicPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_observer(spells::fireball::cast_fireball)
      .add_observer(spells::fireball::on_fireball_detonate)
      .add_observer(spells::chainlightning::cast_chainlightning)
      .add_observer(spells::chainlightning::on_detonate_chainlightning)
      .add_observer(spells::frozenorb::cast_frozenorb)
      .add_observer(spells::frozenorb::on_frozenorb_detonate)
      .add_observer(spells::frozenorb::on_frozenorb_shard_detonate)
      .add_systems(FixedUpdate, (update_cooldowns, cast_auto_spells));
  }
}

fn update_cooldowns(qry: Query<&mut SpellBookState>, time: Res<Time>) {
  for mut sbs in qry {
    for spell_state in sbs.spells_states.iter_mut() {
      if let Some(cd) = spell_state.cooldown.as_mut() {
        cd.tick(time.delta());
        if cd.is_finished() {
          spell_state.cooldown = None;
        }
      }
    }
  }
}

fn cast_auto_spells(
  mut cmd: Commands,
  qry: Query<(Entity, &CombatantState, &SpellBook, &mut SpellBookState)>,
) {
  for (caster, cs, spellbook, mut sbs) in qry {
    if spellbook.disabled {
      continue;
    }

    for (idx, (spell, spell_state)) in spellbook
      .spells
      .iter()
      .zip(sbs.spells_states.iter_mut())
      .enumerate()
    {
      if cs.dead
        || cs.reeling
        || cs.stunned
        || spell_state
          .cooldown
          .as_ref()
          .is_some_and(|c| !c.is_finished())
      {
        continue;
      }

      match &spell.generator {
        SpellGenerator::Fireball(generator) => cmd.trigger(SpellReady {
          caster,
          generator: generator.clone(),
          downside: spell.downside.clone(),
          spell_slot: idx,
          cooldown: spell.cooldown.clone(),
        }),
        SpellGenerator::Chainlightning(generator) => cmd.trigger(SpellReady {
          caster,
          generator: generator.clone(),
          downside: spell.downside.clone(),
          spell_slot: idx,
          cooldown: spell.cooldown.clone(),
        }),
        SpellGenerator::Frozenorb(generator) => cmd.trigger(SpellReady {
          caster,
          generator: generator.clone(),
          downside: spell.downside.clone(),
          spell_slot: idx,
          cooldown: spell.cooldown.clone(),
        }),
      };
    }
  }
}
