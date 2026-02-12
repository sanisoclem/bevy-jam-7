use bevy::{color::palettes::css::RED, prelude::*};
use jam7::level::LevelCommand;
use sys_combat::DamageTaken;
use sys_move::{IsoMovementStage, Placeable};

pub struct AlphaGymPlugin;

impl Plugin for AlphaGymPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Startup, setup)
      .add_systems(Update, (spawn_damage_text, update_damage_text));
  }
}
#[derive(Component, Reflect)]
pub struct EnemyPlaceholder {
  pub fire_timer: Timer,
}

#[derive(Component)]
pub struct DamageText {
  pub timer: Timer,
  pub velocity: Vec2,
}

pub fn setup(mut level_cmd: MessageWriter<LevelCommand>) {
  level_cmd.write(LevelCommand::StartLevel("alpha".to_owned()));
}

fn spawn_damage_text(
  asset_server: Res<AssetServer>,
  mut commands: Commands,
  stage: Query<&IsoMovementStage>,
  mut qry: Query<&Placeable>,
  mut msg_reader: MessageReader<DamageTaken>,
) {
  let Some(stage) = stage.iter().next() else {
    return;
  };
  for msg in msg_reader.read() {
    let Ok(p) = qry.get_mut(msg.target) else {
      continue;
    };
    let screen_pos = p.location.to_screen(stage.aspect_ratio);

    let offset = Vec2::new(
      (fastrand::f32() - 0.5) * 20.0,
      (fastrand::f32() - 0.5) * 10.0,
    );

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_font = TextFont {
      font: font.clone(),
      font_size: 12.0,
      ..default()
    };
    commands.spawn((
      Text2d::new(format!("{:?}", msg.amount)),
      text_font.clone(),
      TextColor(Color::from(RED).with_alpha(1.0)),
      TextLayout::new_with_justify(Justify::Center),
      Transform::from_translation((screen_pos + offset).extend(100.0)),
      DamageText {
        timer: Timer::from_seconds(1.0, TimerMode::Once),
        velocity: Vec2::new(0.0, 50.0), // Float upward
      },
    ));
  }
}

pub fn update_damage_text(
  mut commands: Commands,
  mut query: Query<(Entity, &mut Transform, &mut TextColor, &mut DamageText)>,
  time: Res<Time>,
) {
  for (entity, mut transform, mut color, mut damage_text) in &mut query {
    damage_text.timer.tick(time.delta());

    // Move the text
    transform.translation.x += damage_text.velocity.x * time.delta_secs();
    transform.translation.y += damage_text.velocity.y * time.delta_secs();

    // Fade out over time
    let progress = damage_text.timer.fraction();
    let alpha = 1.0 - progress;

    color.0 = color.0.with_alpha(alpha);

    let scale = if progress < 0.2 {
      1.0 + (progress / 0.2) * 0.3 // Scale from 1.0 to 1.3 in first 20%
    } else {
      1.3
    };
    transform.scale = Vec3::splat(scale);

    // Despawn when finished
    if damage_text.timer.just_finished() {
      commands.entity(entity).despawn();
    }
  }
}
