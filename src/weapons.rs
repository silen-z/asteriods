use super::*;
use bevy::{ecs::component::Component, prelude::*, sprite::Anchor, utils::HashSet};

#[derive(Component)]
pub struct WeaponSystem {
    pub current: usize,
    pub slots: usize,
    pub is_firing: bool,
}

impl WeaponSystem {
    fn is_firing(&self, weapon: &WeaponSlot) -> bool {
        self.is_firing && weapon.slot == self.current
    }

    fn prev(&mut self) -> usize {
        if self.current == 0 {
            self.current = self.slots - 1;
        } else {
            self.current -= 1;
        }
        self.current
    }

    fn next(&mut self) -> usize {
        if self.current == self.slots - 1 {
            self.current = 0;
        } else {
            self.current += 1;
        }
        self.current
    }
}

#[derive(Component)]
pub struct WeaponSlot {
    pub system: Entity,
    pub slot: usize,
}

#[derive(Bundle)]
pub struct WeaponBundle<W: Component> {
    weapon_slot: WeaponSlot,
    transform: Transform,
    global_transform: GlobalTransform,
    weapon: W,
}

impl<W: Component> WeaponBundle<W> {
    pub fn new(weapon: W, slot: usize, system: Entity) -> Self {
        Self {
            weapon_slot: WeaponSlot { slot, system },
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            weapon,
        }
    }
}

#[derive(Component)]
pub struct WeaponCannon(Timer);

impl Default for WeaponCannon {
    fn default() -> Self {
        WeaponCannon(Timer::from_seconds(0.150, false))
    }
}
#[derive(Component, Default)]
pub struct Bullet {
    pub already_hit: bool,
}

#[derive(Component)]
pub enum WeaponLaser {
    Idle,
    Firing(Vec3),
}

impl Default for WeaponLaser {
    fn default() -> Self {
        WeaponLaser::Idle
    }
}

#[derive(Component)]
pub struct LaserBeam {
    origin: Entity,
    impacted: bool,
}

#[derive(Component)]
pub struct LaserImpact;

pub fn weapon_system_switch_weapon(
    mut scroll_reader: EventReader<MouseWheel>,
    mut weapon_systems: Query<&mut WeaponSystem>,
) {
    for event in scroll_reader.iter() {
        for mut system in weapon_systems.iter_mut() {
            match event.y.total_cmp(&0.) {
                std::cmp::Ordering::Less => system.prev(),
                std::cmp::Ordering::Greater => system.next(),
                _ => continue,
            };
        }
    }
}

pub fn weapon_system_fire(
    mut weapon_systems: Query<&mut WeaponSystem>,
    mouse_input: Res<Input<MouseButton>>,
) {
    let is_firing = mouse_input.pressed(MouseButton::Left);

    for mut system in weapon_systems.iter_mut() {
        system.is_firing = is_firing;
    }
}

const BULLET_SPEED: f32 = 1000.;

pub fn ship_cannon(
    mut commands: Commands,
    mut query: Query<(&mut WeaponCannon, &WeaponSlot, &GlobalTransform)>,
    weapon_systems: Query<&WeaponSystem>,
    time: Res<Time>,
    materials: Res<GameMaterials>,
    mouse_pos: Res<MouseWorldPos>,
) {
    for (mut cannon, weapon_slot, transform) in query.iter_mut() {
        cannon.0.tick(time.delta());

        let is_firing = weapon_systems
            .get(weapon_slot.system)
            .map_or(false, |system| system.is_firing(&weapon_slot));

        if is_firing && cannon.0.finished() {
            let shot_direction = mouse_pos.dir_from(transform.translation);

            let angle = shot_direction.angle_between(Vec3::Y) * -shot_direction.x.signum();

            cannon.0.reset();
            commands
                .spawn_bundle(SpriteBundle {
                    texture: materials.bullet.clone().into(),
                    transform: Transform {
                        translation: transform.translation,
                        rotation: Quat::from_rotation_z(angle),
                        ..default()
                    },
                    ..default()
                })
                .insert(CleanupAfterGame)
                .insert(Bullet::default())
                .insert(Velocity::from(shot_direction.normalize() * BULLET_SPEED))
                .insert(Lifetime::seconds(3))
                .insert(Collider(Vec2::new(16., 16.)));
        }
    }
}

