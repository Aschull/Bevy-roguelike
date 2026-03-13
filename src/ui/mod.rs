pub mod systems;

use bevy::prelude::*;
use crate::map::components::MapOverlay;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapOverlay>()
           .add_systems(Update, (
               systems::toggle_map,
               systems::draw_map_overlay, // O novo sistema
           ));
    }
}