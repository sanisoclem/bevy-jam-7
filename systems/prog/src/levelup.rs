use crate::{
  LongTermProgger,
  spells::{SpellBuilder, SpellUpgrade},
};
use bevy::prelude::*;
use sys_magic::{EquippedSpell, EquippedSpellState, SpellBook, SpellBookState, SpellGenerator};

pub mod ui;

pub enum LevelUpPerk {
  NewSpell(EquippedSpell),
  SpellUpgradePerk(SpellUpgradePerk),
}

pub struct SpellUpgradePerk {
  pub upgrades: Vec<(SpellUpgrade, f32)>,
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
  pub perk: LevelUpPerk,
}

#[derive(Resource, Default)]
pub struct PendingLevelUp {
  pub target: Option<Entity>,
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
    builder.create_upgrade(equipped),
  ))
}

pub fn on_levelup(
  evt: On<LevelUp>,
  qry: Query<&SpellBook>,
  mut time: ResMut<Time<Virtual>>,
  lprog: Res<LongTermProgger>,
  mut pending: ResMut<PendingLevelUp>,
) {
  let Some(sb) = qry.get(evt.target).ok() else {
    return;
  };

  pending.target = Some(evt.target);
  pending.choices = generate_levelup_choices(sb, &lprog);
  time.pause();
}

pub fn on_apply_levelup(
  evt: On<ApplyLevelUp>,
  mut qry: Query<(&mut SpellBook, &mut SpellBookState)>,
) {
  let Some((mut sb, mut ss)) = qry.get_mut(evt.target).ok() else {
    return;
  };

  match &evt.perk {
    LevelUpPerk::NewSpell(s) => {
      sb.spells.push(s.clone());
      ss.spells_states.push(EquippedSpellState::default());
    }
    LevelUpPerk::SpellUpgradePerk(u) => {
      todo!()
    }
  }
}
