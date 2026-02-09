pub mod assets;
pub mod easing;

pub mod dps {
  pub const TEAM_PLAYER: u8 = 0;
  pub const TEAM_ENEMY: u8 = 0;
  pub fn get_max_hp_from_toughness_score(toughness_score: f32) -> u32 {
    // TODO: how does toughness relate to other scores
    (toughness_score * 10.).floor() as u32
  }
  pub fn get_effective_range_from_rangeness_score(rangeness_score: f32) -> f32 {
    rangeness_score * 20.
  }
  pub fn get_effective_dps_from_offense_score(offense_score: f32) -> f32 {
    offense_score * 10.
  }
  pub fn get_density_ceiling_from_score(density_score: f32) -> f32 {
    density_score * 0.7
  }
  pub fn get_enemy_size_from_density(density_score: f32) -> f32 {
    density_score / 0.8
  }

  pub fn normalize_scores(power_budget: f32, scores: [f32; 4]) -> [f32; 4] {
    let total: f32 = scores.iter().copied().sum();
    scores.map(|x| x.powi(2) / total * power_budget)
  }
}
