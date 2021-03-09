#![feature(total_cmp)]

mod asteroids;
mod basics;
mod level_generation;
mod math;
mod menu;
mod weapons;

use std::f32::consts::TAU;

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;

use asteroids::*;
use basics::*;
use bevy::render::camera::Camera;
use level_generation::*;
use rand::{random, thread_rng, Rng as _, SeedableRng};
use weapons::*;

pub const APP_STATE_STAGE: &str = "app_state_stage";
#[derive(Clone)]
pub enum AppState {
    Menu,
    InGame,
}

fn main() {
    let mut app_state_stage = StateStage::<AppState>::default();
    app_state_stage
        .on_state_enter(AppState::InGame, start_game.system())
        .on_state_update(AppState::InGame, generate_background.system())
        .on_state_update(AppState::InGame, cleanup_chunks.system())
        .on_state_update(AppState::InGame, continuous_rotation.system())
        .on_state_update(AppState::InGame, lifetime.system())
        .on_state_update(AppState::InGame, maximum_distance_from.system())
        .on_state_update(AppState::InGame, sprite_animation.system())
        .on_state_update(AppState::InGame, spawn_asteroids.system())
        .on_state_update(AppState::InGame, mouse_position.system())
        .on_state_update(AppState::InGame, ship_movement.system())
        .on_state_update(AppState::InGame, movement.system())
        .on_state_update(AppState::InGame, camera_follow.system())
        .on_state_update(AppState::InGame, weapon_system_switch_weapon.system())
        .on_state_update(AppState::InGame, weapon_system_fire.system())
        .on_state_update(AppState::InGame, ship_cannon.system())
        .on_state_update(AppState::InGame, ship_laser.system())
        .on_state_update(AppState::InGame, laser_beam_init.system())
        .on_state_update(AppState::InGame, laser_beam.system())
        .on_state_update(AppState::InGame, laser_impact.system())
        .on_state_update(AppState::InGame, bullets_hit_asteroids.system())
        .on_state_update(AppState::InGame, laser_beams_hit_asteroids.system())
        .on_state_update(AppState::InGame, asteroid_damage.system())
        .on_state_update(AppState::InGame, asteroids_hit_ship.system())
        .on_state_update(AppState::InGame, ship_eats_shards.system())
        .on_state_update(AppState::InGame, hud.system())
        .on_state_exit(AppState::InGame, cleanup::<CleanupAfterGame>.system());

    App::build()
        .add_plugins(DefaultPlugins)
        .add_resource(ClearColor(Color::rgb_u8(7, 0, 17)))
        .add_startup_system(setup.system())
        .add_resource(State::new(AppState::Menu))
        .add_resource(MouseWorldPos::default())
        .add_resource(LevelGenerator::new(123))
        .init_resource::<GameMaterials>()
        .add_stage_after(stage::UPDATE, APP_STATE_STAGE, app_state_stage)
        .add_plugin(menu::MenuPlugin)
        .run();
}

pub struct GameMaterials {
    ship: Handle<ColorMaterial>,
    bullet: Handle<ColorMaterial>,
    asteroid: Handle<TextureAtlas>,
    health_bar: Handle<ColorMaterial>,
    laser: Handle<ColorMaterial>,
    laser_impact: Handle<TextureAtlas>,
    nebulas: Vec<Handle<ColorMaterial>>,
    star: Handle<ColorMaterial>,
}

impl FromResources for GameMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let asset_server = resources.get_mut::<AssetServer>().unwrap();
        let mut materials = resources.get_mut::<Assets<ColorMaterial>>().unwrap();
        let mut texture_atlases = resources.get_mut::<Assets<TextureAtlas>>().unwrap();

        let asteroid = asset_server.load("asteroid.png");
        let asteroid = TextureAtlas::from_grid(asteroid, Vec2::new(48.0, 48.0), 4, 1);

        let laser_impact = asset_server.load("laser_impact.png");
        let laser_impact = TextureAtlas::from_grid(laser_impact, Vec2::new(8.0, 8.0), 4, 1);

        GameMaterials {
            ship: materials.add(asset_server.load("spaceship.png").into()),
            bullet: materials.add(asset_server.load("bullet.png").into()),
            asteroid: texture_atlases.add(asteroid.into()),
            health_bar: materials.add(asset_server.load("ui/health_bar.png").into()),
            laser: materials.add(Color::RED.into()),
            laser_impact: texture_atlases.add(laser_impact.into()),
            nebulas: vec![
                materials.add(asset_server.load("nebula1.png").into()),
                materials.add(asset_server.load("nebula2.png").into()),
                materials.add(asset_server.load("nebula3.png").into()),
            ],
            star: materials.add(asset_server.load("star.png").into()),
        }
    }
}

