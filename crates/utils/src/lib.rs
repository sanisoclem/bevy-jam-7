pub mod assets;
pub mod easing;

// should contain all the variables required to tune difficulty
pub mod diff {
  use bevy::prelude::*;

  pub const MAX_PROJECTILE_TRAVEL: f32 = 15000.;
  pub const MAX_PROJECTILE_LIFETIME: f32 = 5.;
  pub const TEAM_PLAYER: u8 = 1;
  pub const TEAM_ENEMY: u8 = 0;
  pub const TEAM_OTHER: u8 = 7;
  pub fn get_power_budget_from_kills(kills: f32) -> f32 {
    ((kills.max(1.0).log10() - 1.0) * 4.0).max(1.0)
  }
  pub fn get_mobility_from_rangeness(power_budget: f32, rangeness_score: f32) -> f32 {
    const MIN_MOBILITY: f32 = 60.0;
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
    20 + (toughness_score * 300.).floor() as u32
  }
  pub fn get_effective_range_from_rangeness_score(rangeness_score: f32) -> f32 {
    300. + rangeness_score * 600.
  }
  pub fn get_effective_dps_from_offense_score(offense_score: f32) -> f32 {
    10. + offense_score * 50.
  }
  pub fn get_density_ceiling_from_score(density_score: f32) -> f32 {
    (density_score / 1000000.).clamp(0.0000001, 0.0001)
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
}
pub mod colors {
  use bevy::color::Color;

  const TEAM_COLOR_PALETTE: &[Color] = &[
    Color::srgb(0.0, 0.0, 1.0),
    Color::srgb(1.0, 0.3, 0.3),
    Color::srgb(0.3, 0.3, 1.0),
    Color::srgb(1.0, 1.0, 0.3),
    Color::srgb(1.0, 0.3, 1.0),
    Color::srgb(0.3, 1.0, 1.0),
  ];

  pub fn color_from_team(team: u8) -> Color {
    let index = (team as usize) % TEAM_COLOR_PALETTE.len();
    TEAM_COLOR_PALETTE[index]
  }

  const LEVEL_UP_BASE_KILLS: f32 = 1.0;
  const LEVEL_UP_EXPONENT: f32 = 1.0;

  pub fn kills_required_for_level(current_level: u32) -> u32 {
    let kills = LEVEL_UP_BASE_KILLS * (current_level as f32).powf(LEVEL_UP_EXPONENT);
    kills.ceil() as u32
  }

  pub fn get_kills_needed_for_next(current_level: u32, total_kills: u32) -> u32 {
    let kills_for_current: u32 = (1..=current_level).map(kills_required_for_level).sum();
    // fml
    let kills_since_level = total_kills.saturating_sub(kills_for_current);
    let kills_needed = kills_required_for_level(current_level + 1);
    kills_needed.saturating_sub(kills_since_level)
  }
}
pub mod vecstuff {
  use bevy::prelude::*;
  use std::f32::consts::{PI, TAU};

  pub fn subdivide_circle(north: Vec2, count: usize) -> Vec<Vec2> {
    if count == 0 {
      return Vec::new();
    }

    let north_angle = north.y.atan2(north.x);
    let angle_step = TAU / count as f32;

    (0..count)
      .map(|i| {
        // special case for 2 shards
        let angle = if count == 2 {
          north_angle + (PI / 2.0) + i as f32 * angle_step
        } else {
          north_angle + PI + i as f32 * angle_step
        };
        Vec2::from_angle(angle)
      })
      .collect()
  }
}
