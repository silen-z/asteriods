use super::*;
use bevy::prelude::*;

pub struct Asteroid {
    pub hits_needed: u8,
}

pub struct Shard;

pub fn spawn_asteroids(
    commands: &mut Commands,
    query: Query<(Entity, &Transform), With<Spaceship>>,
    asteroids: Query<(), With<Asteroid>>,
    materials: Res<GameMaterials>,
) {
    if asteroids.iter().count() < 5 {
        if let Some((entity, spaceship)) = query.iter().next() {
            let mut position = around(spaceship.translation, 1000.);

            let direction = (spaceship.translation - position).normalize() * 100.0;

            position.z += 0.5;

            let rot = thread_rng().gen_range(-1.5..1.5);

            commands
                .spawn(SpriteSheetBundle {
                    texture_atlas: materials.asteroid.clone(),
                    transform: Transform::from_translation(position),
                    ..Default::default()
                })
                .with(CleanupAfterGame)
                .with(Asteroid { hits_needed: 3 })
                .with(Movement::from(direction))
                .with(Rotation::from(rot))
                .with(Collider(Vec2::new(16., 16.)))
                .with(HitableByLaser {
                    damage_tick: Timer::from_seconds(0.15, false),
                })
                .with(MaximumDistanceFrom {
                    anchor: entity,
                    distance: 1200.0,
                });
        }
    }
}

pub fn bullets_hit_asteroids(
    cmd: &mut Commands,
    mut asteroids: Query<(&Transform, &Collider)>,
    mut bullets: Query<(&mut Bullet, Entity, &Transform, &Collider)>,
) {
    for (transform, collider) in asteroids.iter_mut() {
        for (mut bullet, bullet_entity, bullet_transform, bullet_collider) in bullets.iter_mut() {
            if bullet.0 == false
                && collide(
                    transform.translation,
                    collider.0,
                    bullet_transform.translation,
                    bullet_collider.0,
                )
                .is_some()
            {
                bullet.0 = true;
                cmd.despawn(bullet_entity);
            }
        }
    }
}

pub fn laser_beams_hit_asteroids(mut asteroids: Query<(&mut Asteroid, &mut HitableByLaser)>) {
    for (mut asteroid, mut hitable) in asteroids.iter_mut() {
        if hitable.damage_tick.just_finished() {
            hitable.damage_tick.reset();
            asteroid.hits_needed = asteroid.hits_needed.saturating_sub(1);
        }
    }
}

pub fn asteroid_damage(
    cmd: &mut Commands,
    mut asteroids: Query<
        (Entity, &Asteroid, &mut TextureAtlasSprite, &Transform),
        Without<Lifetime>, // this makes sure asteroid is not already destoyed
    >,
    materials: Res<GameMaterials>,
) {
    for (entity, asteroid, mut sprite, transform) in asteroids.iter_mut() {
        sprite.index = 3 - asteroid.hits_needed as u32;

        if asteroid.hits_needed == 0 {
            cmd.remove::<(Movement, Collider, HitableByLaser)>(entity)
                .insert_one(entity, Lifetime::millis(200));

            for i in 1..=5 {
                let dir = (TAU / 5.0) * i as f32;
                let dir = Quat::from_rotation_z(dir + (random::<f32>() - 1.0));

                let rotation = Quat::from_rotation_z(random::<f32>() * TAU);

                cmd.spawn(SpriteSheetBundle {
                    texture_atlas: materials.asteroid.clone(),
                    transform: Transform {
                        translation: (transform.translation + dir * Vec3::new(10.0, 0., 0.))
                            + Vec3::new(0., 0., -0.1),
                        scale: Vec3::splat(0.3),
                        rotation,
                    },
                    ..Default::default()
                })
                .with(CleanupAfterGame)
                .with(Shard)
                .with(Movement::from(dir * Vec3::unit_x() * 15.0))
                .with(Lifetime::seconds(2));
            }
        }
    }
}
