pub mod assets;

pub mod iso {
  use bevy::prelude::*;
  pub fn world_to_screen(world: Vec2, chunk_size: Vec2) -> Vec2 {
    let rev = world / chunk_size;
    (rev + Vec2::new(-rev.y, rev.x)) * (chunk_size / 2.)
  }

  //   pub fn screen_to_world(screen: Vec2, chunk_size: Vec2) -> Vec2 {
  //     // map.x = (screen.x / TILE_WIDTH_HALF + screen.y / TILE_HEIGHT_HALF) /2;
  //     // map.y = (screen.y / TILE_HEIGHT_HALF -(screen.x / TILE_WIDTH_HALF)) /2;
  //     // map.x = screen.x / TILE_WIDTH + screen.y / TILE_HEIGHT;
  //     // map.y = screen.y / TILE_HEIGHT - screen.x / TILE_WIDTH;
  //
  // //        (screen / chunk_size) + Vec2::new(screen.y / chunk_size.y, screen.x / chunk_size.x)
  //         screen / (chunk_size / 2.)
  //   }

  pub fn screen_to_world(screen: Vec2, chunk_size: Vec2) -> Vec2 {
    let scaled = screen * 2. / chunk_size;
    Vec2::new((scaled.x + scaled.y) / 2., (scaled.y - scaled.x) / 2.) * chunk_size
  }
}
