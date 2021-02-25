#![feature(total_cmp)]

mod macros;
mod asteroids;
mod lifetime;
mod menu;
mod movement;

use std::f32::consts::TAU;

use bevy::sprite::collide_aabb::collide;
use bevy::{input::mouse::MouseWheel, prelude::*};

use asteroids::*;
use bevy::render::camera::Camera;
use lifetime::*;
use movement::*;
use rand::{random, thread_rng, Rng as _};

pub const APP_STATE_STAGE: &str = "app_state_stage";

#[derive(Clone)]
pub enum AppState {
    Menu,
    InGame,
}

pub struct GameMaterials {
    ship: Handle<ColorMaterial>,
    bullet: Handle<ColorMaterial>,
    asteroid: Handle<TextureAtlas>,
    health_bar: Handle<ColorMaterial>,
    laser: Handle<ColorMaterial>,
    laser_impact: Handle<TextureAtlas>,
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
        }
    }
}

fn main() {
    let mut app_state_stage = StateStage::<AppState>::default();
    app_state_stage
        .on_state_enter(AppState::InGame, start_game.system())
        .on_state_update(AppState::InGame, continuous_movement.system())
        .on_state_update(AppState::InGame, continuous_rotation.system())
        .on_state_update(AppState::InGame, lifetime.system())
        .on_state_update(AppState::InGame, maximum_distance_from.system())
        .on_state_update(AppState::InGame, sprite_animation.system())
        .on_state_update(AppState::InGame, spawn_asteroids.system())
        .on_state_update(AppState::InGame, targeting.system())
        .on_state_update(AppState::InGame, ship_movement.system())
        .on_state_update(AppState::InGame, ship_weapon_switch.system())
        .on_state_update(AppState::InGame, ship_cannon.system())
        .on_state_update(AppState::InGame, ship_laser.system())
        .on_state_update(AppState::InGame, laser_beam.system())
        .on_state_update(AppState::InGame, laser_impact.system())
        .on_state_update(AppState::InGame, bullets_hit_asteroids.system())
        .on_state_update(AppState::InGame, laser_beams_hit_asteroids.system())
        .on_state_update(AppState::InGame, asteroid_damage.system())
        .on_state_update(AppState::InGame, asteroids_hit_ship.system())
        .on_state_update(AppState::InGame, ship_eats_shards.system())
        .on_state_update(AppState::InGame, hud.system())
        .on_state_exit(AppState::InGame, cleanup::<CleanupAfterGame>.system());

    let mut laser_beam_stage = SystemStage::parallel();
    laser_beam_stage.add_system(laser_beam_init.system());

    App::build()
        .add_plugins(DefaultPlugins)
        .add_resource(ClearColor(Color::rgb_u8(7, 0, 17)))
        .add_startup_system(setup.system())
        .add_resource(State::new(AppState::Menu))
        .init_resource::<GameMaterials>()
        .add_stage_after(stage::UPDATE, APP_STATE_STAGE, app_state_stage)
        .add_stage_after(APP_STATE_STAGE, "laser_beam_stage", laser_beam_stage)
        .add_plugin(menu::MenuPlugin)
        .run();
}

struct WeaponCannon(Timer);

impl Default for WeaponCannon {
    fn default() -> Self {
        WeaponCannon(Timer::from_seconds(0.150, false))
    }
}

enum WeaponLaser {
    Idle,
    Firing(Vec3),
}

impl Default for WeaponLaser {
    fn default() -> Self {
        WeaponLaser::Idle
    }
}

struct LaserBeam(bool);

struct LaserImpact;

pub struct HitableByLaser {
    damage_tick: Timer,
}

pub struct Spaceship {
    hp: u32,
    score: u32,
}

#[derive(Default)]
struct Target {
    pos: Vec3,
    dir: Vec3,
    dist: f32,
}

pub struct Collider(Vec2);

pub struct Bullet(bool);

struct SpriteAnimation {
    timer: Timer,
    current: u32,
    frames: u32,
}

impl SpriteAnimation {
    fn new(millis: u32, frames: u32) -> Self {
        SpriteAnimation {
            timer: Timer::from_seconds(millis as f32 / 1000., true),
            current: 0,
            frames,
        }
    }

    fn next(&mut self) -> u32 {
        self.current = (self.current + 1) % self.frames;
        self.current
    }
}

struct Healthbar;
struct Score;

struct CleanupAfterGame;

fn setup(commands: &mut Commands) {
    commands
        .spawn(Camera2dBundle::default())
        .spawn(CameraUiBundle::default());
}

