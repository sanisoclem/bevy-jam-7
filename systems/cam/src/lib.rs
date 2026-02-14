use bevy::{
  core_pipeline::{
    FullscreenShader,
    core_2d::graph::{Core2d, Node2d},
    tonemapping::{DebandDither, Tonemapping},
  },
  ecs::query::QueryItem,
  post_process::bloom::Bloom,
  prelude::*,
  render::{
    RenderApp, RenderStartup,
    extract_component::{
      ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
      UniformComponentPlugin,
    },
    render_graph::{
      NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
    },
    render_resource::{
      binding_types::{sampler, texture_2d, uniform_buffer},
      *,
    },
    renderer::{RenderContext, RenderDevice},
    view::ViewTarget,
  },
};
use bevy_enhanced_input::prelude::*;

const SHADER_ASSET_PATH: &str = "shaders/post_processing.wgsl";
pub struct SysCamPlugin;

impl Plugin for SysCamPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_input_context::<PlayerCamera>()
      .add_systems(Startup, setup_camera)
      .add_systems(FixedUpdate, update_camera)
      .add_systems(Update, update_settings)
      .add_observer(apply_game_camera_zoom)
      .add_observer(on_aberrate);

    app.add_plugins((
      ExtractComponentPlugin::<PostProcessSettings>::default(),
      UniformComponentPlugin::<PostProcessSettings>::default(),
    ));

    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
      return;
    };

    render_app.add_systems(RenderStartup, init_post_process_pipeline);

    render_app
      .add_render_graph_node::<ViewNodeRunner<PostProcessNode>>(Core2d, PostProcessLabel)
      .add_render_graph_edges(
        Core2d,
        (
          Node2d::Tonemapping,
          PostProcessLabel,
          Node2d::EndMainPassPostProcessing,
        ),
      );
  }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct PostProcessLabel;

#[derive(Default)]
struct PostProcessNode;

impl ViewNode for PostProcessNode {
  type ViewQuery = (
    &'static ViewTarget,
    &'static PostProcessSettings,
    &'static DynamicUniformIndex<PostProcessSettings>,
  );

  fn run(
    &self,
    _graph: &mut RenderGraphContext,
    render_context: &mut RenderContext,
    (view_target, _post_process_settings, settings_index): QueryItem<Self::ViewQuery>,
    world: &World,
  ) -> Result<(), NodeRunError> {
    let post_process_pipeline = world.resource::<PostProcessPipeline>();
    let pipeline_cache = world.resource::<PipelineCache>();
    let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id)
    else {
      return Ok(());
    };

    let settings_uniforms = world.resource::<ComponentUniforms<PostProcessSettings>>();
    let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
      return Ok(());
    };

    let post_process = view_target.post_process_write();
    let bind_group = render_context.render_device().create_bind_group(
      "post_process_bind_group",
      &pipeline_cache.get_bind_group_layout(&post_process_pipeline.layout),
      &BindGroupEntries::sequential((
        post_process.source,
        &post_process_pipeline.sampler,
        settings_binding.clone(),
      )),
    );

    let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
      label: Some("post_process_pass"),
      color_attachments: &[Some(RenderPassColorAttachment {
        view: post_process.destination,
        depth_slice: None,
        resolve_target: None,
        ops: Operations::default(),
      })],
      depth_stencil_attachment: None,
      timestamp_writes: None,
      occlusion_query_set: None,
    });

    render_pass.set_render_pipeline(pipeline);
    render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
    render_pass.draw(0..3, 0..1);

    Ok(())
  }
}

#[derive(Resource)]
struct PostProcessPipeline {
  layout: BindGroupLayoutDescriptor,
  sampler: Sampler,
  pipeline_id: CachedRenderPipelineId,
}

