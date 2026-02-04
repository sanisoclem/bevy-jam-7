use bevy::prelude::*;
use libnoise::prelude::*;

use crate::level::chunk::ChunkId;

pub fn get_tile_index(chunk: ChunkId, tile_local: UVec2) -> UVec2 {
  let generator = Source::simplex(42) // start with simplex noise
    .fbm(5, 0.013, 2.0, 0.5) // apply fractal brownian motion
    .blend(
      // apply blending...
      Source::worley(43).scale([0.05, 0.05]), // ...with scaled worley noise
      Source::worley(44).scale([0.02, 0.02]),
    );
  let coords = chunk.get_absolute_tile_coords(tile_local).as_vec2();
  let multipler = 0.412401;
  let value = generator.sample([coords.x as f64 * multipler, coords.y as f64 * multipler]);
  let value2 = generator.sample([coords.x as f64 * 10., coords.y as f64 * 10.]);
  let (y, x) = if value <= 0. {
    (10, 0)
  } else if value <= 0.25 {
    (0, if value2 <= 0.1 { 0 } else { 1 })
  } else if value <= 0.5 {
    (1, if value2 <= 0.1 { 0 } else { 1 })
  } else if value <= 0.75 {
    (2, if value2 <= 0.1 { 0 } else { 1 })
  } else {
    (3, if value2 <= 0.1 { 0 } else { 1 })
  };
  UVec2::new(x, y)
}
