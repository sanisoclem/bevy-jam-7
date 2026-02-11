use bevy::prelude::*;

#[derive(Clone, Debug, Reflect)]
pub struct FrozenorbSpellGenerator {
  pub speed: f32,
  pub orb_size: f32,
  pub base_damage: f32,
  pub shard_cooldown: f32,
  pub shard_size: f32,
  pub shard_speed: f32,
}
