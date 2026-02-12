use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use sys_move::IsoWorldCoords;

#[derive(Debug, Clone, Component, Reflect, Serialize, Deserialize)]
pub enum HitTestableShape {
  Circle { radius: f32 },
  Obb { half_extents: Vec2, rotation: f32 },
}

impl HitTestableShape {
  pub fn draw_gizmo(
    &self,
    gizmos: &mut Gizmos,
    location: IsoWorldCoords,
    aspect_ratio: f32,
    color: Color,
  ) {
    match self {
      HitTestableShape::Circle { radius } => {
        let origin = location.to_screen(aspect_ratio);
        // TODO: 0.7 is a magic number, who knows why it works
        gizmos.ellipse_2d(
          Isometry2d::from_translation(origin),
          Vec2::new(*radius * 0.7, *radius * aspect_ratio * 0.7),
          color,
        );
      }
      HitTestableShape::Obb {
        half_extents,
        rotation,
      } => {
        let rot_matrix = Mat2::from_angle(*rotation);
        let center = *location;

        let corners = [
          center + rot_matrix * Vec2::new(-half_extents.x, -half_extents.y),
          center + rot_matrix * Vec2::new(half_extents.x, -half_extents.y),
          center + rot_matrix * Vec2::new(half_extents.x, half_extents.y),
          center + rot_matrix * Vec2::new(-half_extents.x, half_extents.y),
        ];

        let screen_corners: Vec<Vec2> = corners
          .iter()
          .map(|c| IsoWorldCoords::from(*c).to_screen(aspect_ratio))
          .collect();

        for i in 0..4 {
          let j = (i + 1) % 4;
          gizmos.line_2d(screen_corners[i], screen_corners[j], color);
        }
      }
    }
  }

  pub fn hit_test(
    &self,
    self_pos: &IsoWorldCoords,
    other: &HitTestableShape,
    other_pos: &IsoWorldCoords,
  ) -> bool {
    match (self, other) {
      (HitTestableShape::Circle { radius: r1 }, HitTestableShape::Circle { radius: r2 }) => {
        circle_circle_overlap(self_pos, *r1, other_pos, *r2)
      }
      (
        HitTestableShape::Circle { radius },
        HitTestableShape::Obb {
          half_extents,
          rotation,
        },
      ) => circle_obb_overlap(self_pos, *radius, other_pos, *half_extents, *rotation),
      (
        HitTestableShape::Obb {
          half_extents,
          rotation,
        },
        HitTestableShape::Circle { radius },
      ) => circle_obb_overlap(other_pos, *radius, self_pos, *half_extents, *rotation),
      (
        HitTestableShape::Obb {
          half_extents: h1,
          rotation: r1,
        },
        HitTestableShape::Obb {
          half_extents: h2,
          rotation: r2,
        },
      ) => obb_obb_overlap(self_pos, *h1, *r1, other_pos, *h2, *r2),
    }
  }

  pub fn bounding_radius(&self) -> f32 {
    match self {
      HitTestableShape::Circle { radius } => *radius,
      HitTestableShape::Obb { half_extents, .. } => half_extents.length(),
    }
  }
}

fn circle_circle_overlap(pos1: &IsoWorldCoords, r1: f32, pos2: &IsoWorldCoords, r2: f32) -> bool {
  pos1.distance_squared(*pos2) <= (r1 + r2).powi(2)
}

fn circle_obb_overlap(
  circle_pos: &IsoWorldCoords,
  radius: f32,
  obb_pos: &IsoWorldCoords,
  half_extents: Vec2,
  rotation: f32,
) -> bool {
  // transform circle center into OBB's local space
  let circle_vec = **circle_pos;
  let obb_vec = **obb_pos;
  let delta = circle_vec - obb_vec;

  let rot_matrix = Mat2::from_angle(-rotation);
  let local_circle = rot_matrix * delta;

  // find closest point on AABB in local space
  let closest = Vec2::new(
    local_circle.x.clamp(-half_extents.x, half_extents.x),
    local_circle.y.clamp(-half_extents.y, half_extents.y),
  );

  local_circle.distance_squared(closest) <= radius.powi(2)
}

fn obb_obb_overlap(
  pos1: &IsoWorldCoords,
  half1: Vec2,
  rot1: f32,
  pos2: &IsoWorldCoords,
  half2: Vec2,
  rot2: f32,
) -> bool {
  // SAT - test all 4 axes (2 per OBB)
  let axes = [
    Vec2::from_angle(rot1),
    Vec2::from_angle(rot1 + std::f32::consts::FRAC_PI_2),
    Vec2::from_angle(rot2),
    Vec2::from_angle(rot2 + std::f32::consts::FRAC_PI_2),
  ];

  for axis in &axes {
    if !overlap_on_axis(*axis, pos1, half1, rot1, pos2, half2, rot2) {
      return false;
    }
  }

  true
}

