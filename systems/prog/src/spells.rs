use bevy::{platform::collections::HashMap, prelude::*};
use serde::Deserialize;
use std::hash::Hash;
use sys_magic::{
  EquippedSpell, SpellDownside, SpellGenerator,
  spells::{
    chainlightning::ChainlightningSpellGenerator, fireball::FireballSpellGenerator,
    frozenorb::FrozenorbSpellGenerator,
  },
};

use crate::levelup::SpellUpgradePerk;

const DEFAULT_SPELL_LIFETIME: f32 = 2.0;

#[derive(Default, Debug, Deserialize, Clone)]
pub struct SpellBuilder {
  pub rolls: HashMap<SpellUpgrade, SpellRoll>,
  pub rolls_per_upgrade: usize,
  pub downside_chance: f32,
  pub max_downside_rolls: usize,
}

#[derive(Default, Debug, Deserialize, Clone)]
pub struct SpellRoll {
  pub description: String,
  pub min_value: f32,
  pub max_value: f32,
  pub min_rolls: usize,
  pub is_downside: bool,
}
impl SpellRoll {
  pub fn roll_once(&self) -> f32 {
    let t = fastrand::f32();
    self.min_value + t * (self.max_value - self.min_value)
  }

  pub fn roll_minimum(&self) -> f32 {
    (0..self.min_rolls).map(|_| self.roll_once()).sum()
  }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
pub enum SpellUpgrade {
  CooldownReduction,
  RemoveDownsides,
  SpellDownsideUpgrade(SpellDownsideUpgrade),
  FireballSpellUpgrade(FireballSpellRoll),
  ChainlightningSpellUpgrade(ChainlightningSpellRoll),
  FrozenorbSpellUpgrade(FrozenorbSpellRoll),
}

impl SpellBuilder {
  pub fn create_upgrade(
    &self,
    spell_index: usize,
    current_spell: &EquippedSpell,
    more_rolls: bool,
  ) -> SpellUpgradePerk {
    let mut upgrade_pool: Vec<(&SpellUpgrade, &SpellRoll)> = self
      .rolls
      .iter()
      .filter(|(upgrade, roll)| !roll.is_downside && is_applicable(upgrade, current_spell))
      .collect();

    let mut downside_pool: Vec<(&SpellUpgrade, &SpellRoll)> = self
      .rolls
      .iter()
      .filter(|(x, roll)| roll.is_downside && is_applicable(x, current_spell))
      .collect();

    let downsides_to_roll = if !downside_pool.is_empty() {
      let mut chance = self.downside_chance;
      let mut count = 0;
      while count < self.max_downside_rolls && fastrand::f32() < chance {
        count += 1;
        chance *= self.downside_chance;
      }
      count
    } else {
      0
    };

    fastrand::shuffle(&mut upgrade_pool);
    fastrand::shuffle(&mut downside_pool);

    let upgrades: Vec<(SpellUpgrade, f32, String)> = upgrade_pool
      .into_iter()
      .take(match (more_rolls, downsides_to_roll > 0) {
        (true, true) => self.rolls_per_upgrade + 3,
        (true, false) => self.rolls_per_upgrade + 2,
        (false, true) => self.rolls_per_upgrade + 2,
        _ => self.rolls_per_upgrade,
      })
      .chain(downside_pool.into_iter().take(downsides_to_roll))
      .map(|(upgrade, roll)| (upgrade.clone(), roll.roll_once(), roll.description.clone()))
      .collect();

    let remove_downsides = upgrades
      .iter()
      .find(|x| matches!(x.0, SpellUpgrade::RemoveDownsides))
      .cloned();

    if let Some(x) = remove_downsides {
      return SpellUpgradePerk {
        upgrades: vec![x],
        slot: spell_index,
      };
    }

    SpellUpgradePerk {
      upgrades,
      slot: spell_index,
    }
  }
  pub fn create_fireball_spell(&self) -> Option<(EquippedSpell, Vec<(SpellUpgrade, f32, String)>)> {
    let mut breakdown = Vec::new();

    let mut get = |key: SpellUpgrade| -> Option<f32> {
      let roll = self.rolls.get(&key)?;
      let value = roll.roll_minimum();
      breakdown.push((key.clone(), value, roll.description.clone()));
      Some(value)
    };

    let spell = EquippedSpell {
      generator: SpellGenerator::Fireball(FireballSpellGenerator {
        speed: get(SpellUpgrade::FireballSpellUpgrade(FireballSpellRoll::Speed))?,
        base_damage: get(SpellUpgrade::FireballSpellUpgrade(
          FireballSpellRoll::BaseDamage,
        ))? as u32,
        radius: get(SpellUpgrade::FireballSpellUpgrade(FireballSpellRoll::Size))?,
        explosion_radius: get(SpellUpgrade::FireballSpellUpgrade(
          FireballSpellRoll::ExplosionRadius,
        ))?,
        explosion_damage_multiplier: 1.
          * get(SpellUpgrade::FireballSpellUpgrade(
            FireballSpellRoll::ExplosionDamageMult,
          ))?,
        explosion_lifetime: get(SpellUpgrade::FireballSpellUpgrade(
          FireballSpellRoll::ExplosionDuration,
        ))?,
        lifetime: DEFAULT_SPELL_LIFETIME,
      }),
      cooldown: Timer::from_seconds(2.3, TimerMode::Once),
      downside: Vec::new(),
    };

    Some((spell, breakdown))
  }

