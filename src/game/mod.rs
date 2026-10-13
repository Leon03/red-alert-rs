use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub mod building;
pub mod combat;
pub mod economy;
pub mod obstacle;
pub mod pathfinding;

use obstacle::{tile_to_world, update_blocked_tiles, world_to_tile, BlockedTiles};
use pathfinding::find_path;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapSize>()
            .init_resource::<Selection>()
            .init_resource::<BlockedTiles>()
            .init_resource::<economy::PlayerMoney>()
            .init_resource::<building::BuildQueue>()
            .add_systems(Startup, spawn_camera)
            .add_systems(Startup, spawn_grid)
            .add_systems(Startup, spawn_test_units)
            .add_systems(Startup, economy::spawn_ore_fields)
            .add_systems(Startup, economy::spawn_refinery)
            .add_systems(Startup, economy::spawn_harvesters)
            .add_systems(Startup, economy::spawn_money_display)
            .add_systems(Update, handle_click)
            .add_systems(Update, move_units)
            .add_systems(Update, update_blocked_tiles)
            .add_systems(Update, economy::assign_harvesters)
            .add_systems(Update, economy::harvester_navigation)
            .add_systems(Update, economy::mining_system)
            .add_systems(Update, economy::unloading_system)
            .add_systems(Update, economy::refinery_processing)
            .add_systems(Update, economy::update_money_display)
            .add_systems(Update, building::building_placement_system)
            .add_systems(Update, building::build_queue_system)
            .add_systems(Update, building::building_progress_system)
            .add_systems(Update, building::building_damage_system)
            .add_systems(Update, combat::equip_units)
            .add_systems(Update, combat::target_acquisition_system)
            .add_systems(Update, combat::combat_system)
            .add_systems(Update, combat::projectile_system)
            .add_systems(Update, combat::cleanup_dead_system);
    }
}

/// 地图网格大小 (tile count)
#[derive(Resource)]
pub struct MapSize {
    pub width: u32,
    pub height: u32,
    pub tile_size: f32,
}

impl Default for MapSize {
    fn default() -> Self {
        Self {
            width: 50,
            height: 50,
            tile_size: 32.0,
        }
    }
}

/// 当前选中的单位
#[derive(Resource, Default)]
pub struct Selection {
    #[expect(dead_code)]
    pub entities: Vec<Entity>,
}

/// 单位标记组件
#[derive(Component)]
pub struct Unit;

/// 可移动单位标记
#[derive(Component)]
pub struct Movable {
    pub speed: f32,
    /// Waypoints (world-space) the unit should follow in order.
    pub path: Vec<Vec3>,
}

fn spawn_camera(mut commands: Commands, map: Res<MapSize>) {
    let offset_x = map.width as f32 * map.tile_size / 2.0_f32;
    let offset_y = map.height as f32 * map.tile_size / 2.0_f32;
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(offset_x, offset_y, 1000.0),
        ..default()
    });
}

fn spawn_grid(mut commands: Commands, map: Res<MapSize>) {
    let tile = map.tile_size;
    let color = Color::srgb(0.15, 0.18, 0.12);
    for x in 0..map.width {
        for y in 0..map.height {
            let pos = Vec3::new(
                x as f32 * tile + tile / 2.0_f32,
                y as f32 * tile + tile / 2.0_f32,
                0.0,
            );
            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::splat(tile - 1.0)),
                    ..default()
                },
                transform: Transform::from_translation(pos),
                ..default()
            });
        }
    }
}

fn spawn_test_units(mut commands: Commands) {
    // 测试单位 - 红方 (玩家)
    for i in 0..5 {
        let x = 200.0 + i as f32 * 40.0;
        let y = 200.0;
        commands.spawn((
            Unit,
            Movable {
                speed: 80.0,
                path: Vec::new(),
            },
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.9, 0.2, 0.1),
                    custom_size: Some(Vec2::splat(24.0)),
                    ..default()
                },
                transform: Transform::from_xyz(x, y, 1.0),
                ..default()
            },
        ));
    }
    // 添加一些敌方单位用于测试战斗 (蓝方)
    for i in 0..3 {
        let x = 1000.0 + i as f32 * 40.0;
        let y = 800.0;
        commands.spawn((
            Unit,
            Movable {
                speed: 60.0,
                path: Vec::new(),
            },
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.1, 0.3, 0.9),
                    custom_size: Some(Vec2::splat(24.0)),
                    ..default()
                },
                transform: Transform::from_xyz(x, y, 1.0),
                ..default()
            },
        ));
    }
}

fn handle_click(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    map: Res<MapSize>,
    blocked: Res<BlockedTiles>,
    mut q_units: Query<(&Transform, &mut Movable), With<Unit>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok(window) = q_windows.get_single() else {
        return;
    };
    let Ok((camera, cam_transform)) = camera_q.get_single() else {
        return;
    };

    if buttons.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = window.cursor_position() {
            if let Some(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) {
                let end_tile = world_to_tile(world_pos.extend(0.0), map.tile_size);
                if end_tile.0 >= map.width || end_tile.1 >= map.height {
                    return;
                }

                for (transform, mut movable) in q_units.iter_mut() {
                    let start_tile = world_to_tile(transform.translation, map.tile_size);
                    let tile_path =
                        find_path(start_tile, end_tile, &blocked.tiles, map.width, map.height);
                    // Convert tile path to world-space waypoints
                    let waypoints: Vec<Vec3> = tile_path
                        .iter()
                        .map(|t| tile_to_world(*t, map.tile_size))
                        .collect();
                    movable.path = waypoints;
                }
            }
        }
    }
}

/// Move units along their waypoint paths.
pub fn move_units(mut q_units: Query<(&mut Transform, &mut Movable), With<Unit>>, time: Res<Time>) {
    for (mut transform, mut movable) in q_units.iter_mut() {
        if movable.path.is_empty() {
            continue;
        }

        let target = movable.path[0];
        let dir = target - transform.translation;
        if dir.length() > 2.0_f32 {
            let step = dir.normalize() * movable.speed * time.delta_seconds();
            // Don't overshoot
            if step.length() >= dir.length() {
                transform.translation = target;
            } else {
                transform.translation += step;
            }
        } else {
            // Reached this waypoint, remove it
            movable.path.remove(0);
        }
    }
}
