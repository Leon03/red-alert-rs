use crate::game::building::{BuildQueue, BuildingType};
use crate::game::combat::{Attack, Health};
use crate::game::economy::{Harvester, OreField, PlayerMoney, Refinery};
use crate::game::fog::VisionSource;
use crate::game::MapSize;
use bevy::prelude::*;

/// AI player state and decision making.
#[derive(Resource, Default)]
pub struct AIPlayer {
    pub difficulty: AIDifficulty,
    pub last_build_time: f32,
    pub last_attack_time: f32,
    pub known_enemy_positions: Vec<Vec3>,
    pub build_order_index: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AIDifficulty {
    #[default]
    Easy,
    Medium,
    Hard,
}

impl AIDifficulty {
    pub fn build_interval(&self) -> f32 {
        match self {
            AIDifficulty::Easy => 30.0,
            AIDifficulty::Medium => 20.0,
            AIDifficulty::Hard => 12.0,
        }
    }

    pub fn attack_interval(&self) -> f32 {
        match self {
            AIDifficulty::Easy => 60.0,
            AIDifficulty::Medium => 40.0,
            AIDifficulty::Hard => 25.0,
        }
    }

    pub fn max_units(&self) -> usize {
        match self {
            AIDifficulty::Easy => 15,
            AIDifficulty::Medium => 30,
            AIDifficulty::Hard => 50,
        }
    }
}

/// Build order for AI (simplified).
const AI_BUILD_ORDER: &[BuildingType] = &[
    BuildingType::PowerPlant,
    BuildingType::Refinery,
    BuildingType::Barracks,
    BuildingType::WarFactory,
    BuildingType::ServiceDepot,
    BuildingType::TechCenter,
];

/// System: Spawn AI player at game start.
pub fn spawn_ai_player(mut commands: Commands) {
    commands.insert_resource(AIPlayer {
        difficulty: AIDifficulty::Medium,
        last_build_time: 0.0,
        last_attack_time: 0.0,
        known_enemy_positions: Vec::new(),
        build_order_index: 0,
    });
}

/// System: AI economic management - build harvesters, refineries.
pub fn ai_economy_system(
    time: Res<Time>,
    mut ai: ResMut<AIPlayer>,
    money: Res<PlayerMoney>,
    q_harvesters: Query<&Harvester>,
    q_refineries: Query<(&Refinery, &Transform)>,
    q_ore: Query<(&OreField, &Transform)>,
    mut commands: Commands,
    map: Res<MapSize>,
) {
    if money.amount < 500 {
        return; // Not enough money for anything
    }

    let harvester_count = q_harvesters.iter().count();
    let refinery_count = q_refineries.iter().count();
    let ore_fields = q_ore.iter().count();

    // Build more harvesters if we have refineries but few harvesters
    if refinery_count > 0 && harvester_count < refinery_count * 2 && money.amount >= 1400 {
        // Spawn harvester near first refinery
        if let Some((_, refinery_transform)) = q_refineries.iter().next() {
            commands.spawn((
                crate::game::Unit,
                Harvester {
                    capacity: 500,
                    current_load: 0,
                    state: crate::game::economy::HarvesterState::Idle,
                    home_refinery: None,
                    target_ore: None,
                },
                crate::game::Movable {
                    speed: 60.0,
                    path: Vec::new(),
                },
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.85, 0.75, 0.2),
                        custom_size: Some(Vec2::splat(20.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        refinery_transform.translation.x + 40.0,
                        refinery_transform.translation.y - 40.0,
                        1.0,
                    ),
                    ..default()
                },
            ));
        }
    }

    // Build refinery if we have ore but no refinery
    if refinery_count == 0 && ore_fields > 0 && money.amount >= 2000 {
        if let Some((_, ore_transform)) = q_ore.iter().next() {
            commands.spawn((
                Refinery {
                    processing_rate: 100,
                    stored_ore: 0,
                },
                crate::game::obstacle::Obstacle,
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.3, 0.5, 0.8),
                        custom_size: Some(Vec2::splat(map.tile_size * 2.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        ore_transform.translation.x - 80.0,
                        ore_transform.translation.y - 80.0,
                        1.0,
                    ),
                    ..default()
                },
            ));
        }
    }
}