  pub fn create_chainlightning_spell(
    &self,
  ) -> Option<(EquippedSpell, Vec<(SpellUpgrade, f32, String)>)> {
    let mut breakdown = Vec::new();

    let mut get = |key: SpellUpgrade| -> Option<f32> {
      let roll = self.rolls.get(&key)?;
      let value = roll.roll_minimum();
      breakdown.push((key.clone(), value, roll.description.clone()));
      Some(value)
    };

    let spell = EquippedSpell {
      generator: SpellGenerator::Chainlightning(ChainlightningSpellGenerator {
        speed: get(SpellUpgrade::ChainlightningSpellUpgrade(
          ChainlightningSpellRoll::Speed,
        ))?,
        base_damage: get(SpellUpgrade::ChainlightningSpellUpgrade(
          ChainlightningSpellRoll::BaseDamage,
        ))?,
        bounce_children: get(SpellUpgrade::ChainlightningSpellUpgrade(
          ChainlightningSpellRoll::BounceChildren,
        ))?,
        bounce_range: get(SpellUpgrade::ChainlightningSpellUpgrade(
          ChainlightningSpellRoll::BounceRange,
        ))?,
        bounce_mult: get(SpellUpgrade::ChainlightningSpellUpgrade(
          ChainlightningSpellRoll::BounceMult,
        ))?,
      }),
      cooldown: Timer::from_seconds(1.8, TimerMode::Once),
      downside: Vec::new(),
    };

    Some((spell, breakdown))
  }

