use bevy::{platform::collections::HashMap, prelude::*};

#[derive(Resource, Default, Deref, DerefMut, Reflect)]
#[reflect(Resource, Default)]
pub struct IsoTilemapChunkMeshCache(HashMap<UVec2, Handle<Mesh>>);

pub fn get_chunk_mesh(
  chunk_size: u32,
  tile_size: UVec2,
  cache: &mut IsoTilemapChunkMeshCache,
  meshes: &mut Assets<Mesh>,
) -> Handle<Mesh> {
  let mesh_size = chunk_size * tile_size;

  if let Some(mesh) = cache.get(&mesh_size) {
    mesh.clone()
  } else {
    meshes.add(Rectangle::from_size(mesh_size.as_vec2()))
  }
}
