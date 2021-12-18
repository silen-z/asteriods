#![feature(total_cmp)]

mod asteroids;
mod basics;
mod camera;
mod hud;
mod level_generation;
mod math;
mod menu;
mod weapons;

use std::f32::consts::TAU;

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::sprite::collide_aabb::collide;

use asteroids::*;
use basics::*;
use camera::*;
use hud::*;
use level_generation::*;
use rand::{random, thread_rng, Rng as _, SeedableRng};
use weapons::*;

pub const APP_STATE_STAGE: &str = "app_state_stage";
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum AppState {
    Menu,
    InGame,
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb_u8(0, 20, 24)))
        // .add_resource(Msaa { samples: 1 })
        .add_startup_system(setup.system())
        .add_state(AppState::Menu)
        .insert_resource(MouseWorldPos::default())
        .insert_resource(LevelGenerator::new(123))
        .init_resource::<GameMaterials>()
        .init_resource::<UiMaterials>()
        .add_plugin(menu::MenuPlugin)
        .add_system_set(
            SystemSet::on_enter(AppState::InGame)
                .with_system(start_game.system())
                .with_system(init_hud.system()),
        )
        .add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(generate_background.system())
                .with_system(cleanup_chunks.system())
                .with_system(laser_beam.system())
                .with_system(continuous_rotation.system())
                .with_system(lifetime.system())
                .with_system(maximum_distance_from.system())
                .with_system(sprite_animation.system())
                .with_system(spawn_asteroids.system())
                .with_system(mouse_position.system())
                .with_system(ship_movement.system())
                .with_system(movement.system())
                .with_system(camera::camera_follow.system())
                .with_system(weapon_system_switch_weapon.system())
                .with_system(weapon_system_fire.system())
                .with_system(ship_cannon.system())
                .with_system(ship_laser.system())
                .with_system(laser_beam_init.system())
                .with_system(laser_impact.system())
                .with_system(bullets_hit_asteroids.system())
                .with_system(laser_beams_hit_asteroids.system())
                .with_system(asteroid_damage.system())
                .with_system(asteroids_hit_ship.system())
                .with_system(ship_eats_shards.system())
                .with_system(hud_healthbar.system()),
        )
        .add_system_set(
            SystemSet::on_exit(AppState::InGame).with_system(cleanup::<CleanupAfterGame>.system()),
        )
        .run();
}

pub struct GameMaterials {
    spaceship2: Handle<TextureAtlas>,
    bullet: Handle<ColorMaterial>,
    asteroid: Handle<TextureAtlas>,
    laser: Handle<ColorMaterial>,
    laser_impact: Handle<TextureAtlas>,
    nebulas: Vec<Handle<ColorMaterial>>,
    star: Handle<ColorMaterial>,
    // ship: Handle<ColorMaterial>,
}

impl FromWorld for GameMaterials {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        let mut texture_atlases = world.get_resource_mut::<Assets<TextureAtlas>>().unwrap();

        let asteroid = asset_server.load("asteroid.png");
        let asteroid = TextureAtlas::from_grid(asteroid, Vec2::new(48.0, 48.0), 4, 1);

        let laser_impact = asset_server.load("laser_impact.png");
        let laser_impact = TextureAtlas::from_grid(laser_impact, Vec2::new(8.0, 8.0), 4, 1);

        let spaceship2 = asset_server.load("spaceship2.png");
        let spaceship2 = TextureAtlas::from_grid(spaceship2, Vec2::new(32.0, 32.0), 8, 1);

        GameMaterials {
            spaceship2: texture_atlases.add(spaceship2.into()),
            bullet: materials.add(asset_server.load("bullet.png").into()),
            asteroid: texture_atlases.add(asteroid.into()),
            laser: materials.add(asset_server.load("laser_beam.png").into()),
            laser_impact: texture_atlases.add(laser_impact.into()),
            nebulas: vec![materials.add(asset_server.load("nebula.png").into())],
            star: materials.add(asset_server.load("star.png").into()),
            // ship: materials.add(asset_server.load("spaceship.png").into()),
        }
    }
}