  pub fn create_frozenorb_spell(
    &self,
  ) -> Option<(EquippedSpell, Vec<(SpellUpgrade, f32, String)>)> {
    let mut breakdown = Vec::new();

    let mut get = |key: SpellUpgrade| -> Option<f32> {
      let roll = self.rolls.get(&key)?;
      let value = roll.roll_minimum();
      breakdown.push((key.clone(), value, roll.description.clone()));
      Some(value)
    };

    let spell = EquippedSpell {
      generator: SpellGenerator::Frozenorb(FrozenorbSpellGenerator {
        speed: get(SpellUpgrade::FrozenorbSpellUpgrade(
          FrozenorbSpellRoll::Speed,
        ))?,
        orb_size: get(SpellUpgrade::FrozenorbSpellUpgrade(
          FrozenorbSpellRoll::OrbSize,
        ))?,
        base_damage: get(SpellUpgrade::FrozenorbSpellUpgrade(
          FrozenorbSpellRoll::BaseDamage,
        ))?,
        shard_frequency: get(SpellUpgrade::FrozenorbSpellUpgrade(
          FrozenorbSpellRoll::ShardFrequency,
        ))?,
        shard_speed: get(SpellUpgrade::FrozenorbSpellUpgrade(
          FrozenorbSpellRoll::ShardSpeed,
        ))?,
        shard_lifetime: get(SpellUpgrade::FrozenorbSpellUpgrade(
          FrozenorbSpellRoll::ShardLifetime,
        ))?,
        shard_damage_mult: get(SpellUpgrade::FrozenorbSpellUpgrade(
          FrozenorbSpellRoll::ShardDamageMult,
        ))?,
        shard_count: get(SpellUpgrade::FrozenorbSpellUpgrade(
          FrozenorbSpellRoll::ShardCount,
        ))?,
      }),
      cooldown: Timer::from_seconds(1.5, TimerMode::Once),
      downside: Vec::new(),
    };

    Some((spell, breakdown))
  }
}

fn is_applicable(upgrade: &SpellUpgrade, spell: &EquippedSpell) -> bool {
  matches!(
    (upgrade, &spell.generator, spell.downside.is_empty()),
    (
      SpellUpgrade::FireballSpellUpgrade(_),
      SpellGenerator::Fireball(_),
      _
    ) | (
      SpellUpgrade::ChainlightningSpellUpgrade(_),
      SpellGenerator::Chainlightning(_),
      _
    ) | (
      SpellUpgrade::FrozenorbSpellUpgrade(_),
      SpellGenerator::Frozenorb(_),
      _
    ) | (SpellUpgrade::CooldownReduction, _, _)
      | (SpellUpgrade::RemoveDownsides, _, false)
      | (SpellUpgrade::SpellDownsideUpgrade(_), _, _)
  )
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
pub enum SpellDownsideUpgrade {
  DownsideAddFriendlyFire,
  DownsideForceMovement,
  DownsideHpDrain,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default, Deserialize)]
pub enum FireballSpellRoll {
  #[default]
  Speed,
  BaseDamage,
  Size,
  ExplosionRadius,
  ExplosionDamageMult,
  ExplosionDuration,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default, Deserialize)]
pub enum ChainlightningSpellRoll {
  #[default]
  Speed,
  BaseDamage,
  BounceChildren,
  BounceRange,
  BounceMult,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default, Deserialize)]
pub enum FrozenorbSpellRoll {
  #[default]
  Speed,
  OrbSize,
  BaseDamage,
  ShardFrequency,
  ShardLifetime,
  ShardSpeed,
  ShardCount,
  ShardDamageMult,
}

pub trait SpellRollUpgrade: Sized + Clone + Eq + Hash {
  fn into_upgrade(self, value: f32) -> SpellUpgrade;
}

