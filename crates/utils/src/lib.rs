pub mod assets;
pub mod easing;

// should contain all the variables required to tune difficulty
pub mod diff {
  use bevy::prelude::*;

  pub const TEAM_PLAYER: u8 = 1;
  pub const TEAM_ENEMY: u8 = 0;
  pub fn get_power_budget_from_time(time: f32) -> f32 {
    1.2f32.powf((time / 60.).floor()).clamp(0.1, 10.)
  }
  pub fn get_max_hp_from_toughness_score(toughness_score: f32) -> u32 {
    // TODO: how does toughness relate to other scores
    (toughness_score * 100.).floor() as u32
  }
  pub fn get_effective_range_from_rangeness_score(rangeness_score: f32) -> f32 {
    rangeness_score * 20.
  }
  pub fn get_effective_dps_from_offense_score(offense_score: f32) -> f32 {
    offense_score * 10.
  }
  pub fn get_density_ceiling_from_score(density_score: f32) -> f32 {
    density_score * 5.0
  }
  pub fn get_enemy_size_from_density(density_score: f32) -> f32 {
    let retval = 1. + ((0.1 / density_score).floor() * 0.5);
    info!("getting size {:?} for density {:?}", retval, density_score);
    retval
  }
  pub fn get_enemy_tint(toughness_score: f32, rangeness_score: f32, offense_score: f32) -> Color {
    let max = toughness_score.max(rangeness_score).max(offense_score);

    let t = (toughness_score / max).powi(3);
    let r = (rangeness_score / max).powi(3);
    let o = (offense_score / max).powi(3);

    let red = o;
    let green = r;
    let blue = t;

    let channel_max = red.max(green).max(blue);
    Color::srgba(
      red / channel_max,
      green / channel_max,
      blue / channel_max,
      1.0,
    )
  }
  pub fn normalize_scores(power_budget: f32, scores: [f32; 4]) -> [f32; 4] {
    let total: f32 = scores.iter().copied().sum();
    scores.map(|x| x.powi(2) / total * power_budget)
  }
}
