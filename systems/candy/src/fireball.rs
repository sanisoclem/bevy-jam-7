use bevy::{
  ecs::{lifecycle::HookContext, world::DeferredWorld},
  prelude::*,
  render::render_resource::AsBindGroup,
  shader::ShaderRef,
  sprite_render::{AlphaMode2d, Material2d},
};

#[derive(Component)]
#[component(on_insert=on_add_fireball_body)]
pub struct FireballBody {
  pub intensity: f32,
  pub radius: f32,
}

fn on_add_fireball_body(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
  let fb = world.get::<FireballBody>(entity).unwrap();
  let radius = fb.radius;
  let intensity = fb.intensity;
  let mut meshes = world.resource_mut::<Assets<Mesh>>();
  let mesh = meshes.add(Circle::new(radius));
  let mut materials = world.resource_mut::<Assets<FireballMaterial>>();
  let material = materials.add(FireballMaterial {
    data: Vec4::new(0.0, 0.0, intensity, radius),
  });

  world
    .commands()
    .entity(entity)
    .insert((Mesh2d(mesh), MeshMaterial2d(material)));
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct FireballMaterial {
  #[uniform(0)]
  pub data: Vec4,
}

impl Material2d for FireballMaterial {
  fn fragment_shader() -> ShaderRef {
    "shaders/fireball.wgsl".into()
  }
  fn alpha_mode(&self) -> AlphaMode2d {
    AlphaMode2d::Blend
  }
}
