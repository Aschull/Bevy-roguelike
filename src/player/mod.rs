pub mod components;
pub mod systems;

use bevy::prelude::*;
pub struct PlayerPlugin;
use crate::map::systems as map_systems;
use crate::MySets; // Importa o enum que você definios no main.rs
use crate::map::setup_fov_mesh; // Importa a função de setup da malha

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (
            systems::spawn_player,
            setup_fov_mesh, 
        ))
        .add_systems(
            Update,
            (
                // 1. Primeiro processamos o input (Move o Player)
                systems::handle_input.in_set(MySets::Input),
                
                // 2. Depois sincronizamos a posição da malha com a do Player
                systems::sync_fov_mesh.in_set(MySets::Logic),

                // 3. Por fim, geramos a geometria da luz já na posição nova
                systems::update_fov_geometry.in_set(MySets::Render),
            ).chain() // O .chain() aqui garante que nada atropele nada
        )
        .add_systems(
            PostUpdate, 
            (
                systems::camera_follow, 
                systems::camera_zoom
            ).chain()
        );
    }
}

#[derive(Resource)]
pub struct MovementTimer(pub Timer);
