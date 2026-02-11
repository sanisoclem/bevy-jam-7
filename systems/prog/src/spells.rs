use bevy::{platform::collections::HashMap, prelude::*};
use serde::Deserialize;
use std::hash::Hash;
use sys_magic::spells::fireball::FireballSpellGenerator;

#[derive(Default, Debug, Deserialize, Clone)]
pub struct SpellBuilder<TSpellRoll: Eq + PartialEq + Hash + Clone> {
  // what makes a spell (each one will be rolled once)
  pub rolls: HashMap<TSpellRoll, SpellRoll>,
  pub rolls_per_upgrade: usize,
}

#[derive(Default, Debug, Deserialize, Clone)]
pub struct SpellRoll {
  pub min_value: f32,
  pub max_value: f32,
  pub min_rolls: usize,
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

impl SpellBuilder<FireballSpellRoll> {
  pub fn create_spell(&self) -> Option<FireballSpellGenerator> {
    let get =
      |key: &FireballSpellRoll| -> Option<f32> { Some(self.rolls.get(key)?.roll_minimum()) };

    Some(FireballSpellGenerator {
      speed: get(&FireballSpellRoll::Speed)?,
      lifetime: get(&FireballSpellRoll::Lifetime)?,
      base_damage: get(&FireballSpellRoll::BaseDamage)? as u32,
      radius: get(&FireballSpellRoll::Size)?,
      explosion_radius: get(&FireballSpellRoll::ExplosionRadius)?,
      explosion_damage_multiplier: get(&FireballSpellRoll::ExplosionDamageMult)?,
      explosion_lifetime: get(&FireballSpellRoll::ExplosionDuration)?,
    })
  }

  pub fn create_upgrade(&self) -> SpellUpgrade {
    let mut keys: Vec<&FireballSpellRoll> = self.rolls.keys().collect();
    fastrand::shuffle(&mut keys);

    let upgrades = keys
      .into_iter()
      .take(self.rolls_per_upgrade)
      .filter_map(|key| Some((key.clone(), self.rolls.get(key)?.roll_once())))
      .collect();

    SpellUpgrade::FireballSpellUpgrade(upgrades)
  }
}

pub enum SpellUpgrade {
  FireballSpellUpgrade(HashMap<FireballSpellRoll, f32>),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default, Deserialize)]
pub enum FireballSpellRoll {
  #[default]
  Speed,
  Lifetime,
  BaseDamage,
  Size,
  ExplosionRadius,
  ExplosionDamageMult,
  ExplosionDuration,
}
