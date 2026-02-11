use bevy::prelude::*;
#[derive(Clone, Debug, Reflect)]
pub struct ChainlightningSpellGenerator {
  pub speed: f32,
  pub base_damage: f32,
  pub num_chains: f32,
  pub bounce_mult: f32,
  pub bounce_range: f32,
}

