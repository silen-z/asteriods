use super::*;
use bevy::prelude::*;

pub struct Asteroid;

pub struct Shard;

pub fn spawn_asteroids(
    mut commands: Commands,
    ship: Res<PlayerSpaceship>,
    ships: Query<(Entity, &Transform)>,
    asteroids: Query<(), With<Asteroid>>,
    materials: Res<GameMaterials>,
) {
    if asteroids.iter().count() < 5 {
        if let Ok((entity, spaceship)) = ships.get(ship.0) {
            let mut position = around(spaceship.translation, 1000.);

            let direction = (spaceship.translation - position).normalize() * 100.0;

            position.z += 0.5;

            let rot = thread_rng().gen_range(-1.5..1.5);

            commands
                .spawn_bundle(SpriteSheetBundle {
                    texture_atlas: materials.asteroid.clone(),
                    transform: Transform::from_translation(position),
                    ..Default::default()
                })
                .insert(CleanupAfterGame)
                .insert(Asteroid)
                .insert(Hitpoints(3))
                .insert(Velocity::from(direction))
                // .with(Rotation::from(rot))
                .insert(Collider(Vec2::new(16., 16.)))
                .insert(HitableByLaser {
                    damage_tick: Timer::from_seconds(0.15, false),
                })
                .insert(MaximumDistanceFrom {
                    anchor: entity,
                    distance: 1200.0,
                });
        }
    }
}

pub fn asteroid_damage(
    mut cmd: Commands,
    mut asteroids: Query<
        (Entity, &Hitpoints, &mut TextureAtlasSprite, &Transform),
        With<Asteroid>, // this makes sure asteroid is not already destoyed
    >,
    materials: Res<GameMaterials>,
) {
    for (entity, hp, mut sprite, transform) in asteroids.iter_mut() {
        sprite.index = 3 - hp.0;

        if hp.is_dead() {
            cmd.entity(entity)
                .remove::<(Velocity, Collider, HitableByLaser, Hitpoints)>()
                .insert(Lifetime::millis(200));

            for i in 1..=5 {
                let dir = (TAU / 5.0) * i as f32;
                let dir = Quat::from_rotation_z(dir + (random::<f32>() - 1.0));

                let rotation = Quat::from_rotation_z(random::<f32>() * TAU);

                cmd.spawn_bundle(SpriteSheetBundle {
                    texture_atlas: materials.asteroid.clone(),
                    transform: Transform {
                        translation: (transform.translation + dir * Vec3::new(10.0, 0., 0.))
                            + Vec3::new(0., 0., -0.1),
                        scale: Vec3::splat(0.3),
                        rotation,
                    },
                    ..Default::default()
                })
                .insert(CleanupAfterGame)
                .insert(Shard)
                .insert(Velocity::from(dir * Vec3::Y * 15.0))
                .insert(Lifetime::seconds(2));
            }
        }
    }
}

fn around(point: Vec3, radius: f32) -> Vec3 {
    let dir = Quat::from_rotation_z(random::<f32>() * std::f32::consts::TAU);

    point + dir * Vec3::new(radius, 0., 0.0)
}
