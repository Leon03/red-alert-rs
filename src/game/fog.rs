use crate::game::MapSize;
use bevy::prelude::*;

/// Fog of War tile states.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FogState {
    /// Never explored - completely black
    #[default]
    Unexplored,
    /// Explored but not currently visible - dimmed
    Explored,
    /// Currently visible by any player unit/building - full brightness
    Visible,
}

/// Grid-based fog of war resource.
#[derive(Resource)]
pub struct FogOfWar {
    pub tiles: Vec<FogState>,
    pub width: u32,
    pub height: u32,
}

impl FogOfWar {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            tiles: vec![FogState::Unexplored; (width * height) as usize],
            width,
            height,
        }
    }

    pub fn index(&self, tile: (u32, u32)) -> usize {
        (tile.1 * self.width + tile.0) as usize
    }

    pub fn get(&self, tile: (u32, u32)) -> FogState {
        if tile.0 >= self.width || tile.1 >= self.height {
            return FogState::Unexplored;
        }
        self.tiles[self.index(tile)]
    }

    pub fn set(&mut self, tile: (u32, u32), state: FogState) {
        if tile.0 < self.width && tile.1 < self.height {
            let idx = self.index(tile);
            self.tiles[idx] = state;
        }
    }

    /// Reveal tiles in a circular radius around a world position.
    pub fn reveal_circle(&mut self, center_world: Vec3, radius: f32, tile_size: f32) {
        let center_tile = (
            (center_world.x / tile_size).floor() as i32,
            (center_world.y / tile_size).floor() as i32,
        );
        let radius_tiles = (radius / tile_size).ceil() as i32;

        for dy in -radius_tiles..=radius_tiles {
            for dx in -radius_tiles..=radius_tiles {
                let dist_sq = (dx * dx + dy * dy) as f32;
                if dist_sq <= (radius_tiles as f32).powi(2) {
                    let tile = (center_tile.0 + dx, center_tile.1 + dy);
                    if tile.0 >= 0 && tile.1 >= 0 {
                        let tile_u = (tile.0 as u32, tile.1 as u32);
                        if tile_u.0 < self.width && tile_u.1 < self.height {
                            let current = self.get(tile_u);
                            // Only upgrade state: Unexplored -> Explored/Visible, Explored -> Visible
                            let new_state = match current {
                                FogState::Unexplored => FogState::Visible,
                                FogState::Explored => FogState::Visible,
                                FogState::Visible => FogState::Visible,
                            };
                            self.set(tile_u, new_state);
                        }
                    }
                }
            }
        }
    }

    /// Step simulation: Visible -> Explored, Explored stays, Unexplored stays.
    pub fn tick(&mut self) {
        for tile_state in &mut self.tiles {
            *tile_state = match *tile_state {
                FogState::Visible => FogState::Explored,
                other => other,
            };
        }
    }
}

/// Component for entities that provide vision.
#[derive(Component)]
pub struct VisionSource {
    #[allow(dead_code)]
    pub range: f32,
}

/// System: Initialize fog of war.
pub fn init_fog_of_war(mut commands: Commands, map: Res<MapSize>) {
    commands.insert_resource(FogOfWar::new(map.width, map.height));
}

/// System: Update fog of war based on vision sources.
pub fn update_fog_of_war(
    mut fog: ResMut<FogOfWar>,
    q_vision: Query<&Transform, With<VisionSource>>,
    map: Res<MapSize>,
) {
    // First, decay all visible to explored
    fog.tick();

    // Then reveal around each vision source
    for transform in q_vision.iter() {
        fog.reveal_circle(transform.translation, 200.0, map.tile_size);
    }
}

/// System: Render fog of war as a grid overlay.
pub fn render_fog_of_war(
    mut commands: Commands,
    fog: Res<FogOfWar>,
    map: Res<MapSize>,
    q_existing: Query<Entity, With<FogTile>>,
) {
    // Despawn old fog tiles
    for entity in q_existing.iter() {
        commands.entity(entity).despawn();
    }

    let tile = map.tile_size;
    for x in 0..map.width {
        for y in 0..map.height {
            let state = fog.get((x, y));
            let (color, alpha) = match state {
                FogState::Unexplored => (Color::srgb(0.0, 0.0, 0.0), 0.95),
                FogState::Explored => (Color::srgb(0.1, 0.1, 0.1), 0.6),
                FogState::Visible => continue, // Don't render fog on visible tiles
            };

            let pos = Vec3::new(
                x as f32 * tile + tile / 2.0,
                y as f32 * tile + tile / 2.0,
                5.0, // Above map, below units
            );
            commands.spawn((
                FogTile,
                SpriteBundle {
                    sprite: Sprite {
                        color: color.with_alpha(alpha),
                        custom_size: Some(Vec2::splat(tile)),
                        ..default()
                    },
                    transform: Transform::from_translation(pos),
                    ..default()
                },
            ));
        }
    }
}

#[derive(Component)]
pub struct FogTile;

/// Add VisionSource to buildings and units.
pub fn add_vision_to_entities(
    mut commands: Commands,
    q_units: Query<Entity, (With<crate::game::Unit>, Without<VisionSource>)>,
    q_buildings: Query<Entity, (With<crate::game::building::Building>, Without<VisionSource>)>,
) {
    for entity in q_units.iter() {
        commands
            .entity(entity)
            .insert(VisionSource { range: 150.0 });
    }
    for entity in q_buildings.iter() {
        commands
            .entity(entity)
            .insert(VisionSource { range: 200.0 });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fog_of_war_initial_state() {
        let fog = FogOfWar::new(10, 10);
        assert_eq!(fog.get((0, 0)), FogState::Unexplored);
        assert_eq!(fog.get((9, 9)), FogState::Unexplored);
    }

    #[test]
    fn fog_reveal_circle() {
        let mut fog = FogOfWar::new(50, 50);
        fog.reveal_circle(Vec3::new(16.0 * 32.0, 16.0 * 32.0, 0.0), 100.0, 32.0);
        assert_eq!(fog.get((16, 16)), FogState::Visible);
        // Edge of radius
        assert_eq!(fog.get((13, 16)), FogState::Visible);
        assert_eq!(fog.get((19, 16)), FogState::Visible);
    }

    #[test]
    fn fog_tick_decay() {
        let mut fog = FogOfWar::new(10, 10);
        fog.set((5, 5), FogState::Visible);
        fog.tick();
        assert_eq!(fog.get((5, 5)), FogState::Explored);
        fog.tick();
        assert_eq!(fog.get((5, 5)), FogState::Explored); // Stays explored
    }

    #[test]
    fn fog_state_upgrades_only() {
        let mut fog = FogOfWar::new(10, 10);
        fog.set((5, 5), FogState::Explored);
        fog.reveal_circle(Vec3::new(5.0 * 32.0, 5.0 * 32.0, 0.0), 50.0, 32.0);
        assert_eq!(fog.get((5, 5)), FogState::Visible);
        fog.tick();
        assert_eq!(fog.get((5, 5)), FogState::Explored);
    }
}
