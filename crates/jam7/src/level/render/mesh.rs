use bevy::{
  asset::RenderAssetUsages,
  mesh::{Indices, PrimitiveTopology},
  platform::collections::HashMap,
  prelude::*,
};
use sys_move::IsoWorldCoords;

#[derive(Resource, Default, Deref, DerefMut, Reflect)]
#[reflect(Resource, Default)]
pub struct IsoTilemapChunkMeshCache(HashMap<u32, Handle<Mesh>>);

pub fn get_chunk_mesh(
  chunk_size_world: Vec2,
  tile_size_screen: Vec2,
  cache: &mut IsoTilemapChunkMeshCache,
  meshes: &mut Assets<Mesh>,
) -> Handle<Mesh> {
  let hardcoded_mesh_id = 0u32;

  if let Some(mesh) = cache.get(&hardcoded_mesh_id) {
    mesh.clone()
  } else {
    let ar = tile_size_screen.y / tile_size_screen.x;
    let csw_iso: IsoWorldCoords = chunk_size_world.into();
    let p1 = Vec3::splat(0.0);
    let p2 = csw_iso.with_y(0.).to_screen(ar).extend(0.);
    let p3 = csw_iso.to_screen(ar).extend(0.);
    let p4 = csw_iso.with_x(0.).to_screen(ar).extend(0.);

    let mut mesh = Mesh::new(
      PrimitiveTopology::TriangleList,
      RenderAssetUsages::RENDER_WORLD,
    );

    let v_pos = vec![p1.to_array(), p2.to_array(), p3.to_array(), p4.to_array()];
    let uvs: Vec<[f32; 2]> = vec![[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];
    let indices = vec![0, 3, 2, 0, 2, 1];

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    meshes.add(mesh)
  }
}
