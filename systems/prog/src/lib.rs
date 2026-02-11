use bevy::prelude::*;

#[derive(Component)]
pub struct Progger {
  pub level: u32,
  pub base_hp: u32,
  pub hp_gain: u32,
}

#[derive(Resource, Default)]
pub struct LongTermProgger {
  pub lucidty: u32,
  pub runs: u32,
  pub trophies: Vec<ProggerTrophy>,
}

pub enum ProggerTrophy {
  Deal1000DPS,
  Die5Times,
  DefeatFinalBoss,
}
