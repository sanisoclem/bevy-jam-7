use crate::{LongTermProgger, spells::SpellUpgrade};
use bevy::prelude::*;
use sys_magic::{EquippedSpell, EquippedSpellState, SpellBook, SpellBookState, SpellGenerator};

pub mod ui;

pub enum LevelUpPerk {
  NewSpell(NewSpellPerk),
  SpellUpgradePerk(SpellUpgradePerk),
}

pub struct NewSpellPerk {
  pub generator: SpellGenerator,
}

pub struct SpellUpgradePerk {
  pub upgrade: SpellUpgrade,
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
    let pick_new = max_new_spells > 0 && fastrand::f32() < 0.5;

    if pick_new {
      if let Some(perk) = generate_new_spell_perk(sb, lprog) {
        choices.push(perk);
        continue;
      }
    }

    // fallback
    if let Some(perk) = generate_upgrade_perk(sb, lprog) {
      choices.push(perk);
    }
  }

  choices
}

fn generate_new_spell_perk(sb: &SpellBook, lprog: &LongTermProgger) -> Option<LevelUpPerk> {
  let existing: Vec<std::mem::Discriminant<SpellGenerator>> = sb
    .spells
    .iter()
    .map(|s| std::mem::discriminant(&s.generator))
    .collect();

  let mut available: Vec<SpellGenerator> = vec![
    lprog
      .fireball_builder
      .as_ref()
      .and_then(|b| b.create_spell())
      .map(SpellGenerator::Fireball),
    // lprog
    //   .chainlightning_builder
    //   .as_ref()
    //   .and_then(|b| b.create_spell())
    //   .map(SpellGenerator::Chainlightning),
  ]
  .into_iter()
  .flatten()
  .filter(|g| !existing.contains(&std::mem::discriminant(g)))
  .collect();

  if available.is_empty() {
    return None;
  }

  let idx = fastrand::usize(0..available.len());
  Some(LevelUpPerk::NewSpell(NewSpellPerk {
    generator: available.remove(idx),
  }))
}

fn generate_upgrade_perk(sb: &SpellBook, lprog: &LongTermProgger) -> Option<LevelUpPerk> {
  if sb.spells.is_empty() {
    return None;
  }

  let idx = fastrand::usize(0..sb.spells.len());
  let equipped = &sb.spells[idx];

  let upgrade = match &equipped.generator {
    SpellGenerator::Fireball(_) => lprog.fireball_builder.as_ref()?.create_upgrade(),
    // SpellGenerator::Chainlightning(_) => lprog.chainlightning_builder.as_ref()?.create_upgrade(),
  };

  Some(LevelUpPerk::SpellUpgradePerk(SpellUpgradePerk { upgrade }))
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
      sb.spells.push(EquippedSpell {
        generator: s.generator.clone(),
        cooldown: Timer::from_seconds(0.1, TimerMode::Repeating),
        downside: Some(sys_magic::SpellDownside::HpDrain { strength: 100. }),
      });
      ss.spells_states.push(EquippedSpellState::default());
    }
    LevelUpPerk::SpellUpgradePerk(u) => {
      todo!()
    }
  }
}
