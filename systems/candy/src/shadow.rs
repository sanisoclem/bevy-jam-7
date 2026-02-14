use bevy::{
  ecs::{lifecycle::HookContext, world::DeferredWorld},
  prelude::*,
  render::render_resource::AsBindGroup,
  shader::ShaderRef,
  sprite_render::{AlphaMode2d, Material2d},
};

#[derive(Component)]
#[component(on_insert=on_add_shadow)]
pub struct Shadow {
  pub radius: f32,
}

fn on_add_shadow(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
  let shadow = world.get::<Shadow>(entity).unwrap();
  let radius = shadow.radius;
  let mut meshes = world.resource_mut::<Assets<Mesh>>();
  let mesh = meshes.add(Circle::new(radius));
  let mut materials = world.resource_mut::<Assets<ShadowMaterial>>();
  let material = materials.add(ShadowMaterial {
    data: Vec4::new(0.0, 0.0, 0.0, radius),
  });

  world
    .commands()
    .entity(entity)
    .insert((Mesh2d(mesh), MeshMaterial2d(material)));
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct ShadowMaterial {
  #[uniform(0)]
  pub data: Vec4,
}

impl Material2d for ShadowMaterial {
  fn fragment_shader() -> ShaderRef {
    "shaders/shadow.wgsl".into()
  }
  fn alpha_mode(&self) -> AlphaMode2d {
    AlphaMode2d::Blend
  }
}

pub fn sync_shadow_materials(
  qry: Query<(&Transform, &MeshMaterial2d<ShadowMaterial>, &Shadow)>,
  mut materials: ResMut<Assets<ShadowMaterial>>,
) {
  for (t, material_handle, s) in &qry {
    if let Some(material) = materials.get_mut(&material_handle.0) {
      material.data.x = t.translation.x;
      material.data.y = t.translation.y;
      material.data.w = s.radius;
    }
  }
}
