use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct FacingDirection(pub f32);

#[derive(Component, Default)]
pub struct Position {
    pub x: i32,       // Usado para lógica de grid/mapa (índice)
    pub y: i32,       // Usado para lógica de grid/mapa (índice)
    pub x_float: f32, // Usado para movimento suave (mundo)
    pub y_float: f32, // Usado para movimento suave (mundo)
}