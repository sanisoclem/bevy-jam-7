use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use sys_move::{IsoWorldCoords, Moveable, Placeable};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_input_context::<Player>()
      .add_message::<PlayerCommand>()
      .add_observer(apply_movement)
      .add_systems(Update, process_player_commands);
  }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct PlayerId(pub i32);

#[derive(Component, Debug)]
pub struct Player {
  pub id: PlayerId,
}

#[derive(InputAction)]
#[action_output(Vec2)]
pub struct ActionMovePlayer;

#[derive(Message, Debug)]
pub enum PlayerCommand {
  SpawnPlayer(PlayerId, Vec2),
  DespawnPlayer(PlayerId),
}

fn apply_movement(
  movement: On<Fire<ActionMovePlayer>>,
  mut players: Query<&mut Moveable, With<Player>>,
) {
  let mut mv = players.get_mut(movement.context).unwrap();
  mv.net_forces = movement.value.normalize() * 10.;
}

fn process_player_commands(
  mut cmd: Commands,
  mut reader: MessageReader<PlayerCommand>,
  qry_player: Query<(Entity, &Player)>,
) {
  for command in reader.read() {
    match command {
      PlayerCommand::SpawnPlayer(player_id, location) => {
        if let Some(_existing) = qry_player.iter().find(|(_, p)| &p.id == player_id) {
          continue;
        };
        cmd.spawn((
          Player { id: *player_id },
          Transform::default(),
          Visibility::default(),
          Moveable {
            damping: 1.0,
            mass: 0.001,
            net_forces: Vec2::default(),
          },
          Placeable {
            layer: 7,
            location: IsoWorldCoords::default(),
          },
        ));
      }
      PlayerCommand::DespawnPlayer(player_id) => {
        let Some((existing, _)) = qry_player.iter().find(|(_, p)| &p.id == player_id) else {
          continue;
        };

        cmd.entity(existing).despawn();
      }
    };
  }
}