fn init_post_process_pipeline(
  mut commands: Commands,
  render_device: Res<RenderDevice>,
  asset_server: Res<AssetServer>,
  fullscreen_shader: Res<FullscreenShader>,
  pipeline_cache: Res<PipelineCache>,
) {
  let layout = BindGroupLayoutDescriptor::new(
    "post_process_bind_group_layout",
    &BindGroupLayoutEntries::sequential(
      ShaderStages::FRAGMENT,
      (
        texture_2d(TextureSampleType::Float { filterable: true }),
        sampler(SamplerBindingType::Filtering),
        uniform_buffer::<PostProcessSettings>(true),
      ),
    ),
  );
  let sampler = render_device.create_sampler(&SamplerDescriptor::default());
  let shader = asset_server.load(SHADER_ASSET_PATH);
  let vertex_state = fullscreen_shader.to_vertex_state();
  let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
    label: Some("post_process_pipeline".into()),
    layout: vec![layout.clone()],
    vertex: vertex_state,
    fragment: Some(FragmentState {
      shader,
      targets: vec![Some(ColorTargetState {
        format: ViewTarget::TEXTURE_FORMAT_HDR,
        blend: None,
        write_mask: ColorWrites::ALL,
      })],
      ..default()
    }),
    ..default()
  });
  commands.insert_resource(PostProcessPipeline {
    layout,
    sampler,
    pipeline_id,
  });
}

#[derive(Component, Default, Clone, Copy, ExtractComponent, ShaderType)]
struct PostProcessSettings {
  intensity: f32,
  #[cfg(feature = "webgl2")]
  _webgl2_padding: Vec3,
}

#[derive(Event, Clone, Debug)]
pub struct Aberrate {
  pub amount: f32,
}

#[derive(InputAction)]
#[action_output(f32)]
pub struct ZoomCamera;

#[derive(Component, Debug)]
pub struct PlayerCamera;

#[derive(Component, Debug)]
pub struct CameraTarget;

fn setup_camera(mut cmd: Commands) {
  cmd.spawn((
    Camera2d,
    Camera {
      clear_color: ClearColorConfig::Custom(Color::BLACK),
      ..default()
    },
    Transform::default(),
    Tonemapping::TonyMcMapface,
    Bloom::default(),
    DebandDither::Enabled,
    PlayerCamera,
    PostProcessSettings { intensity: 0.0 },
    actions!(
      PlayerCamera[(
        Action::<ZoomCamera>::new(),
        DeadZone::default(),
        Bindings::spawn((
          Spawn((Binding::mouse_wheel(), SwizzleAxis::YXZ)),
          Bidirectional::new(GamepadButton::DPadUp, GamepadButton::DPadDown),
        )),
      )]
    ),
  ));
}

fn apply_game_camera_zoom(
  movement: On<Fire<ZoomCamera>>,
  mut cameras: Query<&mut Transform, With<PlayerCamera>>,
) {
  let mut transform = cameras.get_mut(movement.context).unwrap();
  transform.scale *= 1. + movement.value * 0.1;
  if transform.scale.x <= 0.0 {
    transform.scale = Vec3::splat(0.5);
  }
}

const CAMERA_DECAY_RATE: f32 = 2.;
fn update_camera(
  mut camera: Single<&mut Transform, (With<Camera2d>, Without<CameraTarget>)>,
  target: Single<&Transform, (With<CameraTarget>, Without<Camera2d>)>,
  time: Res<Time>,
) {
  let Vec3 { x, y, .. } = target.translation;
  let direction = Vec3::new(x, y, camera.translation.z);

  camera
    .translation
    .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());
}

fn update_settings(settings: Query<&mut PostProcessSettings>, time: Res<Time>) {
  const DECAY_RATE: f32 = 0.5;
  for mut setting in settings {
    let decay = setting.intensity * DECAY_RATE * time.delta_secs();
    setting.intensity -= decay;
  }
}

fn on_aberrate(evt: On<Aberrate>, qry: Query<&mut PostProcessSettings>) {
  for mut setting in qry {
    setting.intensity += evt.amount;
  }
}
