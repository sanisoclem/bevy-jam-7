use bevy::prelude::*;

mod iso;

pub use iso::*;

#[derive(Default)]
pub struct SysMovePlugin;

impl Plugin for SysMovePlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Update, (update_moveable_state, update_transform));
  }
}

#[derive(Debug, Component)]
pub struct IsoMovementStage {
  pub aspect_ratio: f32,
}

#[derive(Component, Debug, Clone)]
pub struct Placeable {
  pub location: IsoWorldCoords,
  pub layer: u8,
}
#[derive(Component, Debug, Clone)]
pub struct Moveable {
  pub damping: f32,
  pub mass: f32,
  pub net_forces: Vec2,
}

#[derive(Component, Debug, Clone)]
pub struct MoveableState {
  velocity: Vec2,
  direction: Direction,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Direction {
  North,
  Northeast,
  East,
  Southeast,
  South,
  Southwest,
  West,
  Northwest,
}

impl Direction {
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
      0 => Direction::East,
      1 => Direction::Northeast,
      2 => Direction::North,
      3 => Direction::Northwest,
      4 => Direction::West,
      5 => Direction::Southwest,
      6 => Direction::South,
      7 => Direction::Southeast,
      _ => unreachable!(),
    }
  }
}

pub fn update_moveable_state(
  mut qry: Query<(&mut Placeable, &mut MoveableState, &Moveable)>,
  time: Res<Time>,
) {
  for (mut p, mut s, m) in qry.iter_mut() {
    let t = time.delta_secs();
    let decayed_velocity = s.velocity - (s.velocity * m.damping * t);
    let acc = m.net_forces / m.mass;
    let new_velocity = decayed_velocity + acc * t;
    let move_offset = new_velocity * t;

    s.velocity = new_velocity;
    p.location = p.location + move_offset.into();
    if new_velocity.length_squared() > 0.0001 {
      s.direction = Direction::from_velocity(new_velocity);
    }
  }
}
pub fn update_transform(
  mut qry: Query<(&Placeable, &mut Transform)>,
  qry_stage: Query<(Entity, &IsoMovementStage)>,
  qry_children: Query<&Children>,
) {
  for (stage_entity, stage) in qry_stage {
    let Some(children) = qry_children.get(stage_entity).ok() else {
      continue;
    };
    for child in children {
      let Some((p, mut t)) = qry.get_mut(*child).ok() else {
        continue;
      };

      let screen_coords = p.location.to_screen(stage.aspect_ratio);
      t.translation =
        screen_coords.extend(p.layer as f32 + (-(p.location.x + p.location.y) / 10000.));
    }
  }
}
