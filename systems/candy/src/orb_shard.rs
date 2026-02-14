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
#[component(on_insert=on_add_orb_shard)]
pub struct FrozenOrbShard {
  pub radius: f32,
  pub intensity: f32,
  pub direction: f32,
  pub team: u8,
}

fn on_add_orb_shard(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
  let fb = world.get::<FrozenOrbShard>(entity).unwrap();
  let team = fb.team as f32;
  let radius = fb.radius;
  let intensity = fb.intensity;
  let direction = fb.direction;
  let mut meshes = world.resource_mut::<Assets<Mesh>>();
  let mesh = meshes.add(Circle::new(radius));
  let mut materials = world.resource_mut::<Assets<FrozenOrbShardMaterial>>();
  let material = materials.add(FrozenOrbShardMaterial {
    data: Vec4::new(fastrand::f32(), intensity, team, direction),
  });

  world
    .commands()
    .entity(entity)
    .insert((Mesh2d(mesh), MeshMaterial2d(material)));
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct FrozenOrbShardMaterial {
  #[uniform(0)]
  pub data: Vec4,
}

impl Material2d for FrozenOrbShardMaterial {
  fn fragment_shader() -> ShaderRef {
    "shaders/orb_shard.wgsl".into()
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
