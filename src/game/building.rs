use crate::game::obstacle::{tile_to_world, world_to_tile};
use crate::game::MapSize;
use bevy::prelude::*;

/// Building type enum for different building categories.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum BuildingType {
    ConstructionYard,
    PowerPlant,
    Refinery,
    Barracks,
    WarFactory,
    ServiceDepot,
    TechCenter,
}

impl BuildingType {
    pub fn cost(&self) -> i32 {
        match self {
            BuildingType::ConstructionYard => 2000,
            BuildingType::PowerPlant => 500,
            BuildingType::Refinery => 2000,
            BuildingType::Barracks => 600,
            BuildingType::WarFactory => 2000,
            BuildingType::ServiceDepot => 800,
            BuildingType::TechCenter => 1500,
        }
    }

    pub fn size_tiles(&self) -> (u32, u32) {
        match self {
            BuildingType::ConstructionYard => (3, 3),
            BuildingType::PowerPlant => (2, 2),
            BuildingType::Refinery => (3, 2),
            BuildingType::Barracks => (3, 3),
            BuildingType::WarFactory => (4, 3),
            BuildingType::ServiceDepot => (3, 2),
            BuildingType::TechCenter => (3, 3),
        }
    }

    pub fn color(&self) -> Color {
        match self {
            BuildingType::ConstructionYard => Color::srgb(0.4, 0.4, 0.5),
            BuildingType::PowerPlant => Color::srgb(0.7, 0.7, 0.2),
            BuildingType::Refinery => Color::srgb(0.3, 0.5, 0.8),
            BuildingType::Barracks => Color::srgb(0.6, 0.3, 0.2),
            BuildingType::WarFactory => Color::srgb(0.5, 0.5, 0.5),
            BuildingType::ServiceDepot => Color::srgb(0.2, 0.6, 0.3),
            BuildingType::TechCenter => Color::srgb(0.8, 0.2, 0.6),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            BuildingType::ConstructionYard => "Construction Yard",
            BuildingType::PowerPlant => "Power Plant",
            BuildingType::Refinery => "Refinery",
            BuildingType::Barracks => "Barracks",
            BuildingType::WarFactory => "War Factory",
            BuildingType::ServiceDepot => "Service Depot",
            BuildingType::TechCenter => "Tech Center",
        }
    }
}

/// A building entity on the map.
#[derive(Component)]
pub struct Building {
    pub building_type: BuildingType,
    pub health: i32,
    pub max_health: i32,
    pub build_progress: f32, // 0.0 to 1.0
    pub is_built: bool,
    pub owner: Entity, // player entity that owns this building
}

/// Queue entry for building production.
#[derive(Clone, Debug)]
pub struct BuildQueueEntry {
    pub building_type: BuildingType,
    pub target_tile: (u32, u32),
    pub progress: f32,
}

/// Player building queue and state.
#[derive(Resource, Default)]
pub struct BuildQueue {
    pub queue: Vec<BuildQueueEntry>,
    pub selected_building_type: Option<BuildingType>,
    pub preview_tile: Option<(u32, u32)>,
    pub is_placement_mode: bool,
}

/// System: Handle building placement mode (right-click to cancel, left-click to place).
pub fn building_placement_system(
    buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    map: Res<MapSize>,
    blocked: Res<crate::game::obstacle::BlockedTiles>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut build_queue: ResMut<BuildQueue>,
    money: Res<crate::game::economy::PlayerMoney>,
) {
    let Ok(window) = q_windows.get_single() else {
        return;
    };
    let Ok((camera, cam_transform)) = camera_q.get_single() else {
        return;
    };

    // Cancel placement mode with Escape or right-click
    if keyboard.just_pressed(KeyCode::Escape) || buttons.just_pressed(MouseButton::Right) {
        build_queue.is_placement_mode = false;
        build_queue.selected_building_type = None;
        build_queue.preview_tile = None;
        return;
    }

    // If not in placement mode, check for building hotkeys
    if !build_queue.is_placement_mode {
        let building_type = if keyboard.just_pressed(KeyCode::Digit1) {
            Some(BuildingType::ConstructionYard)
        } else if keyboard.just_pressed(KeyCode::Digit2) {
            Some(BuildingType::PowerPlant)
        } else if keyboard.just_pressed(KeyCode::Digit3) {
            Some(BuildingType::Refinery)
        } else if keyboard.just_pressed(KeyCode::Digit4) {
            Some(BuildingType::Barracks)
        } else if keyboard.just_pressed(KeyCode::Digit5) {
            Some(BuildingType::WarFactory)
        } else if keyboard.just_pressed(KeyCode::Digit6) {
            Some(BuildingType::ServiceDepot)
        } else if keyboard.just_pressed(KeyCode::Digit7) {
            Some(BuildingType::TechCenter)
        } else {
            None
        };

        if let Some(bt) = building_type {
            if money.amount >= bt.cost() {
                build_queue.is_placement_mode = true;
                build_queue.selected_building_type = Some(bt);
            }
            return;
        }
        return;
    }

    // In placement mode: update preview position
    if let Some(cursor_pos) = window.cursor_position() {
        if let Some(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) {
            let tile = world_to_tile(world_pos.extend(0.0), map.tile_size);
            if tile.0 < map.width && tile.1 < map.height {
                build_queue.preview_tile = Some(tile);
            }
        }
    }

    // Left-click to confirm placement
    if buttons.just_pressed(MouseButton::Left) {
        if let (Some(bt), Some(tile)) =
            (build_queue.selected_building_type, build_queue.preview_tile)
        {
            // Check if tile is valid (not blocked, in bounds)
            let (w, h) = bt.size_tiles();
            let mut can_place = true;
            for dx in 0..w {
                for dy in 0..h {
                    let check_tile = (tile.0 + dx, tile.1 + dy);
                    if check_tile.0 >= map.width
                        || check_tile.1 >= map.height
                        || blocked.tiles.contains(&check_tile)
                    {
                        can_place = false;
                        break;
                    }
                }
            }
            if can_place {
                // Deduct cost immediately
                // Note: In real game, cost is deducted when construction starts
                build_queue.queue.push(BuildQueueEntry {
                    building_type: bt,
                    target_tile: tile,
                    progress: 0.0,
                });
                build_queue.is_placement_mode = false;
                build_queue.selected_building_type = None;
                build_queue.preview_tile = None;
            }
        }
    }
}

