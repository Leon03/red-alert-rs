use bevy::prelude::*;

/// Weapon type for different attack profiles.
#[expect(
    dead_code,
    reason = "variants used in tests and future combat expansion"
)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WeaponType {
    MachineGun,
    Cannon,
    Missile,
    Laser,
    Flame,
}

impl WeaponType {
    #[allow(dead_code)]
    pub fn damage(&self) -> i32 {
        match self {
            WeaponType::MachineGun => 25,
            WeaponType::Cannon => 80,
            WeaponType::Missile => 120,
            WeaponType::Laser => 50,
            WeaponType::Flame => 40,
        }
    }

    #[allow(dead_code)]
    pub fn range(&self) -> f32 {
        match self {
            WeaponType::MachineGun => 120.0,
            WeaponType::Cannon => 180.0,
            WeaponType::Missile => 250.0,
            WeaponType::Laser => 150.0,
            WeaponType::Flame => 80.0,
        }
    }

    #[allow(dead_code)]
    pub fn cooldown(&self) -> f32 {
        match self {
            WeaponType::MachineGun => 0.3,
            WeaponType::Cannon => 2.0,
            WeaponType::Missile => 3.0,
            WeaponType::Laser => 1.5,
            WeaponType::Flame => 0.2,
        }
    }

    #[allow(dead_code)]
    pub fn projectile_speed(&self) -> f32 {
        match self {
            WeaponType::MachineGun => 400.0,
            WeaponType::Cannon => 300.0,
            WeaponType::Missile => 200.0,
            WeaponType::Laser => 1000.0,
            WeaponType::Flame => 0.0,
        }
    }

    #[allow(dead_code)]
    pub fn color(&self) -> Color {
        match self {
            WeaponType::MachineGun => Color::srgb(0.9, 0.9, 0.3),
            WeaponType::Cannon => Color::srgb(0.8, 0.4, 0.1),
            WeaponType::Missile => Color::srgb(0.9, 0.2, 0.2),
            WeaponType::Laser => Color::srgb(0.2, 0.9, 0.2),
            WeaponType::Flame => Color::srgb(1.0, 0.5, 0.0),
        }
    }
}

/// Attack capability component.
#[derive(Component)]
pub struct Attack {
    pub weapon_type: WeaponType,
    pub target: Option<Entity>,
    pub cooldown_timer: f32,
}

/// Health component for units and buildings.
#[derive(Component)]
pub struct Health {
    pub current: i32,
    #[expect(dead_code)]
    pub max: i32,
}

/// Projectile entity.
#[derive(Component)]
pub struct Projectile {
    pub target: Entity,
    pub damage: i32,
    pub speed: f32,
}

/// Unit type for determining weapon.
#[expect(dead_code, reason = "will be used when unit production is implemented")]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnitType {
    Infantry,
    Tank,
    Aircraft,
    Harvester,
}

impl UnitType {
    #[allow(dead_code)]
    pub fn default_weapon(&self) -> WeaponType {
        match self {
            UnitType::Infantry => WeaponType::MachineGun,
            UnitType::Tank => WeaponType::Cannon,
            UnitType::Aircraft => WeaponType::Missile,
            UnitType::Harvester => WeaponType::MachineGun,
        }
    }

    #[allow(dead_code)]
    pub fn max_health(&self) -> i32 {
        match self {
            UnitType::Infantry => 50,
            UnitType::Tank => 400,
            UnitType::Aircraft => 150,
            UnitType::Harvester => 200,
        }
    }

    #[allow(dead_code)]
    pub fn speed(&self) -> f32 {
        match self {
            UnitType::Infantry => 60.0,
            UnitType::Tank => 50.0,
            UnitType::Aircraft => 120.0,
            UnitType::Harvester => 60.0,
        }
    }
}

/// Add Attack and Health components to existing units.
pub fn equip_units(
    mut commands: Commands,
    q_units: Query<(Entity, &crate::game::Unit), Without<Attack>>,
) {
    for (entity, _) in q_units.iter() {
        commands.entity(entity).insert((
            Attack {
                weapon_type: WeaponType::MachineGun,
                target: None,
                cooldown_timer: 0.0,
            },
            Health {
                current: 100,
                max: 100,
            },
        ));
    }
}

