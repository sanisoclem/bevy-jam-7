use bevy::prelude::*;
use sys_move::IsoWorldCoords;

#[derive(Debug, Clone, Component, Reflect)]
pub enum HitTestableShape {
  Circle { radius: f32 },
  Aabb { half_extents: Vec2 },
  Obb { half_extents: Vec2, rotation: f32 },
  Polygon { vertices: Vec<Vec2> }, // must be convex
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

        gizmos.ellipse_2d(
          Isometry2d::from_translation(origin),
          Vec2::new(*radius, *radius * aspect_ratio),
          color,
        );
      }
      HitTestableShape::Aabb { half_extents } => {
        let top_right = location + IsoWorldCoords::new(half_extents.x, half_extents.y);
        let top_left = location + IsoWorldCoords::new(-half_extents.x, half_extents.y);
        let bot_left = location + IsoWorldCoords::new(-half_extents.x, -half_extents.y);
        let bot_right = location + IsoWorldCoords::new(half_extents.x, -half_extents.y);

        let tr = top_right.to_screen(aspect_ratio);
        let tl = top_left.to_screen(aspect_ratio);
        let bl = bot_left.to_screen(aspect_ratio);
        let br = bot_right.to_screen(aspect_ratio);

        gizmos.line_2d(tl, tr, color);
        gizmos.line_2d(tr, br, color);
        gizmos.line_2d(br, bl, color);
        gizmos.line_2d(bl, tl, color);
      }
      _ => {}
    };
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
        HitTestableShape::Aabb { half_extents: h1 },
        HitTestableShape::Aabb { half_extents: h2 },
      ) => aabb_aabb_overlap(self_pos, *h1, other_pos, *h2),
      (HitTestableShape::Circle { radius }, HitTestableShape::Aabb { half_extents }) => {
        circle_aabb_overlap(self_pos, *radius, other_pos, *half_extents)
      }
      (HitTestableShape::Aabb { half_extents }, HitTestableShape::Circle { radius }) => {
        circle_aabb_overlap(other_pos, *radius, self_pos, *half_extents)
      }
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
      (HitTestableShape::Polygon { vertices: v1 }, HitTestableShape::Polygon { vertices: v2 }) => {
        polygon_polygon_overlap(self_pos, v1, other_pos, v2)
      }
      // fallback - use bounding circles
      _ => {
        let r1 = self.bounding_radius();
        let r2 = other.bounding_radius();
        circle_circle_overlap(self_pos, r1, other_pos, r2)
      }
    }
  }

  pub fn bounding_radius(&self) -> f32 {
    match self {
      HitTestableShape::Circle { radius } => *radius,
      HitTestableShape::Aabb { half_extents } => half_extents.length(),
      HitTestableShape::Obb { half_extents, .. } => half_extents.length(),

      // very rough estimate (should roughly be centered on 0,0)
      HitTestableShape::Polygon { vertices } => vertices
        .iter()
        .map(|v| v.length())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0),
    }
  }
}

fn circle_circle_overlap(pos1: &IsoWorldCoords, r1: f32, pos2: &IsoWorldCoords, r2: f32) -> bool {
  pos1.distance_squared(*pos2) <= (r1 + r2).powi(2)
}

fn aabb_aabb_overlap(
  pos1: &IsoWorldCoords,
  half1: Vec2,
  pos2: &IsoWorldCoords,
  half2: Vec2,
) -> bool {
  let min1 = **pos1 - half1;
  let max1 = **pos1 + half1;
  let min2 = **pos2 - half2;
  let max2 = **pos2 + half2;

  min1.x <= max2.x && max1.x >= min2.x && min1.y <= max2.y && max1.y >= min2.y
}

fn circle_aabb_overlap(
  circle_pos: &IsoWorldCoords,
  radius: f32,
  aabb_pos: &IsoWorldCoords,
  half_extents: Vec2,
) -> bool {
  let circle_vec = **circle_pos;
  let aabb_vec = **aabb_pos;

  let closest = Vec2::new(
    circle_vec
      .x
      .clamp(aabb_vec.x - half_extents.x, aabb_vec.x + half_extents.x),
    circle_vec
      .y
      .clamp(aabb_vec.y - half_extents.y, aabb_vec.y + half_extents.y),
  );

  circle_vec.distance_squared(closest) <= radius.powi(2)
}

