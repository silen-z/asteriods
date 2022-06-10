use bevy::prelude::*;

use crate::basics::{Lifetime, Velocity};

#[derive(Component)]
pub struct MagnetAttractable;

#[derive(Component)]
pub struct Magnet {
    pub force: f32,
    pub max_distance: f32,
}

pub fn magnets(
    mut attracted_items: Query<
        (&Transform, &mut Velocity, Option<&mut Lifetime>),
        With<MagnetAttractable>,
    >,
    magntes: Query<(&Transform, &Magnet), Without<MagnetAttractable>>,
    time: Res<Time>,
) {
    for (item_transform, mut item_velocity, item_lifetime) in attracted_items.iter_mut() {
        let closest_magnet = magntes
            .iter()
            .filter(|(t, magnet)| {
                t.translation.distance_squared(item_transform.translation)
                    < magnet.max_distance.powi(2)
            })
            .min_by(|(m1, _), (m2, _)| {
                f32::total_cmp(
                    &m1.translation.distance_squared(item_transform.translation),
                    &m2.translation.distance_squared(item_transform.translation),
                )
            });

        if let Some((magnet_transform, magnet)) = closest_magnet {
            use bevy::math::Vec3Swizzles;

            let magnet_pos = magnet_transform.translation.xy();
            let item_pos = item_transform.translation.xy();

            let force_direction = (magnet_pos - item_pos).normalize();

            let force = force_direction * (magnet.force * 100.) * time.delta_seconds();

            item_velocity.0 = force.extend(0.);

            if let Some(mut item_lifetime) = item_lifetime {
                item_lifetime.prevent_tick = true;
            }
        }
    }
}
