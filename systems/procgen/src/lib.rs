use bevy::prelude::*;
use libnoise::{Blend, Fbm, Generator, Scale, Simplex, Source, Worley};
use serde::{Deserialize, Serialize};
use sys_move::IsoWorldCoords;

#[derive(Component)]
pub struct ProceduralLevel<const LAYERS: usize = 8> {
  pub seed: u64,
  // coords are divided by this value and floored
  pub tile_size: UVec2,
  pub noisegen: [NoiseGen; LAYERS],
}

type Simplex2D = Scale<2, Simplex<2>>;
type Worley2D = Scale<2, Worley<2>>;
type Blend2D<A, B, C> = Blend<2, A, B, C>;
type Fbm2D<A> = Fbm<2, A>;
type NoiseGen = Scale<2, Blend2D<Fbm2D<Simplex2D>, Worley2D, Worley2D>>;

impl<const LAYERS: usize> ProceduralLevel<LAYERS> {
  pub fn sample(&self, pos: &IsoWorldCoords, layer: usize) -> f32 {
    let coords = (**pos / self.tile_size.as_vec2()).floor();
    self.noisegen[layer].sample([coords.x as f64, coords.y as f64]) as f32 * 0.5 + 0.5
  }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NoiseGenSettings {
  pub overall_scale: [f64; 2],
  pub simplex_scale: [f64; 2],
  pub worley_seed_offset1: u64,
  pub worley_seed_offset2: u64,
  pub worley_scale1: [f64; 2],
  pub worley_scale2: [f64; 2],
  pub fbm_octaves: u32,
  pub fbm_freq: f64,
  pub fbm_lacunarity: f64,
  pub fbm_persistence: f64,
}

impl NoiseGenSettings {
  pub fn create_generator(&self, seed: u64) -> NoiseGen {
    Source::simplex(seed)
      .scale([self.simplex_scale[0], self.simplex_scale[1]])
      .fbm(
        self.fbm_octaves,
        self.fbm_freq,
        self.fbm_lacunarity,
        self.fbm_persistence,
      )
      .blend(
        Source::worley(seed + self.worley_seed_offset1)
          .scale([self.worley_scale1[0], self.worley_scale1[1]]),
        Source::worley(seed + self.worley_seed_offset2)
          .scale([self.worley_scale2[0], self.worley_scale2[1]]),
      )
      .scale([self.overall_scale[0], self.overall_scale[1]])
  }
}
