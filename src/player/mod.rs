pub mod components;
pub mod systems;

use bevy::prelude::*;
pub struct PlayerPlugin;
use crate::map::systems as map_systems;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, systems::spawn_player)
           .add_systems(Update, (
               systems::handle_input,
               systems::update_translation,
               systems::camera_follow,
               systems::camera_zoom,
               systems::update_fov,         // 3. Calcula o que o @ vê (Raycasting)
               map_systems::render_fov,
           ).chain()) // .chain() garante que eles rodem na ordem escrita
           .insert_resource(MovementTimer(Timer::from_seconds(0.15, TimerMode::Repeating)));
    }
}

#[derive(Resource)]
pub struct MovementTimer(pub Timer);