fn obb_obb_overlap(
  pos1: &IsoWorldCoords,
  half1: Vec2,
  rot1: f32,
  pos2: &IsoWorldCoords,
  half2: Vec2,
  rot2: f32,
) -> bool {
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

fn polygon_polygon_overlap(
  pos1: &IsoWorldCoords,
  verts1: &[Vec2],
  pos2: &IsoWorldCoords,
  verts2: &[Vec2],
) -> bool {
  for verts in [verts1, verts2] {
    for i in 0..verts.len() {
      let j = (i + 1) % verts.len();
      let edge = verts[j] - verts[i];
      let axis = Vec2::new(-edge.y, edge.x).normalize();

      if !polygon_overlap_on_axis(axis, pos1, verts1, pos2, verts2) {
        return false;
      }
    }
  }

  true
}

fn polygon_overlap_on_axis(
  axis: Vec2,
  pos1: &IsoWorldCoords,
  verts1: &[Vec2],
  pos2: &IsoWorldCoords,
  verts2: &[Vec2],
) -> bool {
  let (min1, max1) = project_polygon_onto_axis(axis, pos1, verts1);
  let (min2, max2) = project_polygon_onto_axis(axis, pos2, verts2);

  !(max1 < min2 || max2 < min1)
}

fn project_polygon_onto_axis(axis: Vec2, pos: &IsoWorldCoords, verts: &[Vec2]) -> (f32, f32) {
  let pos_vec = **pos;
  let projections: Vec<f32> = verts.iter().map(|v| (pos_vec + *v).dot(axis)).collect();

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
  fn test_aabb_aabb_overlap() {
    let origin = IsoWorldCoords::new(0.0, 0.0);
    let offset = IsoWorldCoords::new(3.0, 3.0);

    assert!(aabb_aabb_overlap(
      &origin,
      Vec2::splat(5.0),
      &offset,
      Vec2::splat(5.0),
    ));

    let edge = IsoWorldCoords::new(10.0, 0.0);
    assert!(aabb_aabb_overlap(
      &origin,
      Vec2::splat(5.0),
      &edge,
      Vec2::splat(5.0),
    ));

    let far = IsoWorldCoords::new(11.0, 0.0);
    assert!(!aabb_aabb_overlap(
      &origin,
      Vec2::splat(5.0),
      &far,
      Vec2::splat(5.0),
    ));
  }

  #[test]
  fn test_circle_aabb_overlap() {
    let origin = IsoWorldCoords::new(0.0, 0.0);

    assert!(circle_aabb_overlap(&origin, 3.0, &origin, Vec2::splat(5.0),));

    let edge = IsoWorldCoords::new(7.0, 0.0);
    assert!(circle_aabb_overlap(&edge, 3.0, &origin, Vec2::splat(5.0),));

    let corner = IsoWorldCoords::new(6.0, 6.0);
    assert!(circle_aabb_overlap(&corner, 2.0, &origin, Vec2::splat(5.0),));

    let far = IsoWorldCoords::new(10.0, 0.0);
    assert!(!circle_aabb_overlap(&far, 3.0, &origin, Vec2::splat(5.0),));
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
      0.0,
    ));

    assert!(obb_obb_overlap(
      &origin,
      Vec2::splat(5.0),
      0.0,
      &offset,
      Vec2::splat(5.0),
      FRAC_PI_4,
    ));

    let far = IsoWorldCoords::new(10.0, 0.0);
    assert!(!obb_obb_overlap(
      &origin,
      Vec2::splat(2.0),
      0.0,
      &far,
      Vec2::splat(2.0),
      FRAC_PI_4,
    ));
  }

  #[test]
  fn test_polygon_polygon_overlap() {
    let origin = IsoWorldCoords::new(0.0, 0.0);

    let triangle = vec![
      Vec2::new(0.0, 0.0),
      Vec2::new(5.0, 0.0),
      Vec2::new(2.5, 5.0),
    ];
    let square = vec![
      Vec2::new(-2.0, -2.0),
      Vec2::new(2.0, -2.0),
      Vec2::new(2.0, 2.0),
      Vec2::new(-2.0, 2.0),
    ];

    assert!(polygon_polygon_overlap(
      &origin, &triangle, &origin, &square
    ));

    let far = IsoWorldCoords::new(10.0, 10.0);
    assert!(!polygon_polygon_overlap(&origin, &triangle, &far, &square,));
  }

  #[test]
  fn test_hit_testable_shape() {
    let circle = HitTestableShape::Circle { radius: 5.0 };
    let aabb = HitTestableShape::Aabb {
      half_extents: Vec2::splat(5.0),
    };

    let origin = IsoWorldCoords::new(0.0, 0.0);
    let offset = IsoWorldCoords::new(3.0, 0.0);
    let offset2 = IsoWorldCoords::new(3.0, 3.0);

    assert!(circle.hit_test(&origin, &circle, &offset));

    assert!(circle.hit_test(&origin, &aabb, &offset2));

    assert!(aabb.hit_test(&origin, &aabb, &offset2));
  }

  #[test]
  fn test_bounding_radius() {
    assert_eq!(
      HitTestableShape::Circle { radius: 5.0 }.bounding_radius(),
      5.0
    );

    let aabb = HitTestableShape::Aabb {
      half_extents: Vec2::new(3.0, 4.0),
    };
    assert_eq!(aabb.bounding_radius(), 5.0); // 3-4-5 triangle

    let polygon = HitTestableShape::Polygon {
      vertices: vec![
        Vec2::new(5.0, 0.0),
        Vec2::new(1.0, 3.0),
        Vec2::new(0.0, 2.0),
      ],
    };
    assert_eq!(polygon.bounding_radius(), 5.0);
  }
}