fn start_game(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    sprites: Res<GameMaterials>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn(SpriteBundle {
            material: sprites.ship.clone(),
            ..Default::default()
        })
        .with(Spaceship { hp: 100, score: 0 })
        .with(CleanupAfterGame)
        .with(Target::default())
        // .with(ShipWeapons::Laser)
        .with(WeaponCannon::default())
        // .with(WeaponLaser::Idle)
        .with(Collider(Vec2::new(16., 48.)));

    // TEST ASTERIOD
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

#[derive(Default)]
struct TargetingState {
    cursor_moved_reader: EventReader<CursorMoved>,
}

fn targeting(
    mut state: Local<TargetingState>,
    windows: Res<Windows>,
    mut query: Query<(&mut Target, &Transform)>,
    camera_query: Query<&Transform, With<Camera>>,
    cursor_moved_events: Res<Events<CursorMoved>>,
) {
    use bevy::math::Vec4Swizzles as _;
    let mouse_move = state
        .cursor_moved_reader
        .latest(&cursor_moved_events)
        .map(|event| {
            let camera_transform = camera_query.iter().next().unwrap();
            let window = windows.get(event.id).unwrap();
            let window_size = Vec2::new(window.width() as f32, window.height() as f32);
            let p = event.position - window_size * 0.5;
            (camera_transform.compute_matrix() * p.extend(0.0).extend(1.0)).xy()
        });

    for (mut target, transform) in query.iter_mut() {
        if let Some(mouse_pos) = mouse_move {
            target.pos = mouse_pos.extend(0.);
        }

        target.dir = (target.pos - transform.translation).normalize();
        target.dist = target.pos.distance_squared(transform.translation);
    }
}

const SHIP_SPEED: f32 = 50.0;

fn ship_movement(
    mouse_input: Res<Input<MouseButton>>,
    mut spaceship_query: Query<(&mut Transform, &Target), With<Spaceship>>,
    time: Res<Time>,
) {
    for (mut transform, target) in spaceship_query.iter_mut() {
        if target.dist > 0.1 {
            let angle = Vec3::unit_y().angle_between(target.dir) * -target.dir.x.signum();
            transform.rotation = Quat::from_rotation_z(angle);

            let speed = if mouse_input.pressed(MouseButton::Right) {
                SHIP_SPEED * 5.0
            } else {
                0.0 //SHIP_SPEED
            };

            let movement = target.dir.normalize() * speed * time.delta_seconds();

            transform.translation += movement;
        }
    }
}

weapon_switcher!(StarshipWeapons {
    WeaponLaser <= WeaponCannon => WeaponLaser,
    WeaponCannon <= WeaponLaser => WeaponCannon,
});

fn ship_weapon_switch(
    cmd: &mut Commands,
    scroll_events: Res<Events<MouseWheel>>,
    mut scroll_reader: Local<EventReader<MouseWheel>>,
    mut query: Query<(Entity, &mut StarshipWeapons)>,
) {
    for event in scroll_reader.iter(&scroll_events) {
        // dbg!(event);
        for (entity, mut weapons) in query.iter_mut() {
            match event.y.total_cmp(&0.) {
                std::cmp::Ordering::Greater => weapons.next_weapon(entity, cmd),
                std::cmp::Ordering::Less => weapons.prev_weapon(entity, cmd),
                _ => {}
            }
        }
    }
}

fn ship_cannon(
    commands: &mut Commands,
    mouse_input: Res<Input<MouseButton>>,
    mut query: Query<(&mut WeaponCannon, &Transform, &Target), With<Spaceship>>,
    time: Res<Time>,
    materials: Res<GameMaterials>,
) {
    for (mut cannon, transform, target) in query.iter_mut() {
        cannon.0.tick(time.delta_seconds());

        if mouse_input.pressed(MouseButton::Left) && cannon.0.finished() {
            cannon.0.reset();
            commands
                .spawn(SpriteBundle {
                    material: materials.bullet.clone(),
                    transform: transform.clone(),
                    ..Default::default()
                })
                .with(CleanupAfterGame)
                .with(Bullet(false))
                .with(Movement::from(target.dir * 500.))
                .with(Lifetime::seconds(3))
                .with(Collider(Vec2::new(16., 16.)));
        }
    }
}

fn ship_laser(
    mouse_input: Res<Input<MouseButton>>,
    mut ships: Query<(&mut WeaponLaser, &Target), With<Spaceship>>,
) {
    let is_left_pressed = mouse_input.pressed(MouseButton::Left);

    for (mut laser, target) in ships.iter_mut() {
        *laser = match is_left_pressed {
            true => WeaponLaser::Firing(target.dir),
            false => WeaponLaser::Idle,
        }
    }
}

fn laser_beam_init(
    commands: &mut Commands,
    added_laser_weapons: Query<Entity, Added<WeaponLaser>>,
    sprites: Res<GameMaterials>,
) {
    for entity in added_laser_weapons.iter() {
        commands
            .spawn(SpriteBundle {
                material: sprites.laser.clone(),
                sprite: Sprite {
                    size: Vec2::new(1., 150.),
                    ..Default::default()
                },
                transform: Transform::from_translation(Vec3::new(0., 75., 5.)),
                visible: Visible {
                    is_visible: false,
                    ..Default::default()
                },
                ..Default::default()
            })
            .with(Parent(entity))
            .with(LaserBeam(false))
            .with(CleanupAfterGame)
            .with_children(|parent| {
                parent
                    .spawn(SpriteSheetBundle {
                        texture_atlas: sprites.laser_impact.clone(),
                        ..Default::default()
                    })
                    .with(LaserImpact)
                    .with(SpriteAnimation::new(150, 4))
                    .with(CleanupAfterGame);
            });
    }
}

fn laser_beam(
    cmd: &mut Commands,
    mut hit_this_frame: Local<Vec<Entity>>,
    time: Res<Time>,
    obstacles: Query<(Entity, &Transform), With<HitableByLaser>>,
    mut weapons: Query<(&WeaponLaser, &Transform)>,
    mut hitables: Query<(Entity, &mut HitableByLaser)>,
    mut laser_beams: Query<(
        Entity,
        &mut LaserBeam,
        &mut Sprite,
        &mut Visible,
        &mut Transform,
        &Parent,
    )>,
) {
    use bevy::math::Vec3Swizzles as _;

    hit_this_frame.clear();

    for (entity, mut laser_beam, mut sprite, mut visible, mut transform, parent) in
        laser_beams.iter_mut()
    {
        let (weapon, weapon_transform) = match weapons.get_mut(parent.0) {
            Ok(e) => e,
            Err(_) => {
                cmd.despawn_recursive(entity);
                continue;
            }
        };

        if let WeaponLaser::Firing(beam_dir) = weapon {
            let beam_origin = weapon_transform.translation.xy();

            let closest_hit = obstacles
                .iter()
                .filter_map(|(e, obstacle)| {
                    ray_circle_intersection(
                        beam_origin,
                        beam_dir.xy(),
                        obstacle.translation.xy(),
                        16.,
                    )
                    .map(|hit| (e, hit))
                })
                .min_by(|(_, hit1), (_, hit2)| {
                    hit1.distance_squared(beam_origin)
                        .total_cmp(&hit2.distance_squared(beam_origin))
                });

            if let Some((entity, _)) = closest_hit {
                hit_this_frame.push(entity);
            }

            let beam_length = closest_hit
                .map(|(_, hit)| beam_origin.distance(hit))
                .unwrap_or(1000.);

            sprite.size.y = beam_length;
            transform.translation.y = beam_length * 0.5;

            visible.is_visible = true;
            laser_beam.0 = closest_hit.is_some();
        } else {
            visible.is_visible = false;
            laser_beam.0 = false;
        }
    }

    for (e, mut hitable) in hitables.iter_mut() {
        if hit_this_frame.contains(&e) {
            hitable.damage_tick.tick(time.delta_seconds());
        } else {
            hitable.damage_tick.reset();
        }
    }
}

fn laser_impact(
    mut impacts: Query<(&mut Transform, &mut Visible, &Parent), With<LaserImpact>>,
    beams: Query<(&LaserBeam, &Sprite)>,
) {
    for (mut transform, mut visible, parent) in impacts.iter_mut() {
        if let Ok((beam, beam_sprite)) = beams.get(parent.0) {
            visible.is_visible = beam.0;
            transform.translation.y = beam_sprite.size.y * 0.5;
        } else {
            bevy::log::warn!("LaserImapct doesn't have Parent");
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

                cmd.remove::<(Movement, Collider)>(asteroid)
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

fn ship_eats_shards(
    cmd: &mut Commands,
    mut ship: Query<(&mut Spaceship, &Transform)>,
    mut shards: Query<(Entity, &mut Transform), With<Shard>>,
) {
    if let Some((mut ship, ship_transform)) = ship.iter_mut().next() {
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

fn sprite_animation(
    mut query: Query<(&mut SpriteAnimation, &mut TextureAtlasSprite)>,
    time: Res<Time>,
) {
    for (mut anim, mut sprite) in query.iter_mut() {
        if anim.timer.tick(time.delta_seconds()).just_finished() {
            sprite.index = anim.next();
        }
    }
}

fn hud(
    spaceship: Query<&Spaceship>,
    mut healthbar: Query<&mut Style, With<Healthbar>>,
    mut score: Query<&mut Text, With<Score>>,
) {
    if let Some(ship) = spaceship.iter().next() {
        for mut style in healthbar.iter_mut() {
            style.max_size.width = Val::Percent(ship.hp as f32);
        }

        for mut text in score.iter_mut() {
            text.value = ship.score.to_string();
        }
    }
}

fn cleanup<C: Component>(cmd: &mut Commands, query: Query<Entity, With<C>>) {
    for entity in query.iter() {
        cmd.despawn(entity);
    }
}

pub fn ray_circle_intersection(start: Vec2, dir: Vec2, origin: Vec2, radius: f32) -> Option<Vec2> {
    let l = -(start - origin);
    let tca = l.dot(dir);
    if tca < 0. {
        return None;
    }
    let d2 = l.dot(l) - tca * tca;
    if d2 > radius * radius {
        return None;
    }
    let thc = (radius * radius - d2).sqrt();
    Some(start + dir * (tca - thc))
}

fn around(point: Vec3, radius: f32) -> Vec3 {
    let dir = Quat::from_rotation_z(random::<f32>() * std::f32::consts::TAU);

    point + dir * Vec3::new(radius, 0., 0.0)
}
