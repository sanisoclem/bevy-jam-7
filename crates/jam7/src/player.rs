use bevy::{color::palettes::css::GREEN, prelude::*};
use bevy_enhanced_input::prelude::*;
use sys_cam::CameraTarget;
use sys_move::{IsoMovementStage, IsoWorldCoords, Moveable, Placeable};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_input_context::<Player>()
      .add_observer(apply_movement)
      .add_observer(stop_movement);
  }
}

#[derive(Component, Debug)]
pub struct Player;

pub fn create_player(
  meshes: &mut Assets<Mesh>,
  materials: &mut Assets<ColorMaterial>,
) -> impl Bundle {
  let mesh = Rectangle::from_size(Vec2::splat(32.0)).mesh().build();
  (
    Player,
    CameraTarget,
    Transform::default(),
    Visibility::default(),
    Mesh2d(meshes.add(mesh)),
    MeshMaterial2d(materials.add(Color::from(GREEN))),
    Moveable {
      damping: 1.0,
      // mass: 0.01,
      net_forces: Vec2::default(),
    },
    Placeable {
      layer: 7,
      location: IsoWorldCoords::default(),
    },
    actions!(
      Player[(
        Action::<ActionMovePlayer>::new(),
        DeadZone::default(), // Applies non-uniform normalization.
        bindings![
          // Keyboard keys captured as `bool`, but the output of `Movement` is defined as `Vec2`,
          // so you need to assign keys to axes using swizzle to reorder them and negation.
          (KeyCode::KeyW, SwizzleAxis::YXZ),
          (KeyCode::KeyA, Negate::all()),
          (KeyCode::KeyS, Negate::all(), SwizzleAxis::YXZ),
          KeyCode::KeyD,
          // In Bevy sticks split by axes and captured as 1-dimensional inputs,
          // so Y stick needs to be sweezled into Y axis.
          GamepadAxis::LeftStickX,
          (GamepadAxis::LeftStickY, SwizzleAxis::YXZ),
        ]
      )]
    ),
  )
}

#[derive(InputAction)]
#[action_output(Vec2)]
pub struct ActionMovePlayer;

fn apply_movement(
  movement: On<Fire<ActionMovePlayer>>,
  mut players: Query<(&mut Moveable, &ChildOf), With<Player>>,
  qry_stage: Query<&IsoMovementStage>,
) {
  let Ok((mut mv, co)) = players.get_mut(movement.context) else {
    return;
  };
  let Ok(stage) = qry_stage.get(co.parent()) else {
    return;
  };

  let world_direction: Vec2 = *IsoWorldCoords::from_screen(movement.value, stage.aspect_ratio);
  mv.net_forces = world_direction * 10.;
}

fn stop_movement(
  movement: On<Complete<ActionMovePlayer>>,
  mut players: Query<&mut Moveable, With<Player>>,
) {
  let Ok(mut mv) = players.get_mut(movement.context) else {
    return;
  };
  mv.net_forces = Vec2::splat(0.);
}