pub struct Spaceship {
    hp: u32,
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

struct Healthbar;
struct Score;

struct CleanupAfterGame;

pub struct PlayerSpaceship(Entity);

struct MainCamera(Entity);

fn setup(commands: &mut Commands) {
    commands.spawn(CameraUiBundle::default());

    let main_camera = commands
        .spawn(Camera2dBundle::default())
        .current_entity()
        .unwrap();
    commands.insert_resource(MainCamera(main_camera));
}

fn start_game(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    sprites: Res<GameMaterials>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    main_camera: Res<MainCamera>,
) {
    let spaceship = commands
        .spawn(SpriteBundle {
            material: sprites.ship.clone(),
            ..Default::default()
        })
        .with(Spaceship { hp: 100, score: 0 })
        .with(Velocity::default())
        .with(CleanupAfterGame)
        .with(ChunkExplorer)
        .with(Collider(Vec2::new(16., 48.)))
        .with(WeaponSystem {
            slots: 2,
            current: 0,
            is_firing: false,
        })
        .with_children(|ship| {
            ship.spawn(WeaponBundle::new(
                WeaponCannon::default(),
                0,
                ship.parent_entity(),
            ));
            ship.spawn(WeaponBundle::new(
                WeaponLaser::default(),
                1,
                ship.parent_entity(),
            ));
        })
        .current_entity()
        .unwrap();

    commands.insert_resource(PlayerSpaceship(spaceship));

    commands.insert_one(main_camera.0, CameraFollow(spaceship));

    // TEST STATIC ASTERIOD
    commands
        .spawn(SpriteSheetBundle {
            texture_atlas: sprites.asteroid.clone(),
            transform: Transform::from_translation(Vec3::new(150., 150., 0.)),
            ..Default::default()
        })
        .with(CleanupAfterGame)
        .with(Asteroid { hits_needed: 3 })
        .with(HitableByLaser {
            damage_tick: Timer::from_seconds(1., false),
        });

    commands
        .spawn(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexStart,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(16.0),
                    left: Val::Px(16.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text {
                value: "Score:".to_string(),
                font: asset_server.load("Freshman.ttf"),
                style: TextStyle {
                    font_size: 60.0,
                    color: Color::rgb_u8(147, 14, 58),
                    ..Default::default()
                },
            },
            ..Default::default()
        })
        .with(Score)
        .with(CleanupAfterGame)
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_items: AlignItems::FlexEnd,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .with(CleanupAfterGame)
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(800.0), Val::Px(32.0)),
                        margin: Rect::all(Val::Px(16.0)),
                        ..Default::default()
                    },
                    material: materials.add(Color::NONE.into()),
                    ..Default::default()
                })
                .with(CleanupAfterGame)
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                position: Rect {
                                    top: Val::Px(4.0),
                                    left: Val::Px(4.0),
                                    ..Default::default()
                                },
                                size: Size::new(Val::Px(792.0), Val::Px(24.0)),
                                ..Default::default()
                            },
                            material: materials.add(Color::rgb_u8(147, 14, 58).into()),
                            // transform: Transform::from_scale(Vec3::new(0.5, 1.0, 1.0)),
                            ..Default::default()
                        })
                        .with(CleanupAfterGame)
                        .with(Healthbar)
                        .spawn(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                ..Default::default()
                            },
                            material: sprites.health_bar.clone(),
                            ..Default::default()
                        })
                        .with(CleanupAfterGame);
                });
        });
}

