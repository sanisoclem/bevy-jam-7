use bevy::{
  platform::collections::{HashMap, HashSet},
  prelude::*,
};
use utils::iso::IsoWorldCoords;

#[derive(Debug, Clone, Component)]
pub struct ChunkGenerator {
  pub level_id: u32,
  pub chunk_size_world: Vec2,
  pub chunk_size_screen: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub struct ChunkId(i32, i32);

impl ChunkId {
  pub fn x(&self) -> i32 {
    self.0
  }
  pub fn y(&self) -> i32 {
    self.1
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
  pub fn origin_world(&self, chunk_size_world: Vec2) -> IsoWorldCoords {
    (self.as_vec2() * chunk_size_world).into()
  }
  pub fn from_world(coords: IsoWorldCoords, chunk_size_world: Vec2) -> ChunkId {
    let x = (coords.x / chunk_size_world.x).floor() as i32;
    let y = (coords.y / chunk_size_world.y).floor() as i32;
    ChunkId(x, y)
  }
  pub fn origin_screen(&self, chunk_size_world: Vec2, chunk_size_screen: Vec2) -> Vec2 {
    self
      .origin_world(chunk_size_world)
      .to_screen(chunk_size_screen.y / chunk_size_screen.x)
  }
  pub fn from_screen_pos(screen: Vec2, chunk_size_world: Vec2, chunk_size_screen: Vec2) -> ChunkId {
    Self::from_world(
      IsoWorldCoords::from_screen(screen, chunk_size_screen.y / chunk_size_screen.x),
      chunk_size_world,
    )
  }
  pub fn should_despawn(
    &self,
    origin: IsoWorldCoords,
    chunk_size_world: Vec2,
    load_radius: u32,
  ) -> bool {
    let radius_squared = (load_radius as f32 * chunk_size_world.x.max(chunk_size_world.y)).powi(2);
    let dist_squared = self.origin_world(chunk_size_world).distance_squared(origin);
    dist_squared > radius_squared
  }
  pub fn get_chunks_to_be_loaded(
    origin: IsoWorldCoords,
    chunk_size_world: Vec2,
    load_radius: u32,
  ) -> Vec<ChunkId> {
    let origin_chunk = ChunkId::from_world(origin, chunk_size_world);
    let radius = load_radius as i32;
    let radius_squared = (load_radius as f32 * chunk_size_world.x.max(chunk_size_world.y)).powi(2);

    (-radius..=radius)
      .flat_map(|dx| {
        (-radius..=radius).filter_map(move |dy| {
          let new_chunk = origin_chunk.offset(dx, dy);
          let dist_squared = new_chunk
            .origin_world(chunk_size_world)
            .distance_squared(origin);
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
}

#[derive(Component, Debug)]
pub struct ChunkSpawner {
  pub owner_id: u32,
  pub load_radius: u32,
  pub unload_radius: u32,
}

pub fn spawn_chunks(
  mut cmd: Commands,
  qry_generator: Query<(Entity, &ChunkGenerator)>,
  qry_chunk: Query<&LevelChunk>,
  qry_chunk_children: Query<&Children>,
  qry_spawner: Query<(&ChunkSpawner, &Transform)>,
) {
  for (spawner, spawner_transform) in qry_spawner {
    let Some((generator_entity, generator)) = qry_generator
      .iter()
      .find(|(_, x)| x.level_id == spawner.owner_id)
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

    // TODO: get world coords from spawner instead of screen coords
    // after this has ben done, chunk module doesn't need to know screen sizes anymore
    let origin = IsoWorldCoords::from_screen(
      spawner_transform.translation.xy(),
      generator.chunk_size_screen.y / generator.chunk_size_screen.x,
    );

    let to_spawn: Vec<_> =
      ChunkId::get_chunks_to_be_loaded(origin, generator.chunk_size_world, spawner.load_radius)
        .into_iter()
        .filter(|chunk_id| !loaded_chunks.contains(chunk_id))
        .collect();

    if to_spawn.is_empty() {
      continue;
    }

    let children: Vec<_> = to_spawn
      .into_iter()
      .map(|chunk| {
        let coords = Vec3::new(0., 0., -(chunk.0 as f32 + chunk.1 as f32) / 10000.);
        cmd
          .spawn((
            LevelChunk { id: chunk },
            Transform::default().with_translation(coords),
            Visibility::default(),
          ))
          .id()
      })
      .collect();

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
    let spawners: Vec<_> = spawner_coords
      .iter()
      .cloned()
      .filter(|(id, _, _)| *id == generator.level_id)
      .map(|(_, radius, xy)| {
        (
          IsoWorldCoords::from_screen(
            xy,
            generator.chunk_size_screen.y / generator.chunk_size_screen.x,
          ),
          radius,
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

    // despawn if too far from ALL spawners
    let to_despawn: Vec<_> = loaded_chunks
      .iter()
      .map(|(x, y)| (*x, *y))
      .filter(|(chunk_id, _entity)| {
        spawners.iter().all(|(spawner_coords, unload_radius)| {
          chunk_id.should_despawn(*spawner_coords, generator.chunk_size_world, *unload_radius)
        })
      })
      .collect();

    for (x, e) in to_despawn {
      info!("Despawning {:?}", x);
      cmd.entity(e).despawn();
    }
  }
}
