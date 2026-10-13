use bevy::prelude::*;

/// Network role: server, client, or standalone.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum NetworkRole {
    #[default]
    Standalone,
    Server,
    Client,
}

/// Configuration for network play.
#[derive(Resource)]
pub struct NetworkConfig {
    pub role: NetworkRole,
    pub server_addr: String,
    pub port: u16,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            role: NetworkRole::Standalone,
            server_addr: "127.0.0.1".to_string(),
            port: 5000,
        }
    }
}

/// Component marking entities that should be replicated to clients.
#[derive(Component, Default)]
pub struct Replicate;

/// Component tracking which client owns an entity.
#[derive(Component, Default)]
pub struct Owner {
    pub client_id: u64,
}

/// Simple tick counter for sync.
#[derive(Resource, Default)]
pub struct NetworkTick {
    pub tick: u64,
}

/// System: Increment network tick each frame.
pub fn tick_system(mut tick: ResMut<NetworkTick>) {
    tick.tick += 1;
}

/// System: Log network role on startup.
pub fn log_network_role(config: Res<NetworkConfig>) {
    match config.role {
        NetworkRole::Standalone => info!("Running in standalone mode (no networking)"),
        NetworkRole::Server => info!(
            "Running as server on {}:{}",
            config.server_addr, config.port
        ),
        NetworkRole::Client => {
            info!(
                "Running as client, connecting to {}:{}",
                config.server_addr, config.port
            )
        }
    }
}

/// Plugin for networking (currently a stub — standalone mode only).
/// Real replication will be added when bevy_replicon API stabilises for Bevy 0.14.
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkConfig>()
            .init_resource::<NetworkTick>()
            .add_systems(Startup, log_network_role)
            .add_systems(Update, tick_system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_config_default() {
        let config = NetworkConfig::default();
        assert_eq!(config.role, NetworkRole::Standalone);
        assert_eq!(config.port, 5000);
    }

    #[test]
    fn network_tick_increments() {
        let mut tick = NetworkTick::default();
        assert_eq!(tick.tick, 0);
        tick.tick += 1;
        assert_eq!(tick.tick, 1);
    }

    #[test]
    fn network_role_default_is_standalone() {
        assert_eq!(NetworkRole::default(), NetworkRole::Standalone);
    }
}