fn mouse_position(
    main_camera: Res<MainCamera>,
    windows: Res<Windows>,
    camera_query: Query<&Transform, With<Camera>>,
    cursor_moved_events: Res<Events<CursorMoved>>,
    mut cursor_moved_reader: Local<EventReader<CursorMoved>>,
    mut mouse_world_pos: ResMut<MouseWorldPos>,
    mut last_mouse_event: Local<Option<CursorMoved>>,
) {
    if let Some(last_event) = cursor_moved_reader.latest(&cursor_moved_events) {
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

const ROTATION_CLAMP: f32 = TAU / 8.;

fn ship_movement(
    mouse_input: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    mut ships: Query<(&mut Transform, &mut Velocity), With<Spaceship>>,
    time: Res<Time>,
    mouse_pos: Res<MouseWorldPos>,
) {
    for (mut transform, mut velocity) in ships.iter_mut() {
        let dir_to_target = mouse_pos.dir_from(transform.translation);

        let angle = Vec3::unit_y().angle_between(dir_to_target) * -dir_to_target.x.signum()
            - (ROTATION_CLAMP * 0.5);

        let angle = ROTATION_CLAMP * (angle / ROTATION_CLAMP).ceil();

        transform.rotation = Quat::from_rotation_z(angle);

        // let x =0.;
        // let y = 0.;
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
                Vec3::zero(),
            );
        }
    }
}

fn asteroids_hit_ship(
    cmd: &mut Commands,
    mut ship_query: Query<(&mut Spaceship, &Transform, &Collider)>,
    mut asteroids: Query<(Entity, &Transform, &Collider, &mut TextureAtlasSprite), With<Asteroid>>,
    mut states: ResMut<State<AppState>>,
) {
    for (mut ship, transform, collider) in ship_query.iter_mut() {
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

                cmd.remove::<(Velocity, Collider)>(asteroid)
                    .insert_one(asteroid, Lifetime::millis(200));

                ship.hp = ship.hp.saturating_sub(10);

                if ship.hp == 0 {
                    states.set_next(AppState::Menu).unwrap();
                    return;
                }
            }
        }
    }
}

pub fn bullets_hit_asteroids(
    cmd: &mut Commands,
    mut asteroids: Query<(&mut Asteroid, &Transform, &Collider)>,
    mut bullets: Query<(&mut Bullet, Entity, &Transform, &Collider)>,
) {
    for (mut asteroid, transform, collider) in asteroids.iter_mut() {
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
                asteroid.hit();
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

fn ship_eats_shards(
    cmd: &mut Commands,
    mut ships: Query<(&mut Spaceship, &Transform)>,
    mut shards: Query<(Entity, &mut Transform), With<Shard>>,
) {
    for (mut ship, ship_transform) in ships.iter_mut() {
        for (entity, transform) in shards.iter_mut() {
            let dist = ship_transform
                .translation
                .distance_squared(transform.translation);

            if dist < 400.0 {
                ship.score += 10;
                cmd.despawn(entity);
            }
        }
    }
}

fn camera_follow(
    spaceship: Res<PlayerSpaceship>,
    main_camera: Res<MainCamera>,
    spaceships: Query<&Transform, With<Spaceship>>,
    mut cameras: Query<&mut Transform, With<Camera>>,
) {
    if let Ok(ship) = spaceships.get(spaceship.0) {
        let mut camera = cameras.get_mut(main_camera.0).unwrap();
        camera.translation.x = ship.translation.x;
        camera.translation.y = ship.translation.y;
    }
}

fn hud(
    ship: Res<PlayerSpaceship>,
    ships: Query<&Spaceship>,
    mut healthbar: Query<&mut Style, With<Healthbar>>,
    mut score: Query<&mut Text, With<Score>>,
) {
    if let Ok(ship) = ships.get(ship.0) {
        for mut style in healthbar.iter_mut() {
            style.max_size.width = Val::Percent(ship.hp as f32);
        }

        for mut text in score.iter_mut() {
            text.value = ship.score.to_string();
        }
    }
}
