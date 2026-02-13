pub mod assets;
pub mod easing;

// should contain all the variables required to tune difficulty
pub mod diff {
  use bevy::prelude::*;

  pub const MAX_PROJECTILE_TRAVEL: f32 = 5000.;
  pub const MAX_PROJECTILE_LIFETIME: f32 = 2.;
  pub const TEAM_PLAYER: u8 = 1;
  pub const TEAM_ENEMY: u8 = 0;
  pub const TEAM_OTHER: u8 = 7;
  pub fn get_power_budget_from_kills(kills: f32) -> f32 {
    ((kills.max(1.0).log10() - 1.0) * 4.0).max(1.0)
  }
  pub fn get_mobility_from_rangeness(power_budget: f32, rangeness_score: f32) -> f32 {
    const MIN_MOBILITY: f32 = 20.0;
    const MIN_MOBILITY_AT_BUDGET_RATIO: f32 = 0.8;

    let capped_budget = power_budget.min(8.0);
    let max_mobility = 100.0 + (capped_budget - 1.0) * (100.0 / 7.0); // 100 at budget=1, 200 at budget=8

    let rangeness_ratio = rangeness_score / power_budget;
    if rangeness_ratio >= MIN_MOBILITY_AT_BUDGET_RATIO {
      MIN_MOBILITY
    } else {
      let t = rangeness_ratio / MIN_MOBILITY_AT_BUDGET_RATIO;
      max_mobility * (1.0 - t) + MIN_MOBILITY * t
    }
  }
  pub fn get_max_hp_from_toughness_score(toughness_score: f32) -> u32 {
    100 + (toughness_score * 500.).floor() as u32
  }
  pub fn get_effective_range_from_rangeness_score(rangeness_score: f32) -> f32 {
    100. + rangeness_score * 200.
  }
  pub fn get_effective_dps_from_offense_score(offense_score: f32) -> f32 {
    10. + offense_score * 50.
  }
  pub fn get_density_ceiling_from_score(density_score: f32) -> f32 {
    (density_score / 10000.).clamp(0.000001, 0.001)
  }
  pub fn get_enemy_size_from_toughness(toughness_score: f32) -> f32 {
    1.0 + toughness_score * 0.1
  }
  pub fn get_enemy_tint(toughness_score: f32, rangeness_score: f32, offense_score: f32) -> Color {
    let max = toughness_score.max(rangeness_score).max(offense_score);

    let r = toughness_score / max;
    let g = rangeness_score / max;
    let b = offense_score / max;

    let retval = Color::srgba(b, g, r, 1.0);
    debug!(
      "tinting {}, {}, {} = {:?}",
      toughness_score, rangeness_score, offense_score, retval
    );
    retval
  }
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

  pub fn get_max_projectile_lifetime(speed: f32) -> f32 {
    MAX_PROJECTILE_LIFETIME.min(MAX_PROJECTILE_TRAVEL / speed)
  }
  const LUCIDITY_GAIN_EXPONENT: f32 = 1.0;

  pub fn get_lucidity_gain(current_lucidity: u32, kills: u32) -> u32 {
    // more lucidity, more kills needed for the next point
    let kills_per_lucidity = (current_lucidity as f32 + 1.0).powf(1.0 / LUCIDITY_GAIN_EXPONENT);
    100 + (kills as f32 / kills_per_lucidity).floor() as u32
  }
}
pub mod colors {
  use bevy::color::Color;

  const TEAM_COLOR_PALETTE: &[Color] = &[
    Color::srgb(1.0, 0.3, 0.3), // red
    Color::srgb(0.3, 1.0, 0.3), // green
    Color::srgb(0.3, 0.3, 1.0), // blue
    Color::srgb(1.0, 1.0, 0.3), // yellow
    Color::srgb(1.0, 0.3, 1.0), // magenta
    Color::srgb(0.3, 1.0, 1.0), // cyan
  ];

  pub fn color_from_team(team: u8) -> Color {
    let index = (team as usize) % TEAM_COLOR_PALETTE.len();
    TEAM_COLOR_PALETTE[index]
  }
}
pub mod vecstuff {
  use bevy::prelude::*;

  pub fn subdivide_circle(count: usize) -> Vec<Vec2> {
    if count == 0 {
      return Vec::new();
    }

    let angle_step = std::f32::consts::TAU / count as f32;

    (0..count)
      .map(|i| {
        let angle = i as f32 * angle_step;
        Vec2::from_angle(angle)
      })
      .collect()
  }

  #[cfg(test)]
  mod tests {
    use super::*;

    #[test]
    fn test_subdivide_circle() {
      let dirs = subdivide_circle(2);
      assert_eq!(dirs.len(), 2);
      assert!((dirs[0] - Vec2::Y).length() < 0.001);
      assert!((dirs[1] - -Vec2::Y).length() < 0.001);

      let dirs = subdivide_circle(4);
      assert_eq!(dirs.len(), 4);
      assert!((dirs[0] - Vec2::Y).length() < 0.001);
      assert!((dirs[1] - -Vec2::X).length() < 0.001);
      assert!((dirs[2] - -Vec2::Y).length() < 0.001);
      assert!((dirs[3] - Vec2::X).length() < 0.001);

      let dirs = subdivide_circle(8);
      assert_eq!(dirs.len(), 8);
      for dir in &dirs {
        assert!((dir.length() - 1.0).abs() < 0.001);
      }

      let dirs = subdivide_circle(0);
      assert_eq!(dirs.len(), 0);
    }
  }
}
