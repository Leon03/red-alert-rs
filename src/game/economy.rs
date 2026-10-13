use crate::game::obstacle::{tile_to_world, world_to_tile};
use crate::game::MapSize;
use bevy::prelude::*;

/// Global player money supply.
#[derive(Resource)]
pub struct PlayerMoney {
    pub amount: i32,
}

impl Default for PlayerMoney {
    fn default() -> Self {
        Self { amount: 5000 }
    }
}

/// An ore deposit on the map. Can be mined by Harvesters.
#[derive(Component)]
pub struct OreField {
    pub amount: i32,
    #[expect(dead_code)]
    pub max_amount: i32,
}

/// A harvester unit that gathers ore and returns it to a refinery.
#[derive(Component)]
pub struct Harvester {
    pub capacity: i32,
    pub current_load: i32,
    pub state: HarvesterState,
    #[expect(dead_code)]
    pub home_refinery: Option<Entity>,
    pub target_ore: Option<Entity>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HarvesterState {
    Idle,
    MovingToOre,
    Mining,
    ReturningToRefinery,
    Unloading,
}

/// A refinery building that receives ore from harvesters and converts it to money.
#[derive(Component)]
pub struct Refinery {
    pub processing_rate: i32, // money per second
    pub stored_ore: i32,
}

/// Spawn ore fields at fixed positions on the map.
pub fn spawn_ore_fields(mut commands: Commands, map: Res<MapSize>) {
    let ore_color = Color::srgb(0.8, 0.6, 0.1);
    let ore_positions: [(u32, u32); 4] = [(10, 10), (40, 10), (10, 40), (40, 40)];
    for (tx, ty) in ore_positions {
        let world = tile_to_world((tx, ty), map.tile_size);
        commands.spawn((
            OreField {
                amount: 5000,
                max_amount: 5000,
            },
            crate::game::obstacle::Obstacle,
            SpriteBundle {
                sprite: Sprite {
                    color: ore_color,
                    custom_size: Some(Vec2::splat(map.tile_size * 2.0)),
                    ..default()
                },
                transform: Transform::from_xyz(world.x, world.y, 1.0),
                ..default()
            },
        ));
    }
}

/// Spawn a player refinery near the starting position.
pub fn spawn_refinery(mut commands: Commands, map: Res<MapSize>) {
    let pos = tile_to_world((3, 3), map.tile_size);
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
            transform: Transform::from_xyz(pos.x, pos.y, 1.0),
            ..default()
        },
    ));
}

/// Spawn two harvester units at the refinery.
pub fn spawn_harvesters(mut commands: Commands, map: Res<MapSize>) {
    let refinery_pos = tile_to_world((3, 3), map.tile_size);
    for i in 0..2 {
        let offset = i as f32 * 36.0;
        commands.spawn((
            crate::game::Unit,
            Harvester {
                capacity: 500,
                current_load: 0,
                state: HarvesterState::Idle,
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
                transform: Transform::from_xyz(refinery_pos.x + offset, refinery_pos.y - 40.0, 1.0),
                ..default()
            },
        ));
    }
}

/// Assign idle harvesters to the nearest available ore field.
pub fn assign_harvesters(
    mut q_harvesters: Query<(Entity, &Transform, &mut Harvester)>,
    q_ore: Query<(Entity, &Transform, &OreField)>,
    map: Res<MapSize>,
) {
    let _ = &map;
    for (entity, transform, mut harvester) in q_harvesters.iter_mut() {
        if harvester.state != HarvesterState::Idle {
            continue;
        }
        // Find nearest ore field with remaining ore
        let mut best: Option<(Entity, f32)> = None;
        for (ore_entity, ore_transform, ore) in q_ore.iter() {
            if ore.amount <= 0 {
                continue;
            }
            let dist = transform.translation.distance(ore_transform.translation);
            if best.is_none_or(|(_, d)| dist < d) {
                best = Some((ore_entity, dist));
            }
        }
        if let Some((ore_entity, _)) = best {
            harvester.target_ore = Some(ore_entity);
            harvester.state = HarvesterState::MovingToOre;
            let _ = entity;
        }
    }
}

/// Set movement paths for harvesters based on their state.
pub fn harvester_navigation(
    mut q_harvesters: Query<(&Transform, &mut Harvester, &mut crate::game::Movable)>,
    q_ore: Query<(&Transform, &OreField), Without<Harvester>>,
    q_refineries: Query<&Transform, With<Refinery>>,
    blocked: Res<crate::game::obstacle::BlockedTiles>,
    map: Res<MapSize>,
) {
    for (transform, mut harvester, mut movable) in q_harvesters.iter_mut() {
        match harvester.state {
            HarvesterState::MovingToOre => {
                if let Some(ore_entity) = harvester.target_ore {
                    if let Ok((ore_transform, ore)) = q_ore.get(ore_entity) {
                        if ore.amount <= 0 {
                            // Ore depleted, go idle
                            harvester.state = HarvesterState::Idle;
                            harvester.target_ore = None;
                            movable.path.clear();
                            continue;
                        }
                        if movable.path.is_empty() {
                            let start = world_to_tile(transform.translation, map.tile_size);
                            let end = world_to_tile(ore_transform.translation, map.tile_size);
                            let path = crate::game::pathfinding::find_path(
                                start,
                                end,
                                &blocked.tiles,
                                map.width,
                                map.height,
                            );
                            movable.path = path
                                .iter()
                                .map(|t| tile_to_world(*t, map.tile_size))
                                .collect();
                        }
                        // Check if we've arrived (path empty AND close to ore)
                        if movable.path.is_empty() {
                            let dist = transform.translation.distance(ore_transform.translation);
                            if dist < map.tile_size {
                                harvester.state = HarvesterState::Mining;
                            }
                        }
                    }
                }
            }
            HarvesterState::Mining => {
                // Mining happens in mining_system (needs mutable ore access)
            }
            HarvesterState::ReturningToRefinery => {
                if movable.path.is_empty() {
                    // Find refinery
                    if let Some(refinery_pos) = q_refineries.iter().next() {
                        let start = world_to_tile(transform.translation, map.tile_size);
                        let end = world_to_tile(refinery_pos.translation, map.tile_size);
                        let path = crate::game::pathfinding::find_path(
                            start,
                            end,
                            &blocked.tiles,
                            map.width,
                            map.height,
                        );
                        movable.path = path
                            .iter()
                            .map(|t| tile_to_world(*t, map.tile_size))
                            .collect();
                    }
                    // Check arrival
                    if let Some(refinery_pos) = q_refineries.iter().next() {
                        let dist = transform.translation.distance(refinery_pos.translation);
                        if dist < map.tile_size * 2.0 {
                            harvester.state = HarvesterState::Unloading;
                        }
                    }
                }
            }
            HarvesterState::Unloading => {
                // Handled in unloading_system (needs mutable refinery + harvester)
            }
            HarvesterState::Idle => {}
        }
    }
}

