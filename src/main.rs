mod map;
mod player;
mod ui;
mod constants;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(map::MapPlugin)    
        .add_plugins(player::PlayerPlugin)
        .add_plugins(ui::UiPlugin) // Adiciona o plugin que acabamos de criar
        .add_systems(Startup, setup_camera)
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}