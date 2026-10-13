use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapSize>()
            .init_resource::<Selection>()
            .add_systems(Startup, spawn_camera)
            .add_systems(Startup, spawn_grid)
            .add_systems(Startup, spawn_test_units)
            .add_systems(Update, handle_click);
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
    pub entities: Vec<Entity>,
}

/// 单位标记组件
#[derive(Component)]
pub struct Unit;

/// 可移动单位标记
#[derive(Component)]
pub struct Movable {
    pub speed: f32,
    pub target: Option<Vec3>,
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
    // 测试单位 - 红方
    for i in 0..5 {
        let x = 200.0 + i as f32 * 40.0;
        let y = 200.0;
        commands.spawn((
            Unit,
            Movable {
                speed: 80.0,
                target: None,
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
}

fn handle_click(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    map: Res<MapSize>,
    mut q_units: Query<(&mut Transform, &mut Movable), With<Unit>>,
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
                // 选中全部单位（简化版）
                for (mut _transform, mut movable) in q_units.iter_mut() {
                    movable.target = Some(world_pos.extend(0.0));
                }
                let _ = &map;
            }
        }
    }
}

/// 移动单位系统（当前未插入到 schedule，预留扩展）
pub fn move_units(mut q_units: Query<(&mut Transform, &mut Movable), With<Unit>>, time: Res<Time>) {
    for (mut transform, mut movable) in q_units.iter_mut() {
        if let Some(target) = movable.target {
            let dir = target - transform.translation;
            if dir.length() > 2.0 {
                let step = dir.normalize() * movable.speed * time.delta_seconds();
                transform.translation += step;
            } else {
                movable.target = None;
            }
        }
    }
}
