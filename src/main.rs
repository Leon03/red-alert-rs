use bevy::prelude::*;

mod game;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Red Alert RS".to_string(),
                resolution: (1280.0_f32, 720.0_f32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(game::GamePlugin)
        .add_plugins(game::network::NetworkPlugin)
        .run();
}