pub struct Spaceship {
    score: u32,
}

#[derive(Default)]
pub struct MouseWorldPos(Vec3);

impl MouseWorldPos {
    fn dir_from(&self, pos: Vec3) -> Vec3 {
        let mut dir = self.0 - pos;
        dir.z = 0.;
        dir.normalize()
    }
}

pub struct Collider(Vec2);

pub struct HitableByLaser {
    damage_tick: Timer,
}

struct CleanupAfterGame;

pub struct PlayerSpaceship(Entity);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    if let Err(e) = asset_server.watch_for_changes() {
        eprintln!("not able to enable hot-reloading: {}", e);
    }

    commands.spawn_bundle(UiCameraBundle::default());

    let main_camera = commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .id();

    commands.insert_resource(MainCamera(main_camera));
}

fn start_game(mut cmd: Commands, sprites: Res<GameMaterials>) {
    let spaceship = cmd
        // .spawn(SpriteBundle {
        //     material: sprites.ship.clone(),
        //     ..Default::default()
        // })
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: sprites.spaceship2.clone(),
            ..Default::default()
        })
        .insert(Spaceship { score: 0 })
        .insert(Hitpoints(100))
        .insert(Velocity::default())
        .insert(CleanupAfterGame)
        .insert(ChunkExplorer)
        .insert(Collider(Vec2::new(16., 48.)))
        .insert(WeaponSystem {
            slots: 2,
            current: 0,
            is_firing: false,
        })
        .with_children(|ship| {
            ship.spawn_bundle(WeaponBundle::new(
                WeaponCannon::default(),
                0,
                ship.parent_entity(),
            ));
            ship.spawn_bundle(WeaponBundle::new(
                WeaponLaser::default(),
                1,
                ship.parent_entity(),
            ));
        })
        .id();

    cmd.insert_resource(PlayerSpaceship(spaceship));

    // STATIC TEST ASTERIOD
    // commands
    //     .spawn(SpriteSheetBundle {
    //         texture_atlas: sprites.asteroid.clone(),
    //         transform: Transform::from_translation(Vec3::new(150., 150., 0.)),
    //         ..Default::default()
    //     })
    //     .with(CleanupAfterGame)
    //     .with(Asteroid)
    //     .with(Hitpoints(3))
    //     .with(HitableByLaser {
    //         damage_tick: Timer::from_seconds(1., false),
    //     });
}

fn mouse_position(
    main_camera: Res<MainCamera>,
    windows: Res<Windows>,
    camera_query: Query<&Transform, With<Camera>>,
    mut cursor_moved_reader: EventReader<CursorMoved>,
    mut mouse_world_pos: ResMut<MouseWorldPos>,
    mut last_mouse_event: Local<Option<CursorMoved>>,
) {
    if let Some(last_event) = cursor_moved_reader.iter().last() {
        *last_mouse_event = Some(last_event.clone());
    }

    if let Some(event) = last_mouse_event.as_ref() {
        use bevy::math::Vec4Swizzles as _;
        let camera_transform = camera_query.get(main_camera.0).unwrap();
        let window = windows.get(event.id).unwrap();
        let window_size = Vec2::new(window.width() as f32, window.height() as f32);
        let p = event.position - window_size * 0.5;

        mouse_world_pos.0 = (camera_transform.compute_matrix() * p.extend(0.0).extend(1.0)).xyz();
    }
}

const SHIP_SPEED_GAIN: f32 = 1000.0;
const SHIP_MAX_SPEED: f32 = 500.0;

