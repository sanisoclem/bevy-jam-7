use std::f32::consts::PI;

use bevy::{prelude::*, text::FontFeatures};
use sys_move::{IsoMovementStage, Placeable};
use utils::colors::color_from_team;

use crate::DamageTaken;

#[derive(Component)]
pub struct DamageText {
  pub timer: Timer,
  pub velocity: Vec2,
  pub scale_curve: EasingCurve<f32>,
}

pub fn spawn_damage_text(
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

    let velocity = Vec2::from_angle(fastrand::f32() * PI * 0.5 + 0.25) * 50.;
    commands.spawn((
      Text2d::new(format!("{:?}", msg.amount)),
      text_font.clone(),
      TextColor(color_from_team(msg.team)),
      TextLayout::new_with_justify(Justify::Center),
      Transform::from_translation((screen_pos + offset).extend(100.0)),
      DamageText {
        timer: Timer::from_seconds(1.0, TimerMode::Once),
        velocity,
        scale_curve: EasingCurve::new(1.0, 0.001, EaseFunction::Linear),
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

    transform.translation.x += damage_text.velocity.x * time.delta_secs();
    transform.translation.y += damage_text.velocity.y * time.delta_secs();

    let remaining = damage_text.timer.remaining_secs();

    const FADE_TIME: f32 = 0.3;
    if remaining <= FADE_TIME {
      let alpha = remaining / FADE_TIME;
      color.0 = color.0.with_alpha(alpha);
    }

    const SCALE_TIME: f32 = 0.2;
    if remaining <= SCALE_TIME
      && let Some(scale) = damage_text.scale_curve.sample(remaining / SCALE_TIME)
    {
      transform.scale = Vec3::splat(scale);
    }

    if damage_text.timer.just_finished() {
      commands.entity(entity).despawn();
    }
  }
}
