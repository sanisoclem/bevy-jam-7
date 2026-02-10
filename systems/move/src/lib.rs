use bevy::{color::palettes::css::GREEN, prelude::*, time::Stopwatch};

mod iso;

pub use iso::*;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct SysMovePlugin;

impl Plugin for SysMovePlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(
      Update,
      (add_moveable_state, update_moveable_state, update_transform),
    );

    #[cfg(feature = "dev")]
    app.add_systems(Update, draw_gizmos);
  }
}

#[derive(Debug, Component, Reflect)]
pub struct IsoMovementStage {
  pub aspect_ratio: f32,
  pub stopwatch: Stopwatch, // how long the level has been loaded
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct Placeable {
  pub location: IsoWorldCoords,
  pub layer: u8,
}
#[derive(Component, Debug, Clone, Reflect)]
pub struct Moveable {
  pub damping: f32,
  // pub mass: f32,
  pub net_forces: Vec2,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct MoveableVelocity {
  world_velocity: Vec2,
  screen_velocity: Vec2,
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect)]
pub struct MoveState {
  pub is_moving: bool,
  pub direction: MoveDirection,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, Reflect, Deserialize, Serialize)]
pub enum MoveDirection {
  North,
  Northeast,
  East,
  Southeast,
  #[default]
  South,
  Southwest,
  West,
  Northwest,
}

impl MoveDirection {
  pub fn all() -> impl Iterator<Item = Self> {
    // todo: convert to static arc slice
    [
      MoveDirection::North,
      MoveDirection::Northeast,
      MoveDirection::East,
      MoveDirection::Southeast,
      MoveDirection::South,
      MoveDirection::Southwest,
      MoveDirection::West,
      MoveDirection::Northwest,
    ]
    .into_iter()
  }
  pub fn from_velocity(velocity: Vec2) -> Self {
    // get angle (rad)
    let angle = velocity.y.atan2(velocity.x);

    // normalize to 0-2
    let normalized = if angle < 0.0 {
      angle + std::f32::consts::TAU
    } else {
      angle
    };

    // segment into 8 (45 degs each)
    // east is 0, going CCW
    let segment =
      ((normalized + std::f32::consts::FRAC_PI_8) / std::f32::consts::FRAC_PI_4) as usize % 8;

    match segment {
      0 => MoveDirection::East,
      1 => MoveDirection::Northeast,
      2 => MoveDirection::North,
      3 => MoveDirection::Northwest,
      4 => MoveDirection::West,
      5 => MoveDirection::Southwest,
      6 => MoveDirection::South,
      7 => MoveDirection::Southeast,
      _ => unreachable!(),
    }
  }
}

pub fn advance_stage_time(qry: Query<&mut IsoMovementStage>, time: Res<Time>) {
  for mut stage in qry {
    stage.stopwatch.tick(time.delta());
  }
}

pub fn add_moveable_state(
  mut cmd: Commands,
  qry: Query<Entity, (With<Moveable>, Without<MoveableVelocity>)>,
) {
  for entity in qry {
    let mut ecmd = cmd.get_entity(entity).expect("entity should exist");
    ecmd.insert((
      MoveableVelocity {
        world_velocity: Vec2::splat(0.),
        screen_velocity: Vec2::splat(0.),
      },
      MoveState {
        is_moving: false,
        direction: MoveDirection::North,
      },
    ));
  }
}
pub fn update_moveable_state(
  mut qry: Query<(
    &mut Placeable,
    &mut MoveableVelocity,
    &mut MoveState,
    &ChildOf,
    &Moveable,
  )>,
  qry_stage: Query<&IsoMovementStage>,
  time: Res<Time>,
) {
  for (mut p, mut v, mut state, co, m) in qry.iter_mut() {
    let Ok(stage) = qry_stage.get(co.parent()) else {
      continue;
    };
    let t = time.delta_secs();
    let decayed_velocity = v.world_velocity - (v.world_velocity * m.damping * t * 8.);
    let new_velocity = decayed_velocity + m.net_forces;
    let screenspace_velocity = IsoWorldCoords::from(new_velocity).to_screen(stage.aspect_ratio);
    let move_offset = new_velocity * t;

    v.world_velocity = new_velocity;
    v.screen_velocity = screenspace_velocity;
    p.location = p.location + move_offset.into();
    if new_velocity.length_squared() > 50.0 {
      state.direction = MoveDirection::from_velocity(screenspace_velocity);
      state.is_moving = true;
    } else {
      state.is_moving = false;
    }
  }
}

pub fn draw_gizmos(
  mut giz: Gizmos,
  mut qry: Query<(&Placeable, &MoveableVelocity, &Moveable)>,
  qry_stage: Query<(Entity, &IsoMovementStage)>,
  qry_children: Query<&Children>,
) {
  for (stage_entity, stage) in qry_stage {
    let Some(children) = qry_children.get(stage_entity).ok() else {
      continue;
    };
    for child in children {
      let Some((p, s, _m)) = qry.get_mut(*child).ok() else {
        continue;
      };

      let origin = p.location.to_screen(stage.aspect_ratio);
      let future_pos = IsoWorldCoords::from(s.world_velocity).to_screen(stage.aspect_ratio);
      // giz.ray_2d(origin, future_pos, Color::from(GREEN));
    }
  }
}

pub fn update_transform(
  mut qry: Query<(&Placeable, &mut Transform)>,
  qry_stage: Query<(Entity, &IsoMovementStage)>,
  qry_children: Query<&Children>,
) {
  for (stage_entity, stage) in qry_stage {
    for child in qry_children.iter_descendants(stage_entity) {
      let Some((p, mut t)) = qry.get_mut(child).ok() else {
        continue;
      };

      let screen_coords = p.location.to_screen(stage.aspect_ratio);
      // info!("updated coords {:?}", screen_coords);
      t.translation =
        screen_coords.extend(p.layer as f32 + (-(p.location.x + p.location.y) / 10000.));
    }
  }
}