/// Mining system: harvesters extract ore from ore fields.
pub fn mining_system(
    time: Res<Time>,
    mut q_harvesters: Query<(&mut Harvester,)>,
    mut q_ore: Query<&mut OreField>,
) {
    for (mut harvester,) in q_harvesters.iter_mut() {
        if harvester.state != HarvesterState::Mining {
            continue;
        }
        let Some(ore_entity) = harvester.target_ore else {
            harvester.state = HarvesterState::Idle;
            continue;
        };
        let Ok(mut ore) = q_ore.get_mut(ore_entity) else {
            harvester.state = HarvesterState::Idle;
            harvester.target_ore = None;
            continue;
        };
        if ore.amount <= 0 {
            harvester.state = HarvesterState::Idle;
            harvester.target_ore = None;
            continue;
        }
        let mine_rate = 200; // ore per second
        let mined = (mine_rate as f32 * time.delta_seconds()) as i32;
        let actual = mined
            .min(ore.amount)
            .min(harvester.capacity - harvester.current_load);
        ore.amount -= actual;
        harvester.current_load += actual;
        if harvester.current_load >= harvester.capacity || ore.amount <= 0 {
            harvester.state = HarvesterState::ReturningToRefinery;
        }
    }
}

/// Unloading system: harvesters dump ore at refinery.
pub fn unloading_system(
    time: Res<Time>,
    mut q_harvesters: Query<&mut Harvester>,
    mut q_refineries: Query<&mut Refinery>,
) {
    for mut harvester in q_harvesters.iter_mut() {
        if harvester.state != HarvesterState::Unloading {
            continue;
        }
        // Find first refinery
        if let Some(mut refinery) = q_refineries.iter_mut().next() {
            let unload_rate = 300;
            let unloaded = (unload_rate as f32 * time.delta_seconds()) as i32;
            let actual = unloaded.min(harvester.current_load);
            harvester.current_load -= actual;
            refinery.stored_ore += actual;
            if harvester.current_load <= 0 {
                harvester.state = HarvesterState::Idle;
            }
        }
    }
}

/// Refinery processes stored ore into money.
pub fn refinery_processing(
    time: Res<Time>,
    mut q_refineries: Query<&mut Refinery>,
    mut money: ResMut<PlayerMoney>,
) {
    for mut refinery in q_refineries.iter_mut() {
        if refinery.stored_ore <= 0 {
            continue;
        }
        let processed = (refinery.processing_rate as f32 * time.delta_seconds()) as i32;
        let actual = processed.min(refinery.stored_ore);
        refinery.stored_ore -= actual;
        money.amount += actual;
    }
}

/// Display money on screen via a text entity.
pub fn spawn_money_display(mut commands: Commands) {
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "Money: $0",
                TextStyle {
                    font_size: 24.0,
                    color: Color::srgb(0.9, 0.9, 0.4),
                    ..default()
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
            ..default()
        },
        MoneyText,
    ));
}

#[derive(Component)]
pub struct MoneyText;

pub fn update_money_display(
    money: Res<PlayerMoney>,
    mut q_text: Query<&mut Text, With<MoneyText>>,
) {
    if money.is_changed() {
        if let Some(mut text) = q_text.iter_mut().next() {
            text.sections[0].value = format!("Money: ${}", money.amount);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harvester_capacity_default() {
        let h = Harvester {
            capacity: 500,
            current_load: 0,
            state: HarvesterState::Idle,
            home_refinery: None,
            target_ore: None,
        };
        assert_eq!(h.capacity, 500);
        assert_eq!(h.current_load, 0);
    }

    #[test]
    fn ore_field_default_amount() {
        let ore = OreField {
            amount: 5000,
            max_amount: 5000,
        };
        assert_eq!(ore.amount, ore.max_amount);
    }

    #[test]
    fn refinery_processing_math() {
        let refinery = Refinery {
            processing_rate: 100,
            stored_ore: 350,
        };
        // At 100/s rate, 350 ore takes 3.5 seconds
        let mut total_money = 0;
        let mut stored = refinery.stored_ore;
        let rate = refinery.processing_rate;
        // Simulate 4 seconds at 1s delta
        for _ in 0..4 {
            let processed = rate.min(stored);
            stored -= processed;
            total_money += processed;
        }
        assert_eq!(total_money, 350);
        assert_eq!(stored, 0);
    }
}
