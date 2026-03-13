mod map;
mod player;
mod ui;
mod constants;

use bevy::prelude::*;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // Dica: evita texturas borradas
        .add_plugins(map::MapPlugin)    
        .add_plugins(player::PlayerPlugin)
        .add_plugins(ui::UiPlugin) 
        .add_systems(Startup, setup_camera)
        
        // Configuração de Ordem Global (Isso resolve a tremedeira e o conflito)
        .configure_sets(
            Update,
            (
                MySets::Input,
                MySets::Logic,
                MySets::Render,
            ).chain(), // Garante que Input -> Logic -> Render aconteça nessa ordem
        )
        
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .run();
}

// Crie esses "Sets" para organizar seus sistemas entre plugins diferentes
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum MySets {
    Input,
    Logic,
    Render,
}


pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 999.0), // Z bem alto
        ..default()
    });
}