/// System: AI building construction following build order.
pub fn ai_building_system(
    time: Res<Time>,
    mut ai: ResMut<AIPlayer>,
    money: Res<PlayerMoney>,
    mut build_queue: ResMut<BuildQueue>,
    q_buildings: Query<(&crate::game::building::Building, &Transform)>,
    blocked: Res<crate::game::obstacle::BlockedTiles>,
) {
    ai.last_build_time += time.delta_seconds();

    if ai.last_build_time < ai.difficulty.build_interval() {
        return;
    }

    if ai.build_order_index >= AI_BUILD_ORDER.len() {
        return; // Completed build order
    }

    if money.amount < 500 {
        return;
    }

    // Try to place next building in build order
    let building_type = AI_BUILD_ORDER[ai.build_order_index];
    let cost = building_type.cost();

    if money.amount >= cost {
        // Find placement spot near existing buildings
        let placement = find_ai_building_spot(building_type, &q_buildings, &blocked);
        if let Some(tile) = placement {
            build_queue
                .queue
                .push(crate::game::building::BuildQueueEntry {
                    building_type,
                    target_tile: tile,
                    progress: 0.0,
                });
            ai.build_order_index += 1;
            ai.last_build_time = 0.0;
        }
    }
}

/// Find a valid placement spot for AI building near existing structures.
fn find_ai_building_spot(
    building_type: BuildingType,
    q_buildings: &Query<(&crate::game::building::Building, &Transform)>,
    blocked: &crate::game::obstacle::BlockedTiles,
) -> Option<(u32, u32)> {
    let (w, h) = building_type.size_tiles();

    // Try spots near existing buildings first
    for (_, building_transform) in q_buildings.iter() {
        // We'd need Transform in the query - simplified
    }

    // Fallback: scan from base corner (assume 50x50 map)
    let map_width = 50u32;
    let map_height = 50u32;
    let start_x = 3;
    let start_y = 3;
    for y in start_y..(map_height - h) {
        for x in start_x..(map_width - w) {
            let mut can_place = true;
            for dy in 0..h {
                for dx in 0..w {
                    if blocked.tiles.contains(&(x + dx, y + dy)) {
                        can_place = false;
                        break;
                    }
                }
                if !can_place {
                    break;
                }
            }
            if can_place {
                return Some((x, y));
            }
        }
    }

    None
}

/// System: AI unit production from War Factory / Barracks.
pub fn ai_unit_production_system(
    time: Res<Time>,
    mut ai: ResMut<AIPlayer>,
    money: Res<PlayerMoney>,
    q_war_factories: Query<&Transform, With<crate::game::building::Building>>,
    q_barracks: Query<&Transform, With<crate::game::building::Building>>,
    q_units: Query<&crate::game::Unit>,
    mut commands: Commands,
) {
    // Count our units
    let unit_count = q_units.iter().count();
    if unit_count >= ai.difficulty.max_units() {
        return;
    }

    if money.amount < 600 {
        return;
    }

    ai.last_attack_time += time.delta_seconds();
    if ai.last_attack_time < ai.difficulty.attack_interval() {
        return;
    }

    // Spawn a tank from War Factory or infantry from Barracks
    let spawn_pos = q_war_factories
        .iter()
        .next()
        .or_else(|| q_barracks.iter().next())
        .map(|t| t.translation)
        .unwrap_or_else(|| Vec3::new(1000.0, 800.0, 0.0));

    // Alternate between tank and infantry
    let is_tank = ai.build_order_index % 2 == 0;
    ai.build_order_index += 1;

    if is_tank && money.amount >= 800 {
        commands.spawn((
            crate::game::Unit,
            crate::game::Movable {
                speed: 50.0,
                path: Vec::new(),
            },
            Attack {
                weapon_type: crate::game::combat::WeaponType::Cannon,
                target: None,
                cooldown_timer: 0.0,
            },
            Health {
                current: 400,
                max: 400,
            },
            VisionSource { range: 150.0 },
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.1, 0.3, 0.9), // Enemy blue
                    custom_size: Some(Vec2::splat(28.0)),
                    ..default()
                },
                transform: Transform::from_xyz(spawn_pos.x, spawn_pos.y - 50.0, 1.0),
                ..default()
            },
        ));
    } else if !is_tank && money.amount >= 400 {
        commands.spawn((
            crate::game::Unit,
            crate::game::Movable {
                speed: 60.0,
                path: Vec::new(),
            },
            Attack {
                weapon_type: crate::game::combat::WeaponType::MachineGun,
                target: None,
                cooldown_timer: 0.0,
            },
            Health {
                current: 50,
                max: 50,
            },
            VisionSource { range: 120.0 },
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.1, 0.3, 0.9),
                    custom_size: Some(Vec2::splat(20.0)),
                    ..default()
                },
                transform: Transform::from_xyz(spawn_pos.x, spawn_pos.y + 50.0, 1.0),
                ..default()
            },
        ));
    }

    ai.last_attack_time = 0.0;
}

