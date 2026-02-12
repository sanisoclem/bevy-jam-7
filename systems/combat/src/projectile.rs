use bevy::prelude::*;
use sys_move::{IsoWorldCoords, Moveable, Placeable};

#[derive(Component, Clone)]
pub struct Projectile {
  pub lifetime: Timer,
  pub detonate_trigger: DetonationTrigger,
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
  None,
  Contact,
  Expiry,
  Pulse(Timer),
}

#[derive(EntityEvent, Clone)]
pub struct DetonatePayload {
  #[event_target]
  pub target: Entity,
  pub location: IsoWorldCoords,
  pub hit: Option<Entity>,
}

pub fn despawn_expired_projectiles(
  mut cmd: Commands,
  qry: Query<(Entity, &mut Projectile, &Placeable)>,
  time: Res<Time>,
) {
  for (e, mut p, loc) in qry {
    p.lifetime.tick(time.delta());

    if p.lifetime.just_finished() {
      // if detonate on expiry, detonation will be determined by the projectile owner
      if let DetonationTrigger::Expiry = &p.detonate_trigger {
        cmd.trigger(DetonatePayload {
          target: e,
          location: loc.location,
          hit: None,
        });
      } else {
        // no detonation on expiry, just  despawn
        cmd.entity(e).despawn();
      }
    }
  }
}
pub fn pulse_projectiles(
  mut cmd: Commands,
  qry: Query<(Entity, &mut Projectile, &Placeable)>,
  time: Res<Time>,
) {
  for (e, mut p, loc) in qry {
    let DetonationTrigger::Pulse(timer) = &mut p.detonate_trigger else {
      continue;
    };

    timer.tick(time.delta());

    if timer.just_finished() {
      cmd.trigger(DetonatePayload {
        target: e,
        location: loc.location,
        hit: None,
      });
    }
  }
}

pub fn update_movement_forces(
  qry_pos: Query<&Placeable>,
  qry: Query<(&Placeable, &mut Moveable, &Projectile)>,
  time: Res<Time>,
) {
  for (p, mut mov, proj) in qry {
    // if detonated.is_some() {
    //   mov.net_forces = Vec2::ZERO;
    //   return;
    // }
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
      } /* ProjectileMovement::Orbit {
         *   target,
         *   orbit_radius,
         * } => {
         *   if let Ok(pos) = qry_pos.get(target) {
         *     todo!()
         *   } else {
         *     mov.net_forces
         *   }
         * } */
    };
  }
}
