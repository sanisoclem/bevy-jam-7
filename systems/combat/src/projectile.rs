use bevy::prelude::*;
use sys_move::{Moveable, Placeable};

use crate::CombatAreaEffect;

#[derive(Component, Debug, Clone)]
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

pub type PayloadFn = fn(&mut Commands);

#[derive(Debug, Clone)]
pub enum ProjectilePayload {
  SpawnEntities(PayloadFn),
}

#[derive(Message, Clone, Debug)]
pub struct DetonatePayload {
  pub payload: ProjectilePayload,
}

pub fn despawn_expired_projectiles(
  mut cmd: Commands,
  qry: Query<(Entity, &mut Projectile)>,
  mut msg_writer: MessageWriter<DetonatePayload>,
  time: Res<Time>,
) {
  for (e, mut p) in qry {
    p.lifetime.tick(time.delta());

    if p.lifetime.just_finished() {
      cmd.entity(e).despawn();

      if let (DetonationTrigger::Expiry, Some(payload)) = (&p.detonate_trigger, &p.payload) {
        msg_writer.write(DetonatePayload {
          payload: payload.clone(),
        });
      };
    }
  }
}

pub fn detonate_hit_projectiles(
  mut cmd: Commands,
  qry: Query<(Entity, &Projectile, &CombatAreaEffect)>,
  mut msg_writer: MessageWriter<DetonatePayload>,
) {
  for (e, p, cae) in qry {
    if cae.hit {
      cmd.entity(e).despawn();

      if let (DetonationTrigger::Contact, Some(payload)) = (&p.detonate_trigger, &p.payload) {
        msg_writer.write(DetonatePayload {
          payload: payload.clone(),
        });
      };
    }
  }
}

pub fn update_movement_forces(
  qry_pos: Query<&Placeable>,
  qry: Query<(&Placeable, &mut Moveable, &Projectile)>,
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
            let desired_velocity = (target_pos.location - p.location).normalize() * speed;
            let current_velocity = mov.net_forces;
            let mut steering = desired_velocity - current_velocity;

            let max_turn = max_angular_velocity * dist.min(1.0);
            if steering.length() > max_turn {
              steering = steering.normalize() * max_turn;
            }

            current_velocity + steering
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
