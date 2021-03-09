use super::*;
use bevy::{prelude::*, utils::HashSet};

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

pub struct WeaponSlot {
    pub system: Entity,
    pub slot: usize,
}

#[derive(Bundle)]
pub struct WeaponBundle<W> {
    weapon_slot: WeaponSlot,
    transform: Transform,
    global_transform: GlobalTransform,
    weapon: W,
}

impl<W> WeaponBundle<W> {
    pub fn new(weapon: W, slot: usize, system: Entity) -> Self {
        Self {
            weapon_slot: WeaponSlot { slot, system },
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            weapon,
        }
    }
}

pub struct WeaponCannon(Timer);

impl Default for WeaponCannon {
    fn default() -> Self {
        WeaponCannon(Timer::from_seconds(0.150, false))
    }
}
#[derive(Default)]
pub struct Bullet {
    pub already_hit: bool,
}

pub enum WeaponLaser {
    Idle,
    Firing(Vec3),
}

impl Default for WeaponLaser {
    fn default() -> Self {
        WeaponLaser::Idle
    }
}

pub struct LaserBeam(bool);

pub struct LaserImpact;

pub fn weapon_system_switch_weapon(
    // cmd: &mut Commands,
    scroll_events: Res<Events<MouseWheel>>,
    mut scroll_reader: Local<EventReader<MouseWheel>>,
    mut weapon_systems: Query<&mut WeaponSystem>,
) {
    for event in scroll_reader.iter(&scroll_events) {
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
    commands: &mut Commands,
    mut query: Query<(&mut WeaponCannon, &WeaponSlot, &GlobalTransform)>,
    weapon_systems: Query<&WeaponSystem>,
    time: Res<Time>,
    materials: Res<GameMaterials>,
) {
    for (mut cannon, weapon_slot, transform) in query.iter_mut() {
        cannon.0.tick(time.delta_seconds());

        let is_firing = weapon_systems
            .get(weapon_slot.system)
            .map_or(false, |system| system.is_firing(&weapon_slot));

        if is_firing && cannon.0.finished() {
            let shot_direction = transform.rotation * Vec3::unit_y();

            cannon.0.reset();
            commands
                .spawn(SpriteBundle {
                    material: materials.bullet.clone(),
                    transform: Transform::from_translation(transform.translation),
                    ..Default::default()
                })
                .with(CleanupAfterGame)
                .with(Bullet::default())
                .with(Velocity::from(shot_direction * BULLET_SPEED))
                .with(Lifetime::seconds(3))
                .with(Collider(Vec2::new(16., 16.)));
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

pub fn laser_beam(
    cmd: &mut Commands,
    time: Res<Time>,
    mut hitables: Query<(Entity, &GlobalTransform, &mut HitableByLaser)>,
    mut weapons: Query<(&WeaponLaser, &GlobalTransform)>,
    mut laser_beams: Query<(
        Entity,
        &mut LaserBeam,
        &mut Sprite,
        &mut Visible,
        &mut Transform,
        &Parent,
    )>,
    mut hit_this_frame: Local<HashSet<Entity>>,
) {
    use bevy::math::Vec3Swizzles as _;

    hit_this_frame.clear();

    for (entity, mut laser_beam, mut sprite, mut visible, mut transform, parent) in
        laser_beams.iter_mut()
    {
        let (weapon, weapon_transform) = match weapons.get_mut(parent.0) {
            Ok(e) => e,
            _ => {
                cmd.despawn_recursive(entity);
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

            let beam_length = closest_hit
                .as_ref()
                .map(|(hit, _, _)| beam_origin.distance(*hit))
                .unwrap_or(1000.);

            visible.is_visible = true;
            sprite.size.y = beam_length;
            transform.translation.y = beam_length * 0.5;
            laser_beam.0 = closest_hit.is_some();

            if let Some((_, entity, mut hitable)) = closest_hit {
                hitable.damage_tick.tick(time.delta_seconds());
                hit_this_frame.insert(entity);
            }
        } else {
            visible.is_visible = false;
            laser_beam.0 = false;
        }
    }

    for (e, _, mut hitable) in hitables.iter_mut() {
        if !hit_this_frame.contains(&e) {
            hitable.damage_tick.reset();
        }
    }
}

pub fn laser_impact(
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
