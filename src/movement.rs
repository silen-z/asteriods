use bevy::prelude::*;

pub struct Movement(Vec3);

impl From<Vec3> for Movement {
    fn from(v: Vec3) -> Self {
        Self(v)
    }
}

pub struct Rotation(f32);

impl From<f32> for Rotation {
    fn from(v: f32) -> Self {
        Self(v)
    }
}

pub fn continuous_movement(time: Res<Time>, mut query: Query<(&Movement, &mut Transform)>) {
    for (movement, mut transform) in query.iter_mut() {
        transform.translation += movement.0 * time.delta_seconds();
    }
}

pub fn continuous_rotation(time: Res<Time>, mut query: Query<(&Rotation, &mut Transform)>) {
    for (rotation, mut transform) in query.iter_mut() {
        let rotation = transform.rotation * Quat::from_rotation_z(rotation.0);
        transform.rotation = transform.rotation.slerp(rotation, time.delta_seconds());
    }
}