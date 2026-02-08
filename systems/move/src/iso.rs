use bevy::prelude::*;
use std::ops::Deref;

#[derive(Debug, Default, PartialEq, Clone, Copy, Reflect)]
pub struct IsoWorldCoords(Vec2);
impl Deref for IsoWorldCoords {
  type Target = Vec2;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
impl From<Vec2> for IsoWorldCoords {
  fn from(value: Vec2) -> Self {
    IsoWorldCoords(value)
  }
}
impl std::ops::Add for IsoWorldCoords {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    Self(rhs.0 + self.0)
  }
}

impl IsoWorldCoords {
  pub fn distance_squared(&self, other: IsoWorldCoords) -> f32 {
    self.0.distance_squared(other.0)
  }
  pub fn with_x(&self, x: f32) -> Self {
    Self(self.0.with_x(x))
  }
  pub fn with_y(&self, y: f32) -> Self {
    Self(self.0.with_y(y))
  }
  pub fn to_screen(&self, aspect_ratio: f32) -> Vec2 {
    world_to_screen(*self, aspect_ratio)
  }
  pub fn from_screen(screen: Vec2, aspect_ratio: f32) -> Self {
    screen_to_world(screen, aspect_ratio)
  }
}

pub fn world_to_screen(world: IsoWorldCoords, aspect_ratio: f32) -> Vec2 {
  Vec2::new(
    (world.x - world.y) / 2.,
    (world.x + world.y) * aspect_ratio / 2.,
  )
}
pub fn screen_to_world(screen: Vec2, aspect_ratio: f32) -> IsoWorldCoords {
  Vec2::new(
    screen.x + screen.y / aspect_ratio,
    screen.y / aspect_ratio - screen.x,
  )
  .into()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  pub fn iso_coords_reversable() {
    let ar = 23. / 11.;
    let world = Vec2::new(314., 43.).into();
    assert_eq!(screen_to_world(world_to_screen(world, ar), ar), world);
  }
}
