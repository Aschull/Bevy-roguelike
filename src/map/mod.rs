pub mod components;
pub mod systems;

use bevy::prelude::*;

use crate::map::components::{Map, PlayerStart};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        let (map, x, y) = Map::new_labyrinth(300, 350);
        
        app.insert_resource(map)
           .insert_resource(PlayerStart(x, y))
           // Remova o Startup de spawn_map! Não queremos spawnar tudo no início.
           .add_systems(Update, (
               // Ele pode rodar junto com os outros, Bevy gerencia o paralelismo
               systems::render_world_fast, 
               systems::render_fov,
           ));
        app.add_systems(Update, systems::render_world_fast);
    }
}