/// System: AI attack behavior - send units to enemy base.
pub fn ai_attack_system(
    mut q_ai_units: Query<(&mut crate::game::Movable, &Transform), With<crate::game::Unit>>,
    q_enemy_buildings: Query<&Transform, With<crate::game::building::Building>>,
    blocked: Res<crate::game::obstacle::BlockedTiles>,
    map: Res<MapSize>,
) {
    // Find enemy construction yard or base center
    let target = q_enemy_buildings
        .iter()
        .next()
        .map(|t| t.translation)
        .unwrap_or_else(|| Vec3::new(200.0, 200.0, 0.0));

    for (mut movable, transform) in q_ai_units.iter_mut() {
        // Only command idle units (no path)
        if movable.path.is_empty() {
            let start_tile =
                crate::game::obstacle::world_to_tile(transform.translation, map.tile_size);
            let end_tile = crate::game::obstacle::world_to_tile(target, map.tile_size);

            if end_tile.0 < map.width && end_tile.1 < map.height {
                let path = crate::game::pathfinding::find_path(
                    start_tile,
                    end_tile,
                    &blocked.tiles,
                    map.width,
                    map.height,
                );
                movable.path = path
                    .iter()
                    .map(|t| crate::game::obstacle::tile_to_world(*t, map.tile_size))
                    .collect();
            }
        }
    }
}

/// System: AI scouting - remember enemy positions.
pub fn ai_scouting_system(
    mut ai: ResMut<AIPlayer>,
    q_enemy_units: Query<
        &Transform,
        (
            With<crate::game::Unit>,
            Without<crate::game::economy::Harvester>,
        ),
    >,
    q_enemy_buildings: Query<&Transform, With<crate::game::building::Building>>,
) {
    for transform in q_enemy_units.iter().chain(q_enemy_buildings.iter()) {
        let pos = transform.translation;
        // Add to known positions if not already known
        if !ai
            .known_enemy_positions
            .iter()
            .any(|p| p.distance(pos) < 100.0)
        {
            ai.known_enemy_positions.push(pos);
        }
    }

    // Limit memory
    if ai.known_enemy_positions.len() > 50 {
        ai.known_enemy_positions.drain(0..25);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_difficulty_settings() {
        assert_eq!(AIDifficulty::Easy.build_interval(), 30.0);
        assert_eq!(AIDifficulty::Medium.build_interval(), 20.0);
        assert_eq!(AIDifficulty::Hard.build_interval(), 12.0);
    }

    #[test]
    fn ai_max_units() {
        assert_eq!(AIDifficulty::Easy.max_units(), 15);
        assert_eq!(AIDifficulty::Medium.max_units(), 30);
        assert_eq!(AIDifficulty::Hard.max_units(), 50);
    }

    #[test]
    fn ai_build_order_not_empty() {
        assert!(!AI_BUILD_ORDER.is_empty());
    }
}