/// System: Process build queue - construct buildings over time.
pub fn build_queue_system(
    time: Res<Time>,
    mut build_queue: ResMut<BuildQueue>,
    mut money: ResMut<crate::game::economy::PlayerMoney>,
    mut commands: Commands,
    map: Res<MapSize>,
    mut blocked: ResMut<crate::game::obstacle::BlockedTiles>,
) {
    if build_queue.queue.is_empty() {
        return;
    }

    // Process first item in queue
    let entry = &mut build_queue.queue[0];
    let build_time = 30.0; // seconds to complete
    entry.progress += time.delta_seconds() / build_time;

    if entry.progress >= 1.0 {
        // Building complete - spawn it
        let bt = entry.building_type;
        let tile = entry.target_tile;
        let (w, h) = bt.size_tiles();
        let world = tile_to_world(tile, map.tile_size);

        // Add to blocked tiles
        for dx in 0..w {
            for dy in 0..h {
                blocked.tiles.insert((tile.0 + dx, tile.1 + dy));
            }
        }

        // Spawn building entity
        commands.spawn((
            Building {
                building_type: bt,
                health: 1000,
                max_health: 1000,
                build_progress: 1.0,
                is_built: true,
                owner: Entity::PLACEHOLDER, // would be player entity
            },
            crate::game::obstacle::Obstacle,
            SpriteBundle {
                sprite: Sprite {
                    color: bt.color(),
                    custom_size: Some(Vec2::new(
                        w as f32 * map.tile_size - 2.0,
                        h as f32 * map.tile_size - 2.0,
                    )),
                    ..default()
                },
                transform: Transform::from_xyz(
                    world.x + (w as f32 - 1.0) * map.tile_size / 2.0,
                    world.y + (h as f32 - 1.0) * map.tile_size / 2.0,
                    1.0,
                ),
                ..default()
            },
        ));

        // Remove from queue
        build_queue.queue.remove(0);
    }
}

/// System: Handle building construction progress visualization.
pub fn building_progress_system(mut q_buildings: Query<(&mut Building, &mut Sprite)>) {
    for (mut building, mut sprite) in q_buildings.iter_mut() {
        if !building.is_built {
            building.build_progress += 0.01; // placeholder
            if building.build_progress >= 1.0 {
                building.is_built = true;
                building.build_progress = 1.0;
            }
            // Visual feedback: alpha based on progress
            sprite.color.set_alpha(building.build_progress);
        }
    }
}

/// System: Handle building damage and destruction.
pub fn building_damage_system(
    mut commands: Commands,
    mut q_buildings: Query<(Entity, &mut Building)>,
    mut blocked: ResMut<crate::game::obstacle::BlockedTiles>,
    map: Res<MapSize>,
) {
    for (entity, mut building) in q_buildings.iter_mut() {
        if building.health <= 0 && building.is_built {
            // Remove from blocked tiles
            let (w, h) = building.building_type.size_tiles();
            // Find building position (would need Transform in query)
            // For now just despawn
            commands.entity(entity).despawn();
            // In full impl, we'd clear the blocked tiles
        }
    }
}

/// Check if a building can be placed at given tile.
pub fn can_place_building(
    tile: (u32, u32),
    bt: BuildingType,
    map: &MapSize,
    blocked: &crate::game::obstacle::BlockedTiles,
) -> bool {
    let (w, h) = bt.size_tiles();
    if tile.0 + w > map.width || tile.1 + h > map.height {
        return false;
    }
    for dx in 0..w {
        for dy in 0..h {
            if blocked.tiles.contains(&(tile.0 + dx, tile.1 + dy)) {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn building_costs() {
        assert_eq!(BuildingType::PowerPlant.cost(), 500);
        assert_eq!(BuildingType::Barracks.cost(), 600);
        assert_eq!(BuildingType::WarFactory.cost(), 2000);
    }

    #[test]
    fn building_sizes() {
        assert_eq!(BuildingType::PowerPlant.size_tiles(), (2, 2));
        assert_eq!(BuildingType::ConstructionYard.size_tiles(), (3, 3));
        assert_eq!(BuildingType::WarFactory.size_tiles(), (4, 3));
    }

    #[test]
    fn can_place_empty_map() {
        let map = MapSize {
            width: 50,
            height: 50,
            tile_size: 32.0,
        };
        let blocked = crate::game::obstacle::BlockedTiles::default();
        assert!(can_place_building(
            (10, 10),
            BuildingType::PowerPlant,
            &map,
            &blocked
        ));
        assert!(!can_place_building(
            (49, 49),
            BuildingType::PowerPlant,
            &map,
            &blocked
        )); // out of bounds
    }

    #[test]
    fn can_place_blocked() {
        let map = MapSize {
            width: 50,
            height: 50,
            tile_size: 32.0,
        };
        let mut blocked = crate::game::obstacle::BlockedTiles::default();
        blocked.tiles.insert((10, 10));
        assert!(!can_place_building(
            (10, 10),
            BuildingType::PowerPlant,
            &map,
            &blocked
        ));
    }
}
