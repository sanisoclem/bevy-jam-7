use bevy::{
  platform::collections::{HashMap, HashSet},
  prelude::*,
};

#[derive(Debug, Clone, Component)]
pub struct ChunkGenerator {
  pub chunk_size: f32,
  pub seed: i64,
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
  pub const fn offset(&self, x: i32, y: i32) -> ChunkId {
    ChunkId(self.0 + x, self.1 + y)
  }
  pub fn center_world(&self, chunk_size: f32) -> Vec2 {
    let center_x = (self.0 as f32) * chunk_size;
    let center_y = (self.1 as f32) * chunk_size;
    Vec2::new(center_x, center_y)
  }
  pub fn from_world_pos(world: Vec2, chunk_size: f32) -> ChunkId {
    let x = (world.x / chunk_size).floor() as i32;
    let y = (world.y / chunk_size).floor() as i32;
    ChunkId(x, y)
  }
  pub fn get_chunks_to_be_loaded(origin: Vec2, chunk_size: f32, load_radius: u32) -> Vec<ChunkId> {
    let origin_chunk = ChunkId::from_world_pos(origin, chunk_size);
    let radius = load_radius as i32;
    let radius_squared = (load_radius as f32 * chunk_size).powi(2);

    (-radius..=radius)
      .flat_map(|dx| {
        (-radius..=radius).filter_map(move |dy| {
          let new_chunk = origin_chunk.offset(dx, dy);
          let dist_squared = new_chunk.center_world(chunk_size).distance_squared(origin);
          (dist_squared <= radius_squared).then_some(new_chunk)
        })
      })
      .collect()
  }
}

#[derive(Clone, Component, Reflect)]
#[reflect(Component, Clone)]
pub struct LevelChunk {
  pub id: ChunkId,
  pub size: f32,
  pub center: Vec2,
}

#[derive(Component, Debug)]
pub struct ChunkSpawner {
  pub generator: Entity,
  pub load_radius: u32,
  pub unload_radius: u32,
}

pub fn spawn_chunks(
  mut cmd: Commands,
  qry_generator: Query<&ChunkGenerator>,
  qry_chunk: Query<&LevelChunk>,
  qry_chunk_children: Query<&Children>,
  qry_spawner: Query<(&ChunkSpawner, &Transform)>,
) {
  for (spawner, spawner_transform) in qry_spawner {
    let Ok(generator) = qry_generator.get(spawner.generator) else {
      continue;
    };
    let loaded_chunks: HashSet<_> = qry_chunk_children
      .get(spawner.generator)
      .ok()
      .into_iter()
      .flat_map(|children| children.iter())
      .filter_map(|child| qry_chunk.get(child).ok().map(|c| c.id))
      .collect();

    let to_spawn: Vec<_> = ChunkId::get_chunks_to_be_loaded(
      spawner_transform.translation.xy(),
      generator.chunk_size,
      spawner.load_radius,
    )
    .into_iter()
    .filter(|chunk_id| !loaded_chunks.contains(chunk_id))
    .collect();

    if to_spawn.is_empty() {
      continue;
    }

    let mut children = Vec::new();
    // TODO: can we use spawn_batch() to spawn children???
    for chunk in to_spawn {
      let center = chunk.center_world(generator.chunk_size);
      let e = cmd
        .spawn((
          LevelChunk {
            id: chunk,
            center,
            size: generator.chunk_size,
          },
          Transform::default().with_translation(center.extend(0.0)),
        ))
        .id();
      children.push(e);
      // controller.load_chunk(chunk, e);
    }

    cmd.entity(spawner.generator).add_children(&children);
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
        spawner.generator,
        spawner.unload_radius,
        spawner_transform.translation.xy(),
      )
    })
    .collect();
  for (entity_root, generator) in qry_generator.iter() {
    let to_check: Vec<_> = spawner_coords
      .iter()
      .cloned()
      .filter(|(id, _, _)| *id == entity_root)
      .map(|(_, radius, xy)| ((radius as f32 * generator.chunk_size).powi(2), xy))
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
        let center = chunk_id.center_world(generator.chunk_size);
        to_check
          .iter()
          .all(|(dist_squared, xy)| center.distance_squared(*xy) >= *dist_squared)
      })
      .collect();

    for (_, e) in to_despawn {
      cmd.entity(e).despawn();
    }
  }
}
