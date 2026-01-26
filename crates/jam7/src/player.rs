pub use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

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
  mut players: Query<&mut Transform, With<Player>>,
) {
  let mut transform = players.get_mut(movement.context).unwrap();
  transform.translation += (movement.value.normalize() * 20.).extend(0.0);
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
          Transform::default().with_translation(location.extend(1.)),
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
