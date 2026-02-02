use bevy::{
  camera::visibility::NoFrustumCulling,
  color::palettes::css::PURPLE,
  platform::collections::{HashMap, HashSet},
  prelude::*,
};
pub use components::*;
pub use material::*;
pub use mesh::*;

mod components;
mod material;
mod mesh;

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

    let mut children = Vec::new();
    // TODO: can we use spawn_batch() to spawn children???
    for chunk in to_spawn {
      let center = chunk.center_world(generator.chunk_size, generator.tile_size);
      let e = cmd
        .spawn((
          LevelChunk {
            id: chunk,
            center,
            size: generator.chunk_size,
            tile_size: generator.tile_size,
          },
          Transform::default().with_translation((center).extend(-chunk.x().max(chunk.y()) as f32)),
        ))
        .id();
      children.push(e);
      info!("Spawning {:?}", chunk);
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
        let center = chunk_id.center_world(generator.chunk_size, generator.tile_size);
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

pub fn generate_level_chunk_mesh(
  mut cmd: Commands,
  mut cache: ResMut<IsoTilemapChunkMeshCache>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ChunkMaterial>>,
  qry: Query<(&LevelChunk, Entity), Without<Mesh2d>>,
) {
  for (chunk, entity) in qry {
    // TODO: generate create resource to track meshes and materials used by chunks and
    // unload them when no longer needed

    let mesh = mesh::get_chunk_mesh(chunk.size, chunk.tile_size, &mut cache, &mut meshes);
    cmd.entity(entity).insert((
      NoFrustumCulling,
      Mesh2d(mesh),
      // MeshMaterial2d(materials.add(Color::from(PURPLE))),
      MeshMaterial2d(
        materials.add(ChunkMaterial {
          id: IVec4::new(chunk.id.x(), chunk.id.y(), 0, 0),
          player_pos: Vec2::default()
            .extend(chunk.size as f32 * chunk.tile_size.x.max(chunk.tile_size.y) as f32)
            .extend(0.),
        }),
      ),
    ));
  }
}

pub fn update_chunk_spawner_pos(
  mut materials: ResMut<Assets<ChunkMaterial>>,
  qry: Query<&Transform, With<ChunkSpawner>>,
) {
  let Ok(player_transform) = qry.single() else {
    return;
  };

  for mat in materials.iter_mut() {
    mat.1.player_pos.x = player_transform.translation.x;
    mat.1.player_pos.y = player_transform.translation.y;
  }
}