fn ship_movement(
    mouse_input: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    mut ships: Query<(&Transform, &mut Velocity, &mut TextureAtlasSprite), With<Spaceship>>,
    time: Res<Time>,
    mouse_pos: Res<MouseWorldPos>,
) {
    for (transform, mut velocity, mut sprite) in ships.iter_mut() {
        let dir_to_target = mouse_pos.dir_from(transform.translation);

        let angle = Vec3::Y.angle_between(dir_to_target);

        let angle = if dir_to_target.x.signum() < 0. {
            TAU - angle
        } else {
            angle
        };
        let sprite_direction = (8.0 * angle / TAU).round();

        sprite.index = (8 - sprite_direction as u32) % 8;

        // let angle = ROTATION_CLAMP * (angle / ROTATION_CLAMP).ceil();

        // transform.rotation = Quat::from_rotation_z(
        //     dir_to_target.angle_between(Vec3::unit_y()) * -dir_to_target.x.signum(),
        // );

        let mut acceleration = Vec3::default();

        if mouse_input.pressed(MouseButton::Right) {
            acceleration += dir_to_target;
        } else {
            if keys.pressed(KeyCode::W) {
                acceleration.y += 1.;
            }

            if keys.pressed(KeyCode::A) {
                acceleration.x -= 1.;
            }

            if keys.pressed(KeyCode::S) {
                acceleration.y -= 1.;
            }

            if keys.pressed(KeyCode::D) {
                acceleration.x += 1.;
            }
        }

        if acceleration.length_squared() > 0. {
            let gain = acceleration.normalize() * SHIP_SPEED_GAIN * time.delta_seconds();
            velocity.0 = (velocity.0 + gain)
                .min(Vec3::splat(SHIP_MAX_SPEED))
                .max(-Vec3::splat(SHIP_MAX_SPEED));
        } else {
            velocity.0 = Vec3::max(
                velocity.0 - Vec3::splat(SHIP_SPEED_GAIN) * time.delta_seconds(),
                Vec3::ZERO,
            );
        }
    }
}

fn asteroids_hit_ship(
    mut cmd: Commands,
    mut ships: Query<(&mut Hitpoints, &Transform, &Collider), With<Spaceship>>,
    mut asteroids: Query<(Entity, &Transform, &Collider, &mut TextureAtlasSprite), With<Asteroid>>,
    mut states: ResMut<State<AppState>>,
) {
    for (mut hp, transform, collider) in ships.iter_mut() {
        for (asteroid, asteroid_transform, asteroid_collider, mut sprite) in asteroids.iter_mut() {
            if collide(
                transform.translation,
                collider.0,
                asteroid_transform.translation,
                asteroid_collider.0,
            )
            .is_some()
            {
                sprite.index = 3;

                cmd.entity(asteroid)
                    .remove::<(Velocity, Collider, Hitpoints)>()
                    .insert(Lifetime::millis(200));

                hp.damage(10);

                if hp.is_dead() {
                    states.replace(AppState::Menu).unwrap();
                    return;
                }
            }
        }
    }
}

pub fn bullets_hit_asteroids(
    mut cmd: Commands,
    mut asteroids: Query<(&mut Hitpoints, &Transform, &Collider), With<Asteroid>>,
    mut bullets: Query<(&mut Bullet, Entity, &Transform, &Collider)>,
) {
    for (mut hp, transform, collider) in asteroids.iter_mut() {
        for (mut bullet, bullet_entity, bullet_transform, bullet_collider) in bullets.iter_mut() {
            if !bullet.already_hit
                && collide(
                    transform.translation,
                    collider.0,
                    bullet_transform.translation,
                    bullet_collider.0,
                )
                .is_some()
            {
                bullet.already_hit = true;
                hp.damage(1);
                cmd.entity(bullet_entity).despawn();
            }
        }
    }
}

pub fn laser_beams_hit_asteroids(mut asteroids: Query<(&mut Hitpoints, &mut HitableByLaser)>) {
    for (mut hp, mut hitable) in asteroids.iter_mut() {
        if hitable.damage_tick.just_finished() {
            hitable.damage_tick.reset();
            hp.damage(1);
        }
    }
}

fn ship_eats_shards(
    mut cmd: Commands,
    mut ships: Query<(&mut Spaceship, &Transform), Without<Shard>>,
    mut shards: Query<(Entity, &mut Transform), With<Shard>>,
) {
    for (mut ship, ship_transform) in ships.iter_mut() {
        for (entity, transform) in shards.iter_mut() {
            let dist = ship_transform
                .translation
                .distance_squared(transform.translation);

            if dist < 400.0 {
                ship.score += 10;
                cmd.entity(entity).despawn()
            }
        }
    }
}