pub fn upgrade_spell(spell: &mut EquippedSpell, upgrades: &Vec<(SpellUpgrade, f32, String)>) {
  for (key, value, _) in upgrades {
    match (key, &mut spell.generator) {
      (SpellUpgrade::CooldownReduction, _) => {
        spell.cooldown = Timer::from_seconds(
          spell.cooldown.duration().as_secs_f32() * value,
          TimerMode::Repeating,
        );
      }
      (SpellUpgrade::RemoveDownsides, _) => {
        spell.downside.clear();
      }
      (SpellUpgrade::SpellDownsideUpgrade(downside), _) => {
        spell.downside.push(match downside {
          SpellDownsideUpgrade::DownsideAddFriendlyFire => SpellDownside::FriendFire,
          SpellDownsideUpgrade::DownsideForceMovement => SpellDownside::ForceMovement {
            strength: *value,
            duration: 0.1,
          },
          SpellDownsideUpgrade::DownsideHpDrain => SpellDownside::HpDrain { strength: *value },
        });
      }

      // fireball
      (
        SpellUpgrade::FireballSpellUpgrade(FireballSpellRoll::Speed),
        SpellGenerator::Fireball(g),
      ) => g.speed += value,
      (
        SpellUpgrade::FireballSpellUpgrade(FireballSpellRoll::BaseDamage),
        SpellGenerator::Fireball(g),
      ) => g.base_damage += *value as u32,
      (
        SpellUpgrade::FireballSpellUpgrade(FireballSpellRoll::Size),
        SpellGenerator::Fireball(g),
      ) => g.radius += value,
      (
        SpellUpgrade::FireballSpellUpgrade(FireballSpellRoll::ExplosionRadius),
        SpellGenerator::Fireball(g),
      ) => g.explosion_radius += value,
      (
        SpellUpgrade::FireballSpellUpgrade(FireballSpellRoll::ExplosionDamageMult),
        SpellGenerator::Fireball(g),
      ) => g.explosion_damage_multiplier *= value,
      (
        SpellUpgrade::FireballSpellUpgrade(FireballSpellRoll::ExplosionDuration),
        SpellGenerator::Fireball(g),
      ) => g.explosion_lifetime += value,

      // chainlightning
      (
        SpellUpgrade::ChainlightningSpellUpgrade(ChainlightningSpellRoll::Speed),
        SpellGenerator::Chainlightning(g),
      ) => g.speed += value,
      (
        SpellUpgrade::ChainlightningSpellUpgrade(ChainlightningSpellRoll::BaseDamage),
        SpellGenerator::Chainlightning(g),
      ) => g.base_damage += value,
      (
        SpellUpgrade::ChainlightningSpellUpgrade(ChainlightningSpellRoll::BounceChildren),
        SpellGenerator::Chainlightning(g),
      ) => g.bounce_children += value,
      (
        SpellUpgrade::ChainlightningSpellUpgrade(ChainlightningSpellRoll::BounceRange),
        SpellGenerator::Chainlightning(g),
      ) => g.bounce_range += value,
      (
        SpellUpgrade::ChainlightningSpellUpgrade(ChainlightningSpellRoll::BounceMult),
        SpellGenerator::Chainlightning(g),
      ) => g.bounce_mult += value,

      // frozenorb
      (
        SpellUpgrade::FrozenorbSpellUpgrade(FrozenorbSpellRoll::Speed),
        SpellGenerator::Frozenorb(g),
      ) => g.speed += value,
      (
        SpellUpgrade::FrozenorbSpellUpgrade(FrozenorbSpellRoll::OrbSize),
        SpellGenerator::Frozenorb(g),
      ) => g.orb_size += value,
      (
        SpellUpgrade::FrozenorbSpellUpgrade(FrozenorbSpellRoll::BaseDamage),
        SpellGenerator::Frozenorb(g),
      ) => g.base_damage += value,
      (
        SpellUpgrade::FrozenorbSpellUpgrade(FrozenorbSpellRoll::ShardFrequency),
        SpellGenerator::Frozenorb(g),
      ) => g.shard_frequency += value,
      (
        SpellUpgrade::FrozenorbSpellUpgrade(FrozenorbSpellRoll::ShardSpeed),
        SpellGenerator::Frozenorb(g),
      ) => g.shard_speed += value,
      (
        SpellUpgrade::FrozenorbSpellUpgrade(FrozenorbSpellRoll::ShardLifetime),
        SpellGenerator::Frozenorb(g),
      ) => g.shard_lifetime += value,
      (
        SpellUpgrade::FrozenorbSpellUpgrade(FrozenorbSpellRoll::ShardCount),
        SpellGenerator::Frozenorb(g),
      ) => g.shard_count += value,
      (
        SpellUpgrade::FrozenorbSpellUpgrade(FrozenorbSpellRoll::ShardDamageMult),
        SpellGenerator::Frozenorb(g),
      ) => g.shard_lifetime += value,
      _ => {
        warn!("Unprocessed upgrade, possible misconfiguration");
      }
    }
  }
}
