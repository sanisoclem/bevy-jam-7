pub mod assets;
pub mod easing;

// should contain all the variables required to tune difficulty
pub mod diff {
  use bevy::prelude::*;

  pub const TEAM_PLAYER: u8 = 1;
  pub const TEAM_ENEMY: u8 = 0;
  pub fn get_power_budget_from_kills(kills: f32) -> f32 {
    ((kills.max(1.0).log10() - 1.0) * 4.0).max(1.0)
  }

  pub fn get_max_hp_from_toughness_score(toughness_score: f32) -> u32 {
    100 + (toughness_score * 500.).floor() as u32
  }
  pub fn get_effective_range_from_rangeness_score(rangeness_score: f32) -> f32 {
    100. + rangeness_score * 200.
  }
  pub fn get_effective_dps_from_offense_score(offense_score: f32) -> f32 {
    10. + offense_score * 5.
  }
  pub fn get_density_ceiling_from_score(density_score: f32) -> f32 {
    (density_score / 10000.).clamp(0.000001, 0.001)
  }
  pub fn get_enemy_size_from_toughness(toughness_score: f32) -> f32 {
    1.0 + toughness_score * 0.1
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
    let retval = Color::srgba(
      red / channel_max,
      green / channel_max,
      blue / channel_max,
      1.0,
    );
    debug!(
      "tinting {}, {}, {} = {:?}",
      toughness_score, rangeness_score, offense_score, retval
    );
    retval
  }
  // pub fn normalize_scores(power_budget: f32, scores: [f32; 4]) -> [f32; 4] {
  //   let total: f32 = scores.iter().copied().sum();
  //   scores.map(|x| x / total * power_budget)
  // }
  pub fn normalize_scores(power_budget: f32, scores: [f32; 4]) -> [f32; 4] {
    let mut result = scores;

    let i1 = (0..4)
      .max_by(|&a, &b| scores[a].partial_cmp(&scores[b]).unwrap())
      .unwrap();
    let i2 = (0..4)
      .filter(|&i| i != i1)
      .max_by(|&a, &b| scores[a].partial_cmp(&scores[b]).unwrap())
      .unwrap();

    result = std::array::from_fn(|i| {
      if i == i1 {
        result[i]
      } else {
        result[i].powi(2)
      }
    });
    result = std::array::from_fn(|i| {
      if i == i1 || i == i2 {
        result[i]
      } else {
        result[i].powi(2)
      }
    });

    let total: f32 = result.iter().copied().sum();
    if total == 0.0 {
      return scores.map(|_| power_budget / 4.0);
    }
    result.map(|x| x / total * power_budget)
  }
}
