use bevy::{
  ecs::world,
  platform::collections::{HashMap, HashSet},
  prelude::*,
};

use crate::level::procgen;

#[derive(Debug, Clone, Component)]
pub struct ChunkGenerator {
  pub owner_id: u32,
  pub chunk_size: u32,
  pub tile_size: UVec2,
  pub seed: i64,
  pub tileset: Handle<Image>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Reflect)]
pub struct ChunkId(i32, i32);

impl ChunkId {
  pub fn x(&self) -> i32 {
    self.0
  }
  pub fn y(&self) -> i32 {
    self.1
  }
  pub fn get_absolute_tile_coords(&self, chunk_size: u32, tile_local: UVec2) -> IVec2 {
    chunk_size as i32 * self.as_ivec2() + tile_local.as_ivec2()
  }
  pub fn as_vec2(&self) -> Vec2 {
    Vec2::new(self.0 as f32, self.1 as f32)
  }
  pub fn as_ivec2(&self) -> IVec2 {
    IVec2::new(self.0, self.1)
  }
  pub const fn offset(&self, x: i32, y: i32) -> ChunkId {
    ChunkId(self.0 + x, self.1 + y)
  }
  pub fn origin(&self, chunk_size: u32, tile_size: UVec2) -> Vec2 {
    let effective_chunk_size = chunk_size as f32 * tile_size.as_vec2();
    let origin = self.as_vec2() * effective_chunk_size;
    utils::iso::world_to_screen(origin, effective_chunk_size)
  }
  pub fn from_screen_pos(screen: Vec2, effective_chunk_size: Vec2) -> ChunkId {
    let world = utils::iso::screen_to_world(screen, effective_chunk_size);
    let x = (world.x / effective_chunk_size.x).floor() as i32;
    let y = (world.y / effective_chunk_size.y).floor() as i32;
    ChunkId(x, y)
  }
  pub fn get_chunks_to_be_loaded(
    origin: Vec2,
    chunk_size: u32,
    tile_size: UVec2,
    load_radius: u32,
  ) -> Vec<ChunkId> {
    let effective_chunk_size = chunk_size as f32 * tile_size.as_vec2();
    let origin_chunk = ChunkId::from_screen_pos(origin, effective_chunk_size);
    let radius = load_radius as i32;
    // // let radius_squared =
    //   (load_radius as f32 * effective_chunk_size.x.max(effective_chunk_size.y)).powi(2);

    (-radius..=radius)
      .flat_map(|dx| {
        (-radius..=radius).filter_map(move |dy| {
          let new_chunk = origin_chunk.offset(dx, dy);
          let dist_squared = new_chunk
            .origin(chunk_size, tile_size)
            .distance_squared(origin);
          // (dist_squared <= radius_squared).then_some(new_chunk)
          Some(new_chunk)
        })
      })
      .collect()
  }
}

#[derive(Clone, Component, Reflect)]
#[reflect(Component, Clone)]
pub struct LevelChunk {
  pub id: ChunkId,
  pub size: u32,
  pub tile_size: UVec2,
  pub origin: Vec2,
  pub tileset: Handle<Image>,
}

#[derive(Component, Debug)]
pub struct ChunkSpawner {
  pub owner_id: u32,
  pub load_radius: u32,
  pub unload_radius: u32,
}

