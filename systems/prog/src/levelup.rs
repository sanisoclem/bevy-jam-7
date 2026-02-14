use crate::{
  LongTermProgFeature, LongTermProgger, Progger,
  death::RequestGameRestart,
  spells::{SpellBuilder, SpellUpgrade, upgrade_spell},
};
use bevy::prelude::*;
use sys_candy::CandySettings;
use sys_combat::{Combatant, CombatantGuages};
use sys_magic::{EquippedSpell, EquippedSpellState, SpellBook, SpellBookState, SpellGenerator};

pub mod ui;

#[derive(Debug, Clone)]
pub enum LevelUpPerk {
  NewSpell(EquippedSpell, Vec<(SpellUpgrade, f32, String)>),
  SpellUpgradePerk(SpellUpgradePerk),
}

#[derive(Debug, Clone)]
pub struct SpellUpgradePerk {
  pub upgrades: Vec<(SpellUpgrade, f32, String)>,
  pub slot: usize,
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
  let num = if lprog.has_upgrade(LongTermProgFeature::MoreChoices) {
    lprog.num_perk_choices * 2
  } else {
    lprog.num_perk_choices
  };

  for _ in 0..num {
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
  // let has_fireball = true;

  let mut available: Vec<
    fn(&SpellBuilder) -> Option<(EquippedSpell, Vec<(SpellUpgrade, f32, String)>)>,
  > = Vec::new();
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
  available[idx](builder).map(|(x, y)| LevelUpPerk::NewSpell(x, y))
}

fn generate_upgrade_perk(sb: &SpellBook, lprog: &LongTermProgger) -> Option<LevelUpPerk> {
  if sb.spells.is_empty() {
    return None;
  }
  let builder = lprog.spell_builder.as_ref()?;

  let idx = fastrand::usize(0..sb.spells.len());
  let equipped = &sb.spells[idx];
  Some(LevelUpPerk::SpellUpgradePerk(builder.create_upgrade(
    idx,
    equipped,
    lprog.has_upgrade(LongTermProgFeature::BetterSpellUpgrades),
  )))
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
    .insert(PendingLevelUp {
      choices: generate_levelup_choices(sb, &lprog),
    })
    .trigger(ShowPendingLevelUpUi);
  time.pause();
}

pub fn on_apply_levelup(
  evt: On<ApplyLevelUp>,
  mut cmd: Commands,
  mut qry: Query<(
    &mut SpellBook,
    &mut Progger,
    &mut Combatant,
    &mut CombatantGuages,
    &mut SpellBookState,
    &PendingLevelUp,
  )>,
  mut time: ResMut<Time<Virtual>>,
  lprog: Res<LongTermProgger>,
) {
  let Some((mut sb, mut prog, mut c, mut g, mut ss, pending)) = qry.get_mut(evt.target).ok() else {
    return;
  };

  let Some(selection) = pending.choices.get(evt.slot) else {
    return;
  };

  // upgrade spellbook
  match selection {
    LevelUpPerk::NewSpell(s, _) => {
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

  let more_hp = if lprog.has_upgrade(LongTermProgFeature::MoarHp) {
    prog.hp_gain * 2
  } else {
    prog.hp_gain
  };

  // upgrade combatant
  prog.level += 1;
  c.max_hp += more_hp;
  g.current_hp += more_hp;
  g.reeling_timer = None;
  g.stun_timer = None;

  cmd.entity(evt.target).remove::<PendingLevelUp>();

  time.unpause();
}

pub fn on_game_restart(
  _evt: On<RequestGameRestart>,
  mut candy: ResMut<CandySettings>,
  lprog: Res<LongTermProgger>,
) {
  candy.aberrate_on_kill = !lprog.has_upgrade(LongTermProgFeature::SeeInSingle);
}
