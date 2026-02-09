use bevy::{color::palettes::css::RED, prelude::*, sprite::Anchor};
use jam7::{level::LevelCommand, player::Player};
use sys_combat::{Combatant, CombatantGuages, DamageTaken, DeathBehavior, HitTestableShape};
use sys_magic::{
  EquippedSpell, EquippedSpellState, SpellBook, SpellBookState, SpellGenerator, SpellTrigger,
};
use sys_move::{IsoMovementStage, IsoWorldCoords, Moveable, Placeable};

pub struct AlphaGymPlugin;

impl Plugin for AlphaGymPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, setup).add_systems(
      Update,
      (
        spawn_enemy,
        spawn_damage_text,
        update_damage_text,
        spawn_death_text,
      ),
    );
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

#[derive(Component)]
pub struct DeathText;

pub fn setup(mut level_cmd: MessageWriter<LevelCommand>) {
  level_cmd.write(LevelCommand::StartLevel("alpha".to_owned()));
}

pub fn spawn_enemy(
  mut spawned: Local<Option<Entity>>,
  mut cmd: Commands,
  qry_stage: Query<(Entity, &IsoMovementStage)>,
) {
  let Some((stage_entity, _stage)) = qry_stage.iter().next() else {
    return;
  };

  let location = IsoWorldCoords::new(100., 100.);

  if let Some(e) = *spawned {
    return;
  }

  let e = cmd
    .spawn((
      Transform::default().with_scale(Vec3::splat(0.1)),
      Visibility::default(),
      Combatant {
        max_hp: 100,
        hitbox: HitTestableShape::Circle { radius: 7.0 },
        team: 1,
        regen: 0,
        regen_delay: 0,
        death_behavior: DeathBehavior::Respawn(
          Timer::from_seconds(5.0, TimerMode::Once),
          Timer::from_seconds(2.0, TimerMode::Once),
        ),
      },
      Anchor(Vec2::new(0., -0.3)),
      SpellBook {
        spells: vec![EquippedSpell {
          generator: SpellGenerator::Fireball {
            radius: 3.,
            base_damage: 3,
            lifetime: 12.0,
            speed: 20.,
          },
          cooldown: Timer::from_seconds(0.3, TimerMode::Repeating),
          trigger: SpellTrigger::Auto,
        }],
      },
      SpellBookState {
        spells_states: vec![EquippedSpellState::default()],
      },
      Placeable { layer: 5, location },
      EnemyPlaceholder {
        fire_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
      },
    ))
    .id();
  *spawned = Some(e);
  cmd.entity(stage_entity).add_child(e);
}

fn spawn_death_text(
  mut dead: Local<Option<Entity>>,
  asset_server: Res<AssetServer>,
  mut cmd: Commands,
  stage: Query<&IsoMovementStage>,
  mut qry: Query<(&Placeable, &Combatant, &CombatantGuages), With<Player>>,
  mut qry_text: Query<&mut Text2d>,
  time: Res<Time>,
) {
  let Some(stage) = stage.iter().next() else {
    return;
  };
  let Some((p, cd, mut g)) = qry.iter_mut().next() else {
    return;
  };

  if g.death_timer.is_some() && dead.is_none() {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_font = TextFont {
      font: font.clone(),
      font_size: 12.0,
      ..default()
    };
    let screen_pos = p.location.to_screen(stage.aspect_ratio);
    *dead = Some(
      cmd
        .spawn((
          Text2d::new("You died, respawning in 5 seconds"),
          text_font.clone(),
          TextColor(Color::from(RED).with_alpha(1.0)),
          TextLayout::new_with_justify(Justify::Center),
          Transform::from_translation((screen_pos).extend(100.0)),
          DeathText,
        ))
        .id(),
    );
    return;
  }

  if let Some(text_entity) = dead.as_ref() {
    let mut txt = qry_text.get_mut(*text_entity).unwrap();
    let Some(dt) = &g.death_timer else {
      cmd.entity(*text_entity).despawn();
      *dead = None;
      return;
    };

    txt.0 = format!(
      "You died, respawning in {} seconds",
      dt.remaining_secs().floor()
    );
  }
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
      (rand::random::<f32>() - 0.5) * 20.0,
      (rand::random::<f32>() - 0.5) * 10.0,
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
