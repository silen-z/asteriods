use bevy::prelude::*;

pub struct Lifetime(Timer);

impl Lifetime {
    pub fn millis(millis: u32) -> Self {
        Self(Timer::from_seconds(millis as f32 / 1000., false))
    }

    pub fn seconds(seconds: u32) -> Self {
        Self(Timer::from_seconds(seconds as f32, false))
    }
}

pub struct MaximumDistanceFrom {
    pub anchor: Entity,
    pub distance: f32,
}

pub fn lifetime(
    commands: &mut Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        if lifetime.0.tick(time.delta_seconds()).finished() {
            commands.despawn(entity);
        }
    }
}

pub fn maximum_distance_from(
    cmd: &mut Commands,
    query: Query<(Entity, &Transform, &MaximumDistanceFrom)>,
    anchors: Query<&Transform>,
) {
    for (entity, transform, max_dist) in query.iter() {
        if let Ok(anchor_transform) = anchors.get(max_dist.anchor) {
            if transform.translation.distance(anchor_transform.translation) > max_dist.distance {
                cmd.despawn(entity);
            }
        } else {
            cmd.despawn(entity);
        }
    }
}
