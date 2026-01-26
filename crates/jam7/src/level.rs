use bevy::{platform::collections::HashMap, prelude::*};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(
      Update,
      (spawn_chunks, despawn_chunks, process_level_commands),
    );
  }
}

#[derive(Debug, Message)]
pub enum LevelCommands {
  StartLevel(LevelId, LevelDescriptor),
  UnloadLevel(LevelId),
}

#[derive(Debug, Clone)]
pub struct LevelDescriptor {
  chunk_size: f32,
  seed: i64,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct LevelId(i32);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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
    let center_x = (self.0 as f32 + 0.5) * chunk_size;
    let center_y = (self.1 as f32 + 0.5) * chunk_size;
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

#[derive(Component)]
pub struct ProceduralLevel;

#[derive(Debug, Component)]
pub struct LevelChunk {
  id: ChunkId,
}

#[derive(Component, Debug)]
pub struct ChunkSpawner {
  level: LevelId,
  load_radius: u32,
  unload_radius: u32,
}

#[derive(Resource, Default)]
pub struct LevelTracker {
  levels: HashMap<LevelId, LevelController>,
}

pub struct LevelController {
  descriptor: LevelDescriptor,
  root: Entity,
  loaded_chunks: HashMap<ChunkId, Entity>,
}
impl LevelController {
  pub fn load_chunk(&mut self, chunk: ChunkId, entity: Entity) {
    self.loaded_chunks.insert(chunk, entity);
  }
  pub fn unload_chunk(&mut self, chunk: &ChunkId) {
    self.loaded_chunks.remove(chunk);
  }
}

pub fn process_level_commands(
  mut cmd: Commands,
  mut reader: MessageReader<LevelCommands>,
  mut tracker: ResMut<LevelTracker>,
) {
  for command in reader.read() {
    match command {
      LevelCommands::StartLevel(level_id, descriptor) => {
        if tracker.levels.contains_key(level_id) {
          continue;
        }
        let root = cmd.spawn(ProceduralLevel).id();
        tracker.levels.insert(
          *level_id,
          LevelController {
            descriptor: descriptor.clone(),
            root,
            loaded_chunks: HashMap::new(),
          },
        );
      }
      LevelCommands::UnloadLevel(level_id) => {
        let Some(existing) = tracker.levels.remove(level_id) else {
          continue;
        };
        cmd.entity(existing.root).despawn();
      }
    }
  }
}

pub fn spawn_chunks(
  mut cmd: Commands,
  mut tracker: ResMut<LevelTracker>,
  qry_spawner: Query<(&ChunkSpawner, &Transform)>,
) {
  for (spawner, spawner_transform) in qry_spawner {
    let Some(controller) = tracker.levels.get_mut(&spawner.level) else {
      continue;
    };
    let to_spawn: Vec<_> = ChunkId::get_chunks_to_be_loaded(
      spawner_transform.translation.xy(),
      controller.descriptor.chunk_size,
      spawner.load_radius,
    )
    .into_iter()
    .filter(|chunk_id| !controller.loaded_chunks.contains_key(chunk_id))
    .collect();

    let mut children = Vec::new();

    // TODO: can we use spawn_batch() to spawn children???
    for chunk in to_spawn {
      let e = cmd.spawn(LevelChunk { id: chunk }).id();
      children.push(e);
      controller.load_chunk(chunk, e);
    }

    cmd.entity(controller.root).add_children(&children);
  }
}

pub fn despawn_chunks(
  mut cmd: Commands,
  mut tracker: ResMut<LevelTracker>,
  qry_spawner: Query<(&ChunkSpawner, &Transform)>,
) {
  let spawner_coords: Vec<_> = qry_spawner
    .iter()
    .map(|(spawner, spawner_transform)| {
      (
        spawner.level,
        spawner.unload_radius,
        spawner_transform.translation.xy(),
      )
    })
    .collect();
  for (level_id, controller) in &mut tracker.levels {
    let to_check: Vec<_> = spawner_coords
      .iter()
      .cloned()
      .filter(|(id, _, _)| id == level_id)
      .map(|(_, radius, xy)| (radius as f32 * controller.descriptor.chunk_size, xy))
      .collect();
    // gets all loaded chunks that are outside unload_radius of all spawners for that level
    let to_despawn: Vec<_> = controller
      .loaded_chunks
      .iter()
      .map(|(x, y)| (*x, *y))
      .filter(|(chunk_id, _entity)| {
        let center = chunk_id.center_world(controller.descriptor.chunk_size);
        to_check
          .iter()
          .all(|(dist_squared, xy)| center.distance_squared(*xy) >= *dist_squared)
      })
      .collect();
    for (chunk_id, e) in to_despawn {
      cmd.entity(e).despawn();
      controller.unload_chunk(&chunk_id);
    }
  }
}
