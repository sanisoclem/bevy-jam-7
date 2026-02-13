use bevy::prelude::*;
use std::fmt::Debug;

use crate::spells::{
  chainlightning::ChainlightningSpellGenerator, fireball::FireballSpellGenerator,
  frozenorb::FrozenorbSpellGenerator,
};

#[derive(Component, Debug, Reflect, Clone, Default)]
pub struct SpellBook {
  pub spells: Vec<EquippedSpell>,
  pub disabled: bool,
}
#[derive(Clone, Debug, Reflect)]
pub struct EquippedSpell {
  pub generator: SpellGenerator,
  pub cooldown: Timer,
  pub downside: Vec<SpellDownside>,
}

#[derive(Component, Debug, Reflect, Clone, Default)]
pub struct SpellBookState {
  pub spells_states: Vec<EquippedSpellState>,
}
#[derive(Debug, Reflect, Clone, Default)]
pub struct EquippedSpellState {
  pub cooldown: Option<Timer>,
}

#[derive(Debug, Reflect, Clone)]
pub enum SpellDownside {
  FriendFire,
  ForceMovement { strength: f32, duration: f32 },
  HpDrain { strength: f32 },
}

#[derive(Clone, Debug, Reflect)]
pub enum SpellGenerator {
  Fireball(FireballSpellGenerator),
  Chainlightning(ChainlightningSpellGenerator),
  Frozenorb(FrozenorbSpellGenerator),
}

#[derive(EntityEvent, Clone, Debug, Reflect)]
pub struct SpellReady<T> {
  #[event_target]
  pub caster: Entity,
  pub generator: T,
  pub downside: Vec<SpellDownside>,
  pub spell_slot: usize,
  pub cooldown: Timer,
}

#[derive(Message, Clone, Debug, Reflect)]
pub struct CastSpell<TSpellInstance> {
  pub caster: Entity,
  pub spawn_parent: Entity,
  pub team: u8,
  pub spell: TSpellInstance,
}