pub fn spawn_chunks(
  mut cmd: Commands,
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
  qry_generator: Query<(Entity, &ChunkGenerator)>,
  qry_chunk: Query<&LevelChunk>,
  qry_chunk_children: Query<&Children>,
  qry_spawner: Query<(&ChunkSpawner, &Transform)>,
) {
  for (spawner, spawner_transform) in qry_spawner {
    let Some((generator_entity, generator)) = qry_generator
      .iter()
      .find(|(_, x)| x.owner_id == spawner.owner_id)
    else {
      continue;
    };
    let loaded_chunks: HashSet<_> = qry_chunk_children
      .get(generator_entity)
      .ok()
      .into_iter()
      .flat_map(|children| children.iter())
      .filter_map(|child| qry_chunk.get(child).ok().map(|c| c.id))
      .collect();

    let to_spawn: Vec<_> = ChunkId::get_chunks_to_be_loaded(
      spawner_transform.translation.xy(),
      generator.chunk_size,
      generator.tile_size,
      spawner.load_radius,
    )
    .into_iter()
    .filter(|chunk_id| !loaded_chunks.contains(chunk_id))
    .collect();

    if to_spawn.is_empty() {
      continue;
    }

    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 11, 11, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let mut children = Vec::new();
    // TODO: can we use spawn_batch() to spawn children???
    for chunk in to_spawn {
      let origin = chunk.origin(generator.chunk_size, generator.tile_size);
      let coords = origin.extend(-(chunk.0 as f32 + chunk.1 as f32) / 10000.);
      let e = cmd
        .spawn((
          LevelChunk {
            id: chunk,
            origin,
            size: generator.chunk_size,
            tile_size: generator.tile_size,
            tileset: generator.tileset.clone(),
          },
          Transform::default().with_translation(coords),
          InheritedVisibility::VISIBLE,
        ))
        .with_children(|c| {
          for x in 0..generator.chunk_size as i32 {
            for y in 0..generator.chunk_size as i32 {
              let world_coords = Vec2::new(x as f32, y as f32) * generator.tile_size.as_vec2();
              let index = procgen::get_tile_index(
                chunk,
                generator.chunk_size,
                UVec2::new(x as u32, y as u32),
              );
              c.spawn((
                Transform::default().with_translation(
                  (utils::iso::world_to_screen(world_coords, generator.tile_size.as_vec2())
                    + Vec2::new(
                      0.,
                      -(generator.chunk_size as f32 * generator.tile_size.y as f32 * 0.5),
                    ))
                  .extend(
                    (-((chunk.0 * generator.chunk_size as i32
                      + chunk.1 * generator.chunk_size as i32
                      + x
                      + y) as f32))
                      / 10000.,
                  ),
                ),
                Sprite::from_atlas_image(
                  generator.tileset.clone(),
                  TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: (index.y * 11 + index.x) as usize,
                  },
                ),
              ));
            }
          }
        })
        .id();
      children.push(e);
      info!(
        "Spawning {:?} in {:?} ({:?},{:?})",
        chunk, coords, generator.chunk_size, generator.tile_size
      );
      // controller.load_chunk(chunk, e);
    }

    cmd.entity(generator_entity).add_children(&children);
  }
}

pub fn despawn_chunks(
  mut cmd: Commands,
  qry_generator: Query<(Entity, &ChunkGenerator)>,
  qry_chunk: Query<&LevelChunk>,
  qry_chunk_children: Query<&Children>,
  qry_spawner: Query<(&ChunkSpawner, &Transform)>,
) {
  let spawner_coords: Vec<_> = qry_spawner
    .iter()
    .map(|(spawner, spawner_transform)| {
      (
        spawner.owner_id,
        spawner.unload_radius,
        spawner_transform.translation.xy(),
      )
    })
    .collect();
  for (entity_root, generator) in qry_generator.iter() {
    let to_check: Vec<_> = spawner_coords
      .iter()
      .cloned()
      .filter(|(id, _, _)| *id == generator.owner_id)
      .map(|(_, radius, xy)| {
        (
          (radius as f32
            * generator.chunk_size as f32
            * generator.tile_size.x.max(generator.tile_size.y) as f32)
            .powi(2),
          xy,
        )
      })
      .collect();

    let loaded_chunks: HashMap<_, _> = qry_chunk_children
      .get(entity_root)
      .ok()
      .into_iter()
      .flat_map(|children| children.iter())
      .filter_map(|child| qry_chunk.get(child).ok().map(|c| (c.id, child)))
      .collect();

    let to_despawn: Vec<_> = loaded_chunks
      .iter()
      .map(|(x, y)| (*x, *y))
      .filter(|(chunk_id, _entity)| {
        let center = chunk_id.origin(generator.chunk_size, generator.tile_size);
        to_check
          .iter()
          .all(|(dist_squared, xy)| center.distance_squared(*xy) >= *dist_squared)
      })
      .collect();

    for (x, e) in to_despawn {
      info!("Despawning {:?}", x);
      cmd.entity(e).despawn();
    }
  }
}

pub fn update_chunk_spawner_pos(
  // mut materials: ResMut<Assets<ChunkMaterial>>,
  qry: Query<&Transform, With<ChunkSpawner>>,
) {
  let Ok(player_transform) = qry.single() else {
    return;
  };

  // for mat in materials.iter_mut() {
  //   // mat.1.player_pos.x = player_transform.translation.x;
  //   // mat.1.player_pos.y = player_transform.translation.y;
  // }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  pub fn test_chunk_id_origin() {
    let chunk = ChunkId(532, 41);
    let chunk_size = 13;
    let tile_size = UVec2::new(31, 33);
    assert_eq!(
      ChunkId::from_screen_pos(
        chunk.origin(chunk_size, tile_size),
        chunk_size as f32 * tile_size.as_vec2()
      ),
      chunk
    );
  }
}
