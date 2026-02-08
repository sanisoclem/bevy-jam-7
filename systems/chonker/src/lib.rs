use bevy::{
  platform::collections::{HashMap, HashSet},
  prelude::*,
};
use sys_move::{IsoWorldCoords, Placeable};

pub struct SysChonkerPlugin;

impl Plugin for SysChonkerPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Update, (spawn_chunks, despawn_chunks));
  }
}

#[derive(Debug, Clone, Component)]
pub struct ChunkGenerator {
  pub chunk_size_world: Vec2,
  pub load_radius: u32,
  pub unload_radius: u32,
  pub load_around: Entity,
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
  qry_chunk: Query<&LevelChunk>,
  qry_chunk_children: Query<&Children>,
  qry_placeable: Query<&Placeable>,
  qry_generator: Query<(Entity, &ChunkGenerator)>,
) {
  for (generator_entity, generator) in qry_generator {
    let loaded_chunks: HashSet<_> = qry_chunk_children
      .get(generator_entity)
      .ok()
      .into_iter()
      .flat_map(|children| children.iter())
      .filter_map(|child| qry_chunk.get(child).ok().map(|c| c.id))
      .collect();

    let Ok(origin) = qry_placeable.get(generator.load_around) else {
      warn!("Unable to find load around entity, skipping loading chunks");
      continue;
    };

    let to_spawn: Vec<_> = ChunkId::get_chunks_to_be_loaded(
      origin.location,
      generator.chunk_size_world,
      generator.load_radius,
    )
    .into_iter()
    .filter(|chunk_id| !loaded_chunks.contains(chunk_id))
    .collect();

    if to_spawn.is_empty() {
      continue;
    }

    let children: Vec<_> = to_spawn
      .into_iter()
      .map(|chunk| {
        cmd
          .spawn((
            LevelChunk { id: chunk },
            Placeable {
              location: chunk.origin_world(generator.chunk_size_world),
              layer: 0,
            },
            Transform::default(),
            Visibility::default(),
          ))
          .id()
      })
      .collect();

    info!("spawning chunks {:?}", children);

    cmd.entity(generator_entity).add_children(&children);
  }
}

pub fn despawn_chunks(
  mut cmd: Commands,
  qry_generator: Query<(Entity, &ChunkGenerator)>,
  qry_chunk: Query<&LevelChunk>,
  qry_placeable: Query<&Placeable>,
  qry_chunk_children: Query<&Children>,
) {
  for (entity_root, generator) in qry_generator {
    let loaded_chunks: HashMap<_, _> = qry_chunk_children
      .get(entity_root)
      .ok()
      .into_iter()
      .flat_map(|children| children.iter())
      .filter_map(|child| qry_chunk.get(child).ok().map(|c| (c.id, child)))
      .collect();

    let Ok(origin) = qry_placeable.get(generator.load_around) else {
      warn!("Unable to find load around entity, skipping loading chunks");
      continue;
    };

    let to_despawn: Vec<_> = loaded_chunks
      .iter()
      .map(|(x, y)| (*x, *y))
      .filter(|(chunk_id, _entity)| {
        chunk_id.should_despawn(
          origin.location,
          generator.chunk_size_world,
          generator.unload_radius,
        )
      })
      .collect();

    for (x, e) in to_despawn {
      info!("Despawning {:?}", x);
      cmd.entity(e).despawn();
    }
  }
}