pub fn ship_laser(
    mut lasers: Query<(&mut WeaponLaser, &WeaponSlot, &GlobalTransform)>,
    weapon_systems: Query<&WeaponSystem>,
    mouse_pos: Res<MouseWorldPos>,
) {
    for (mut laser, weapon_slot, transform) in lasers.iter_mut() {
        let is_firing = weapon_systems
            .get(weapon_slot.system)
            .map_or(false, |system| system.is_firing(&weapon_slot));

        if is_firing {
            *laser = WeaponLaser::Firing(mouse_pos.dir_from(transform.translation));
        } else {
            *laser = WeaponLaser::Idle;
        }
    }
}

pub fn laser_beam_init(
    mut commands: Commands,
    added_laser_weapons: Query<Entity, Added<WeaponLaser>>,
    sprites: Res<GameMaterials>,
) {
    for entity in added_laser_weapons.iter() {
        commands
            .spawn_bundle(SpriteBundle {
                texture: sprites.laser.clone().into(),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(2., 1.)),
                    anchor: Anchor::BottomCenter,
                    ..default()
                },
                visibility: Visibility {
                    is_visible: false,
                    ..default()
                },
                ..default()
            })
            .insert(LaserBeam {
                origin: entity,
                impacted: false,
            })
            .insert(CleanupAfterGame)
            .with_children(|parent| {
                parent
                    .spawn_bundle(SpriteSheetBundle {
                        transform: Transform::from_scale(Vec2::splat(2.).extend(1.)),
                        texture_atlas: sprites.laser_impact.clone(),
                        ..default()
                    })
                    .insert(LaserImpact)
                    .insert(SpriteAnimation::new(150, 4))
                    .insert(CleanupAfterGame);
            });
    }
}

pub fn laser_beam(
    mut cmd: Commands,
    time: Res<Time>,
    mut hitables: Query<(Entity, &GlobalTransform, &mut HitableByLaser)>,
    mut weapons: Query<(&WeaponLaser, &GlobalTransform)>,
    mut laser_beams: Query<(
        Entity,
        &mut LaserBeam,
        &mut Sprite,
        &mut Visibility,
        &mut Transform,
    )>,
    mut hit_this_frame: Local<HashSet<Entity>>,
) {
    use bevy::math::Vec3Swizzles as _;

    hit_this_frame.clear();

    for (entity, mut laser_beam, sprite, mut visible, mut transform) in laser_beams.iter_mut() {
        let (weapon, weapon_transform) = match weapons.get_mut(laser_beam.origin) {
            Ok(e) => e,
            _ => {
                cmd.entity(entity).despawn_recursive();
                continue;
            }
        };

        if let WeaponLaser::Firing(beam_dir) = weapon {
            let beam_origin = weapon_transform.translation.xy();

            let closest_hit = hitables
                .iter_mut()
                .filter_map(|(e, obstacle, hitable)| {
                    math::ray_circle_intersection(
                        beam_origin,
                        beam_dir.xy(),
                        obstacle.translation.xy(),
                        16.,
                    )
                    .map(|hit| (hit, e, hitable))
                })
                .min_by(|(hit1, _, _), (hit2, _, _)| {
                    f32::total_cmp(
                        &hit1.distance_squared(beam_origin),
                        &hit2.distance_squared(beam_origin),
                    )
                });

            let beam_end = closest_hit
                .as_ref()
                .map_or_else(|| beam_origin + (beam_dir.xy() * 1000.), |(hit, _, _)| *hit);

            visible.is_visible = true;
            laser_beam.impacted = closest_hit.is_some();

            let mut sprite: Mut<Sprite> = sprite;

            for s in sprite.custom_size.iter_mut() {
                s.y = beam_origin.distance(beam_end);
            }

            transform.translation = beam_origin.extend(1.);
            transform.rotation =
                Quat::from_rotation_z(beam_dir.angle_between(Vec3::Y) * -beam_dir.x.signum());

            if let Some((_, entity, mut hitable)) = closest_hit {
                hitable.damage_tick.tick(time.delta());
                hit_this_frame.insert(entity);
            }
        } else {
            visible.is_visible = false;
            laser_beam.impacted = false;
        }
    }

    for (e, _, mut hitable) in hitables.iter_mut() {
        if !hit_this_frame.contains(&e) {
            hitable.damage_tick.reset();
        }
    }
}

pub fn laser_impact(
    mut impacts: Query<(&mut Transform, &mut Visibility, &Parent), With<LaserImpact>>,
    beams: Query<(&LaserBeam, &Sprite)>,
) {
    for (mut transform, mut visible, parent) in impacts.iter_mut() {
        if let Ok((beam, beam_sprite)) = beams.get(parent.0) {
            visible.is_visible = beam.impacted;
            transform.translation.y = beam_sprite.custom_size.unwrap().y;
        } else {
            bevy::log::warn!("LaserImapct doesn't have Parent");
        }
    }
}