fn overlap_on_axis(
  axis: Vec2,
  pos1: &IsoWorldCoords,
  half1: Vec2,
  rot1: f32,
  pos2: &IsoWorldCoords,
  half2: Vec2,
  rot2: f32,
) -> bool {
  let (min1, max1) = project_obb_onto_axis(axis, pos1, half1, rot1);
  let (min2, max2) = project_obb_onto_axis(axis, pos2, half2, rot2);

  !(max1 < min2 || max2 < min1)
}

fn project_obb_onto_axis(
  axis: Vec2,
  pos: &IsoWorldCoords,
  half_extents: Vec2,
  rotation: f32,
) -> (f32, f32) {
  let rot_matrix = Mat2::from_angle(rotation);
  let pos_vec = **pos;

  let corners = [
    pos_vec + rot_matrix * Vec2::new(-half_extents.x, -half_extents.y),
    pos_vec + rot_matrix * Vec2::new(half_extents.x, -half_extents.y),
    pos_vec + rot_matrix * Vec2::new(half_extents.x, half_extents.y),
    pos_vec + rot_matrix * Vec2::new(-half_extents.x, half_extents.y),
  ];

  let projections: Vec<f32> = corners.iter().map(|c| c.dot(axis)).collect();

  let min = projections.iter().copied().fold(f32::INFINITY, f32::min);
  let max = projections
    .iter()
    .copied()
    .fold(f32::NEG_INFINITY, f32::max);

  (min, max)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_circle_circle_overlap() {
    let origin = IsoWorldCoords::new(0.0, 0.0);
    let offset = IsoWorldCoords::new(3.0, 0.0);
    assert!(circle_circle_overlap(&origin, 5.0, &offset, 5.0));

    let far = IsoWorldCoords::new(10.0, 0.0);
    assert!(circle_circle_overlap(&origin, 5.0, &far, 5.0));

    let very_far = IsoWorldCoords::new(11.0, 0.0);
    assert!(!circle_circle_overlap(&origin, 5.0, &very_far, 5.0));
  }

  #[test]
  fn test_circle_obb_overlap() {
    let origin = IsoWorldCoords::new(0.0, 0.0);

    // circle overlaps centered OBB
    assert!(circle_obb_overlap(
      &origin,
      3.0,
      &origin,
      Vec2::splat(5.0),
      0.0
    ));

    // circle touches edge of OBB
    let edge = IsoWorldCoords::new(7.0, 0.0);
    assert!(circle_obb_overlap(
      &edge,
      3.0,
      &origin,
      Vec2::splat(5.0),
      0.0
    ));

    // circle far from OBB
    let far = IsoWorldCoords::new(10.0, 0.0);
    assert!(!circle_obb_overlap(
      &far,
      3.0,
      &origin,
      Vec2::splat(5.0),
      0.0
    ));

    // rotated OBB
    use std::f32::consts::FRAC_PI_4;
    assert!(circle_obb_overlap(
      &origin,
      3.0,
      &origin,
      Vec2::splat(5.0),
      FRAC_PI_4
    ));
  }

  #[test]
  fn test_obb_obb_overlap() {
    use std::f32::consts::FRAC_PI_4;

    let origin = IsoWorldCoords::new(0.0, 0.0);
    let offset = IsoWorldCoords::new(3.0, 3.0);

    assert!(obb_obb_overlap(
      &origin,
      Vec2::splat(5.0),
      0.0,
      &offset,
      Vec2::splat(5.0),
      0.0
    ));
    assert!(obb_obb_overlap(
      &origin,
      Vec2::splat(5.0),
      0.0,
      &offset,
      Vec2::splat(5.0),
      FRAC_PI_4
    ));

    let far = IsoWorldCoords::new(10.0, 0.0);
    assert!(!obb_obb_overlap(
      &origin,
      Vec2::splat(2.0),
      0.0,
      &far,
      Vec2::splat(2.0),
      FRAC_PI_4
    ));
  }

  #[test]
  fn test_hit_testable_shape() {
    let circle = HitTestableShape::Circle { radius: 5.0 };
    let obb = HitTestableShape::Obb {
      half_extents: Vec2::splat(5.0),
      rotation: 0.0,
    };

    let origin = IsoWorldCoords::new(0.0, 0.0);
    let offset = IsoWorldCoords::new(3.0, 0.0);

    assert!(circle.hit_test(&origin, &circle, &offset));
    assert!(circle.hit_test(&origin, &obb, &offset));
    assert!(obb.hit_test(&origin, &obb, &offset));
  }

  #[test]
  fn test_bounding_radius() {
    assert_eq!(
      HitTestableShape::Circle { radius: 5.0 }.bounding_radius(),
      5.0
    );

    let obb = HitTestableShape::Obb {
      half_extents: Vec2::new(3.0, 4.0),
      rotation: 0.0,
    };
    assert_eq!(obb.bounding_radius(), 5.0); // 3-4-5 triangle
  }
}
