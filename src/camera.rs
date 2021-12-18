use bevy::prelude::*;
use bevy::render::camera::Camera;

use crate::PlayerSpaceship;

pub struct MainCamera(pub Entity);

pub fn camera_follow(
    spaceship: Res<PlayerSpaceship>,
    main_camera: Res<MainCamera>,
    spaceships: Query<&Transform, Without<Camera>>,
    mut cameras: Query<&mut Transform, With<Camera>>,
) {
    if let Ok(ship) = spaceships.get(spaceship.0) {
        let mut camera = cameras.get_mut(main_camera.0).unwrap();
        camera.translation.x = ship.translation.x;
        camera.translation.y = ship.translation.y;
    }
}

// pub struct CameraFollow(pub Entity);

// pub fn camera_follow(
//     cmds: &mut Commands,
//     mut cameras: Query<(&CameraFollow, &mut Transform)>,
//     entities: Query<&Transform, Without<CameraFollow>>,
// ) {
//     for (camera_follow, mut camera) in cameras.iter_mut() {
//         if let Ok(followed) = entities.get(camera_follow.0) {
//             camera.translation.x = followed.translation.x;
//             camera.translation.y = followed.translation.y;
//         } else {
//             cmds.remove_one::<CameraFollow>(camera_follow.0);
//         }
//     }
// }
