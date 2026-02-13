use bevy::{
  ecs::{lifecycle::HookContext, world::DeferredWorld},
  mesh::MeshVertexBufferLayoutRef,
  prelude::*,
  render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, RenderPipelineDescriptor,
    SpecializedMeshPipelineError,
  },
  shader::ShaderRef,
  sprite_render::{AlphaMode2d, Material2d, Material2dKey},
};

#[derive(Component)]
#[component(on_insert = on_add_fireball_explosion)]
pub struct FireballExplodeBody {
  pub lifetime: Timer,
  pub intensity: f32,
  pub radius: f32,
}

fn on_add_fireball_explosion(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
  let explosion = world.get::<FireballExplodeBody>(entity).unwrap();
  let radius = explosion.radius;
  let intensity = explosion.intensity;

  let mut meshes = world.resource_mut::<Assets<Mesh>>();
  let mesh = meshes.add(Circle::new(radius));

  let mut materials = world.resource_mut::<Assets<FireballExplosionMaterial>>();
  let material = materials.add(FireballExplosionMaterial {
    data: Vec4::new(fastrand::f32(), intensity, radius, 0.0),
  });

  world
    .commands()
    .entity(entity)
    .insert((Mesh2d(mesh), MeshMaterial2d(material)));
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct FireballExplosionMaterial {
  #[uniform(0)]
  pub data: Vec4,
}

impl Material2d for FireballExplosionMaterial {
  fn fragment_shader() -> ShaderRef {
    "shaders/fireball_explosion.wgsl".into()
  }

  fn alpha_mode(&self) -> AlphaMode2d {
    AlphaMode2d::Blend
  }

  fn specialize(
    descriptor: &mut RenderPipelineDescriptor,
    _layout: &MeshVertexBufferLayoutRef,
    _key: Material2dKey<Self>,
  ) -> Result<(), SpecializedMeshPipelineError> {
    if let Some(fragment) = &mut descriptor.fragment
      && let Some(Some(target)) = fragment.targets.get_mut(0)
    {
      target.blend = Some(BlendState {
        color: BlendComponent {
          src_factor: BlendFactor::SrcAlpha,
          dst_factor: BlendFactor::One,
          operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
          src_factor: BlendFactor::One,
          dst_factor: BlendFactor::One,
          operation: BlendOperation::Add,
        },
      });
    }

    Ok(())
  }
}
