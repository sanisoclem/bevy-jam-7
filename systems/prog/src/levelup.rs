use crate::{
  LongTermProgger,
  spells::{SpellBuilder, SpellUpgrade, upgrade_spell},
};
use bevy::prelude::*;
use sys_magic::{EquippedSpell, EquippedSpellState, SpellBook, SpellBookState, SpellGenerator};

pub mod ui;

#[derive(Debug, Clone)]
pub enum LevelUpPerk {
  NewSpell(EquippedSpell),
  SpellUpgradePerk(SpellUpgradePerk),
}

#[derive(Debug, Clone)]
pub struct SpellUpgradePerk {
  pub upgrades: Vec<(SpellUpgrade, f32, String)>,
  pub slot: usize,
}
impl std::fmt::Display for SpellUpgradePerk {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "Upgrade for slot {}", self.slot)?;
    for (_upgrade, value, description) in &self.upgrades {
      // use the description as a format template
      let formatted = description.replace("{}", &format!("{:.1}", value));
      writeln!(f, "  - {}", formatted)?;
    }
    Ok(())
  }
}
#[derive(EntityEvent)]
pub struct LevelUp {
  #[event_target]
  pub target: Entity,
}

#[derive(EntityEvent)]
pub struct ApplyLevelUp {
  #[event_target]
  pub target: Entity,
  pub slot: usize,
}

#[derive(EntityEvent)]
pub struct ShowPendingLevelUpUi(Entity);

#[derive(Component, Default)]
pub struct PendingLevelUp {
  pub choices: Vec<LevelUpPerk>,
}

fn generate_levelup_choices(sb: &SpellBook, lprog: &LongTermProgger) -> Vec<LevelUpPerk> {
  let max_new_spells = lprog.max_spells - sb.spells.len();
  let mut choices = Vec::new();

  for _ in 0..lprog.num_perk_choices {
    let pick_new = sb.spells.is_empty() || (max_new_spells > 0 && fastrand::f32() < 0.5);

    if pick_new && let Some(perk) = generate_new_spell_perk(sb, lprog) {
      choices.push(perk);
      continue;
    }

    // fallback
    if let Some(perk) = generate_upgrade_perk(sb, lprog) {
      choices.push(perk);
    }
  }
  debug!("generated choies: {:?}", choices);

  choices
}

fn generate_new_spell_perk(sb: &SpellBook, lprog: &LongTermProgger) -> Option<LevelUpPerk> {
  let builder = lprog.spell_builder.as_ref()?;

  let has_fireball = sb
    .spells
    .iter()
    .any(|s| matches!(s.generator, SpellGenerator::Fireball(_)));
  let has_chainlightning = sb
    .spells
    .iter()
    .any(|s| matches!(s.generator, SpellGenerator::Chainlightning(_)));
  let has_frozenorb = sb
    .spells
    .iter()
    .any(|s| matches!(s.generator, SpellGenerator::Frozenorb(_)));
  let has_fireball = true;

  let mut available: Vec<fn(&SpellBuilder) -> Option<EquippedSpell>> = Vec::new();
  if !has_fireball {
    available.push(|b| b.create_fireball_spell());
  }
  if !has_chainlightning {
    available.push(|b| b.create_chainlightning_spell());
  }
  if !has_frozenorb {
    available.push(|b| b.create_frozenorb_spell());
  }

  if available.is_empty() {
    return None;
  }

  let idx = fastrand::usize(0..available.len());
  available[idx](builder).map(LevelUpPerk::NewSpell)
}

fn generate_upgrade_perk(sb: &SpellBook, lprog: &LongTermProgger) -> Option<LevelUpPerk> {
  if sb.spells.is_empty() {
    return None;
  }
  let builder = lprog.spell_builder.as_ref()?;

  let idx = fastrand::usize(0..sb.spells.len());
  let equipped = &sb.spells[idx];
  Some(LevelUpPerk::SpellUpgradePerk(
    builder.create_upgrade(idx, equipped),
  ))
}

pub fn on_levelup(
  evt: On<LevelUp>,
  mut cmd: Commands,
  qry: Query<&SpellBook>,
  mut time: ResMut<Time<Virtual>>,
  lprog: Res<LongTermProgger>,
) {
  let Some(sb) = qry.get(evt.target).ok() else {
    return;
  };

  cmd
    .entity(evt.target)
    .insert_if_new(PendingLevelUp {
      choices: generate_levelup_choices(sb, &lprog),
    })
    .trigger(ShowPendingLevelUpUi);
  time.pause();
}

pub fn on_apply_levelup(
  evt: On<ApplyLevelUp>,
  mut cmd: Commands,
  mut qry: Query<(&mut SpellBook, &mut SpellBookState, &PendingLevelUp)>,
  mut time: ResMut<Time<Virtual>>,
) {
  let Some((mut sb, mut ss, pending)) = qry.get_mut(evt.target).ok() else {
    return;
  };

  let Some(selection) = pending.choices.get(evt.slot) else {
    return;
  };

  match selection {
    LevelUpPerk::NewSpell(s) => {
      sb.spells.push(s.clone());
      ss.spells_states.push(EquippedSpellState::default());
    }
    LevelUpPerk::SpellUpgradePerk(u) => {
      let Some(existing) = sb.spells.get_mut(u.slot) else {
        return;
      };

      upgrade_spell(existing, &u.upgrades);
    }
  }
  cmd.entity(evt.target).remove::<PendingLevelUp>();

  time.unpause();
}
