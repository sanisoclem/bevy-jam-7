use std::sync::Arc;

use bevy::prelude::*;
use sys_move::{IsoWorldCoords, Moveable, Placeable};

use crate::CombatAreaEffect;

#[derive(Component, Clone)]
pub struct Projectile {
  pub lifetime: Timer,
  pub detonate_trigger: DetonationTrigger,
  pub payload: Option<ProjectilePayload>,
  pub movement: ProjectileMovement,
}

#[derive(Debug, Clone)]
pub enum ProjectileMovement {
  Static,
  Straight(Vec2),
  Seek {
    target: Entity,
    max_angular_velocity: f32,
    speed: f32,
  },
  // Orbit {
  //   target: Entity,
  //   orbit_radius: f32,
  // },
}

#[derive(Debug, Clone)]
pub enum DetonationTrigger {
  Contact,
  Expiry,
}

pub type PayloadFn =
  Arc<dyn Fn(&mut Commands, &IsoWorldCoords) -> Vec<Entity> + Send + Sync + 'static>;

#[derive(Clone)]
pub enum ProjectilePayload {
  SpawnEntities(Entity, PayloadFn),
}

#[derive(Message, Clone)]
pub struct DetonatePayload {
  pub payload: ProjectilePayload,
  pub location: IsoWorldCoords,
}

pub fn process_detonations(mut cmd: Commands, mut msg_reader: MessageReader<DetonatePayload>) {
  for msg in msg_reader.read() {
    let ProjectilePayload::SpawnEntities(parent, spawn_fn) = &msg.payload;
    let spawned = spawn_fn(&mut cmd, &msg.location);

    if let Ok(mut pcmd) = cmd.get_entity(*parent) {
      pcmd.add_children(&spawned);
    } else {
      warn!("orphaned projectile payloads created");
      continue;
    };
  }
}
pub fn despawn_expired_projectiles(
  mut cmd: Commands,
  qry: Query<(Entity, &mut Projectile, &Placeable)>,
  mut msg_writer: MessageWriter<DetonatePayload>,
  time: Res<Time>,
) {
  for (e, mut p, loc) in qry {
    p.lifetime.tick(time.delta());

    if p.lifetime.just_finished() {
      cmd.entity(e).despawn();

      if let (DetonationTrigger::Expiry, Some(payload)) = (&p.detonate_trigger, &p.payload) {
        msg_writer.write(DetonatePayload {
          payload: payload.clone(),
          location: loc.location,
        });
      };
    }
  }
}

pub fn detonate_hit_projectiles(
  mut cmd: Commands,
  qry: Query<(Entity, &Projectile, &CombatAreaEffect, &Placeable)>,
  mut msg_writer: MessageWriter<DetonatePayload>,
) {
  for (e, p, cae, loc) in qry {
    if !cae.hit {
      continue;
    }

    if let (DetonationTrigger::Contact, Some(payload)) = (&p.detonate_trigger, &p.payload) {
      cmd.entity(e).despawn();
      msg_writer.write(DetonatePayload {
        payload: payload.clone(),
        location: loc.location,
      });
    };
  }
}

pub fn update_movement_forces(
  qry_pos: Query<&Placeable>,
  qry: Query<(&Placeable, &mut Moveable, &Projectile)>,
  time: Res<Time>,
) {
  for (p, mut mov, proj) in qry {
    mov.net_forces = match proj.movement {
      ProjectileMovement::Static => Vec2::splat(0.),
      ProjectileMovement::Straight(vel) => vel,
      ProjectileMovement::Seek {
        target,
        max_angular_velocity,
        speed,
      } => {
        if let Ok(target_pos) = qry_pos.get(target) {
          let dist = p.location.distance(target_pos.location);

          if dist > 0.01 {
            let desired_dir = (target_pos.location - p.location).normalize();
            let current_dir = mov.net_forces.normalize_or_zero();
            if current_dir.length_squared() > 0.01 {
              let max_turn = max_angular_velocity * time.delta_secs();
              let angle = current_dir.angle_to(desired_dir).clamp(-max_turn, max_turn);
              current_dir.rotate(Vec2::from_angle(angle)) * speed
            } else {
              desired_dir * speed
            }
          } else {
            Vec2::ZERO
          }
        } else {
          mov.net_forces
        }
      } // ProjectileMovement::Orbit {
        //   target,
        //   orbit_radius,
        // } => {
        //   if let Ok(pos) = qry_pos.get(target) {
        //     todo!()
        //   } else {
        //     mov.net_forces
        //   }
        // }
    };
  }
}