/// System: Find targets for units with Attack component.
pub fn target_acquisition_system(
    mut q_attackers: Query<(
        &mut Attack,
        &Transform,
        Option<&crate::game::economy::Harvester>,
    )>,
    q_targets: Query<(Entity, &Transform, &Health), Without<crate::game::economy::Harvester>>,
) {
    for (mut attack, transform, harvester) in q_attackers.iter_mut() {
        if harvester.is_some() {
            continue;
        }

        let mut need_new_target = true;
        if let Some(target_entity) = attack.target {
            if let Ok((_, target_transform, target_health)) = q_targets.get(target_entity) {
                let dist = transform.translation.distance(target_transform.translation);
                if target_health.current > 0 && dist <= attack.weapon_type.range() {
                    need_new_target = false;
                }
            }
        }

        if need_new_target {
            let mut best: Option<(Entity, f32)> = None;
            for (target_entity, target_transform, target_health) in q_targets.iter() {
                if target_health.current <= 0 {
                    continue;
                }
                let dist = transform.translation.distance(target_transform.translation);
                if dist <= attack.weapon_type.range() && best.is_none_or(|(_, d)| dist < d) {
                    best = Some((target_entity, dist));
                }
            }
            attack.target = best.map(|(e, _)| e);
        }
    }
}

/// System: Fire weapons at targets.
pub fn combat_system(
    time: Res<Time>,
    mut q_attackers: Query<(&mut Attack, &Transform), Without<crate::game::economy::Harvester>>,
    q_targets: Query<(&Transform, &Health)>,
    mut commands: Commands,
) {
    for (mut attack, transform) in q_attackers.iter_mut() {
        attack.cooldown_timer -= time.delta_seconds();

        if attack.cooldown_timer > 0.0 {
            continue;
        }

        let Some(target_entity) = attack.target else {
            continue;
        };

        let Ok((target_transform, target_health)) = q_targets.get(target_entity) else {
            attack.target = None;
            continue;
        };

        if target_health.current <= 0 {
            attack.target = None;
            continue;
        }

        let dist = transform.translation.distance(target_transform.translation);
        if dist > attack.weapon_type.range() {
            attack.target = None;
            continue;
        }

        // Fire!
        let weapon = attack.weapon_type;
        attack.cooldown_timer = weapon.cooldown();

        if weapon.projectile_speed() > 0.0 {
            commands.spawn((
                Projectile {
                    target: target_entity,
                    damage: weapon.damage(),
                    speed: weapon.projectile_speed(),
                },
                SpriteBundle {
                    sprite: Sprite {
                        color: weapon.color(),
                        custom_size: Some(Vec2::splat(6.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        transform.translation.x,
                        transform.translation.y,
                        2.0,
                    ),
                    ..default()
                },
            ));
        }
    }
}

/// System: Move projectiles and apply damage on hit.
pub fn projectile_system(
    time: Res<Time>,
    mut q_projectiles: Query<(Entity, &mut Transform, &mut Projectile)>,
    q_target_transforms: Query<&Transform>,
    mut q_targets: Query<&mut Health>,
    mut commands: Commands,
) {
    for (entity, mut transform, projectile) in q_projectiles.iter_mut() {
        let Ok(target_transform) = q_target_transforms.get(projectile.target) else {
            commands.entity(entity).despawn();
            continue;
        };

        let Ok(mut target_health) = q_targets.get_mut(projectile.target) else {
            commands.entity(entity).despawn();
            continue;
        };

        let direction = target_transform.translation - transform.translation;
        let distance = direction.length();

        if distance <= 10.0 {
            target_health.current = (target_health.current - projectile.damage).max(0);
            commands.entity(entity).despawn();
        } else {
            let step = direction.normalize() * projectile.speed * time.delta_seconds();
            transform.translation += step;
        }
    }
}

/// System: Clean up dead entities.
pub fn cleanup_dead_system(mut commands: Commands, q_entities: Query<(Entity, &Health)>) {
    for (entity, health) in q_entities.iter() {
        if health.current <= 0 {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weapon_stats() {
        assert_eq!(WeaponType::MachineGun.damage(), 25);
        assert_eq!(WeaponType::Cannon.range(), 180.0);
        assert_eq!(WeaponType::Missile.cooldown(), 3.0);
        assert_eq!(WeaponType::Laser.projectile_speed(), 1000.0);
    }

    #[test]
    fn unit_type_stats() {
        assert_eq!(UnitType::Tank.max_health(), 400);
        assert_eq!(UnitType::Infantry.speed(), 60.0);
        assert_eq!(UnitType::Aircraft.default_weapon(), WeaponType::Missile);
    }

    #[test]
    fn health_damage() {
        let mut health = Health {
            current: 100,
            max: 100,
        };
        health.current = (health.current - 30).max(0);
        assert_eq!(health.current, 70);
        health.current = (health.current - 100).max(0);
        assert_eq!(health.current, 0);
    }
}
