use bevy::prelude::*;
use libnoise::prelude::*;

use crate::level::chunk::ChunkId;

pub fn get_tile_index(chunk: ChunkId, chunk_size: u32, tile_local: UVec2) -> UVec2 {
  let generator = Source::simplex(42) // start with simplex noise
    .fbm(5, 0.013, 2.0, 0.5) // apply fractal brownian motion
    .blend(
      // apply blending...
      Source::worley(43).scale([0.05, 0.05]), // ...with scaled worley noise
      Source::worley(44).scale([0.02, 0.02]),
    );
  let coords = chunk
    .get_absolute_tile_coords(chunk_size, tile_local)
    .as_vec2();
  let multipler = 0.412401;
  let value = generator.sample([coords.x as f64 * multipler, coords.y as f64 * multipler]);
  let value2 = generator.sample([coords.x as f64 * 10., coords.y as f64 * 10.]);

  UVec2::new(
    sample(value2, &[(0, 0.5), (1, 0.1), (2, 0.1), (3, 0.1), (4, 0.1)]),
    sample(value, &[(10, 0.5), (0, 0.1), (1, 0.1), (2, 0.1), (3, 0.1)]),
  )
}

pub fn sample(value: f64, probabilities: &[(usize, f64)]) -> u32 {
  let mut threshold = -1.;
  for (idx, x) in probabilities.iter() {
    threshold += x * 2.;
    if value <= threshold {
      return *idx as u32;
    }
  }
  probabilities.last().unwrap().0 as u32
}
