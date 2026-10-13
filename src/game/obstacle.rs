use crate::game::MapSize;
use bevy::prelude::*;
use std::collections::HashSet;

/// Marks an entity as an obstacle that blocks a tile on the grid.
#[derive(Component)]
pub struct Obstacle;

/// Resource holding the set of blocked tile coordinates.
#[derive(Resource, Default)]
pub struct BlockedTiles {
    pub tiles: HashSet<(u32, u32)>,
}

/// System that scans all `Obstacle` entities and updates the `BlockedTiles` resource.
pub fn update_blocked_tiles(
    map: Res<MapSize>,
    q_obstacles: Query<&Transform, With<Obstacle>>,
    mut blocked: ResMut<BlockedTiles>,
) {
    blocked.tiles.clear();
    for transform in q_obstacles.iter() {
        let tile = world_to_tile(transform.translation, map.tile_size);
        if tile.0 < map.width && tile.1 < map.height {
            blocked.tiles.insert(tile);
        }
    }
}

/// Convert a world-space `Vec3` position to tile coordinates.
pub fn world_to_tile(pos: Vec3, tile_size: f32) -> (u32, u32) {
    let x = (pos.x / tile_size).floor() as u32;
    let y = (pos.y / tile_size).floor() as u32;
    (x, y)
}

/// Convert tile coordinates to the center of that tile in world space.
pub fn tile_to_world(tile: (u32, u32), tile_size: f32) -> Vec3 {
    Vec3::new(
        tile.0 as f32 * tile_size + tile_size / 2.0_f32,
        tile.1 as f32 * tile_size + tile_size / 2.0_f32,
        0.0,
    )
}
